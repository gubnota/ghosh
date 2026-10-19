"""Private stdin/NDJSON adapter for upstream Ansible Runner, not a YAML engine."""
import importlib.metadata
import json
import os
from pathlib import Path
import signal
import sys

os.umask(0o077)
stopped = False


def stop(*_):
    global stopped
    stopped = True


signal.signal(signal.SIGTERM, stop)


def emit(value):
    print(json.dumps(value, ensure_ascii=True), flush=True)


def main():
    try:
        import ansible_runner
        versions = {name: importlib.metadata.version(name) for name in ("ansible-core", "ansible-runner")}
        if versions != {"ansible-core": "2.20.10", "ansible-runner": "2.4.3"}:
            raise RuntimeError("Select the pinned controller: ansible-core 2.20.10, ansible-runner 2.4.3.")
    except ImportError:
        raise RuntimeError("Ansible controller missing. Install requirements-controller.txt in a Python virtual environment and select its Python.") from None
    cfg = json.load(sys.stdin)
    root = Path(cfg["private_dir"])
    node = cfg["node"]
    host = cfg["host"]
    env = {
        "PATH": str(Path(sys.executable).parent) + os.pathsep + os.environ.get("PATH", ""),
        "ANSIBLE_HOST_KEY_CHECKING": "True",
        "ANSIBLE_NOCOLOR": "1", "ANSIBLE_DISPLAY_ARGS_TO_STDOUT": "False",
        "ANSIBLE_RETRY_FILES_ENABLED": "False", "ANSIBLE_LOCAL_TEMP": str(root / "local"),
        "ANSIBLE_LOG_PATH": str(root / "ansible.log"),
    }
    variables = cfg["input"]
    if any(k.startswith("ansible_") or k in ("inventory_hostname", "hostvars", "groups") for k in variables):
        raise RuntimeError("Run inputs must not override Ansible connection or inventory variables.")
    # Inject reviewed connection variables at highest precedence. The original
    # project/inventory stays in place; roles, includes and group_vars still work.
    connection = {"ansible_host": host, "ansible_user": cfg["user"], "ansible_port": cfg["port"]}
    identity = cfg.get("identity", {})
    kwargs = {}
    if node == "@local":
        connection.update(ansible_connection="local", ansible_python_interpreter=sys.executable, ansible_remote_tmp=str(root / "remote"))
    else:
        pin = root / "known_hosts"
        pin.write_text(cfg["host_pin"])
        # Ignore arbitrary ssh_config routes; the selected Ghosh endpoint and
        # fresh native handshake own host trust, not a global accept-new setting.
        import shlex
        connection.update(ansible_connection="ssh", ansible_ssh_args="-F /dev/null -o ControlMaster=no -o StrictHostKeyChecking=yes -o GlobalKnownHostsFile=/dev/null -o UserKnownHostsFile=" + shlex.quote(str(pin)))
        if identity.get("ssh_key"):
            kwargs["ssh_key"] = identity["ssh_key"]
            # Runner adds precisely this key to a private, per-invocation agent.
            env["SSH_AUTH_SOCK"] = ""
            if cfg.get("credential"):
                kwargs["passwords"] = {r"Enter passphrase for .*:": cfg["credential"]}
        elif identity.get("agent_socket"):
            public = root / "identity.pub"
            public.write_text(identity["public_key"])
            connection["ansible_ssh_common_args"] = "-o IdentitiesOnly=yes -i " + shlex.quote(str(public))
            env["SSH_AUTH_SOCK"] = identity["agent_socket"]
        elif identity.get("password_auth"):
            connection["ansible_password"] = cfg.get("credential", "")
            connection["ansible_ssh_password_mechanism"] = "ssh_askpass"
    variables.update(connection)
    inventory = {"all": {"hosts": {"localhost" if node == "@local" else node: {}}}}
    alias = "localhost" if node == "@local" else node
    if cfg.get("inventory"):
        generated = root / "selected-inventory.json"
        generated.write_text(json.dumps(inventory))
        inventory = [cfg["inventory"], str(generated)]
    def sanitized(value):
        if isinstance(value, dict):
            if value.get("_ansible_no_log") or "censored" in value:
                return "[no_log]"
            return {k: sanitized(v) for k, v in value.items() if k in ("msg", "stdout", "stderr", "rc", "results", "changed", "skipped")}
        if isinstance(value, list):
            return [sanitized(v) for v in value[:256]]
        return value

    observed = 0

    def event(data):
        nonlocal observed
        kind = data.get("event", "")
        if kind == "runner_on_start":
            emit({"active_task": str(data.get("event_data", {}).get("task", "Ansible task"))[:200]})
            return False
        if kind not in ("runner_on_ok", "runner_on_failed", "runner_on_unreachable", "runner_on_skipped"):
            return False
        observed += 1
        details = data.get("event_data", {})
        result = details.get("res", {})
        hidden = result.get("_ansible_no_log") or "censored" in result
        state = "failed" if kind in ("runner_on_failed", "runner_on_unreachable") and not details.get("ignore_errors") else "skipped" if kind == "runner_on_skipped" else "drift" if cfg["check"] and result.get("changed") else "changed" if result.get("changed") else "ok"
        output = "[no_log]" if hidden else json.dumps(sanitized(result), ensure_ascii=True)
        encoded = output.encode("utf-8")
        output = encoded[:32768].decode("utf-8", errors="ignore")
        emit({"node_id": node, "name": str(details.get("task", "Ansible task"))[:200], "phase": "check" if cfg["check"] else "apply", "state": state, "output": output, "exit_code": result.get("rc"), "truncated": len(encoded) > 32768})
        # Never retain raw events (module args, invocation and secrets) in reports.
        return False

    emit({"status": "Controller ready", "versions": versions})
    run = ansible_runner.run(
        private_data_dir=str(root), project_dir=cfg["project"], playbook=cfg["playbook"],
        inventory=inventory, extravars=variables, limit=alias, forks=1,
        cmdline="--check" if cfg["check"] else "", quiet=True,
        envvars=env, event_handler=event, cancel_callback=lambda: stopped,
        settings={"job_timeout": 3600, "idle_timeout": 600, "suppress_ansible_output": True},
        **kwargs,
    )
    if not observed and run.rc == 0:
        raise RuntimeError("No tasks matched the selected inventory alias. Check play hosts/groups and the Ghosh node ID; nothing was executed.")
    if run.status in ("canceled", "timeout"):
        return 130
    return run.rc or 0


try:
    sys.exit(main())
except Exception as error:
    # Known setup failures only; do not dump config, tracebacks or credentials.
    emit({"error": str(error)[:1000] if isinstance(error, RuntimeError) else "Ansible controller setup failed; check project paths, runtime and inventory."})
    sys.exit(1)
