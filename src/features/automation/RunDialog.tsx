import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../../app/ipc";
import { Dialog } from "../../components/Dialog";
import { Select } from "../../components/Select";
import type { AutomationSummary, NodeConnectionPreview, RunMode, RunTarget, WorkspaceSnapshot } from "../../generated/contracts";

export function RunDialog({ workspace, item, initialMode, dismiss, started, busyChanged }: {
  workspace: WorkspaceSnapshot; item: AutomationSummary; initialMode: RunMode;
  dismiss: () => void; started: () => void; busyChanged: (busy: boolean) => void;
}) {
  const playbook = item.kind === "playbook" || item.kind === "ansible";
  const [python, setPython] = useState(() => localStorage.getItem("ghosh.controller.python") || "");
  const [revision] = useState(workspace.revision);
  const [targets, setTargets] = useState<string[]>([workspace.nodes[0]?.id || "@local"]);
  const [mode, setMode] = useState<RunMode>(playbook ? initialMode : "apply");
  const [input, setInput] = useState("{}"), [busy, setBusy] = useState(false), [error, setError] = useState("");
  const [prompts, setPrompts] = useState<NodeConnectionPreview[]>([]), [credentials, setCredentials] = useState<Record<string, string>>({});
  const inFlight = useRef(false);
  const stale = workspace.revision !== revision || workspace.external_changed || !workspace.trusted || workspace.read_only;
  useEffect(() => { busyChanged(busy); return () => busyChanged(false); }, [busy, busyChanged]);
  const submit = async () => {
    if (inFlight.current || stale || !targets.length) return;
    inFlight.current = true; setBusy(true); setError("");
    const context = { workspace_id: workspace.id, revision };
    try {
      const parsed: unknown = JSON.parse(input);
      if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) throw new Error("Inputs must be a JSON object.");
      if (playbook && python && !python.startsWith("/")) throw new Error("Choose the absolute Python path from your Ansible controller environment.");
      if (python) localStorage.setItem("ghosh.controller.python", python);
      const connections: RunTarget[] = [], required: NodeConnectionPreview[] = [];
      for (const node_id of targets) {
        if (node_id === "@local") { connections.push({ node_id, connection: null }); continue; }
        const prepared = await api.connectionPreview({ context, node_id });
        if (prepared.credential_prompt && !credentials[node_id]) required.push(prepared);
        connections.push({ node_id, connection: { ticket: prepared.ticket, allow_unknown: true, credential: credentials[node_id] || null } });
      }
      if (required.length) { setPrompts(required); return; }
      await api.runStart({ automation: { context, id: item.id, kind: item.kind }, targets: connections, mode, input, controller_python: python || null });
      setCredentials({}); started();
    } catch (e) { setCredentials({}); setPrompts([]); setError(errorMessage(e)); }
    finally { inFlight.current = false; setBusy(false); }
  };
  return <Dialog title="run-title" busy={busy} dismiss={dismiss}><form onSubmit={e => { e.preventDefault(); void submit(); }}>
    <p className="eyebrow">SEQUENTIAL EXECUTION</p><h2 id="run-title">Run {item.validation.name}</h2>
    <label htmlFor="run-mode">Mode</label><Select id="run-mode" data-dialog-focus value={mode} disabled={busy || !!prompts.length} onChange={e => setMode(e.target.value as RunMode)}>
      {playbook && <option value="check">Check · report drift</option>}<option value="apply">{playbook ? "Apply playbook" : "Execute script"}</option>
    </Select>
    {playbook && <><label htmlFor="controller-python">Ansible controller · absolute Python path</label><input id="controller-python" placeholder="/path/to/controller/bin/python" value={python} disabled={busy || !!prompts.length} onChange={e => setPython(e.target.value)} /><p className="field-help">Leave empty for simple schema: 1 scenarios; they need no Ansible. Standard Ansible requires Python 3.12+ with requirements-controller.txt installed. This path is saved on this machine.</p></>}
    <fieldset disabled={busy || !!prompts.length} className="run-targets"><legend>Targets · choose up to eight</legend>{[{ id: "@local", host: "This machine" }, ...workspace.nodes].map(node => <label key={node.id} title={node.host}><input type="checkbox" checked={targets.includes(node.id)} onChange={e => setTargets(current => e.target.checked ? [...current, node.id] : current.filter(id => id !== node.id))} /><span>{node.id === "@local" ? "Local machine" : node.id}<small>{node.host}</small></span></label>)}</fieldset>
    <label htmlFor="run-input">Inputs · JSON object</label><textarea id="run-input" rows={3} spellCheck={false} value={input} disabled={busy || !!prompts.length} onChange={e => setInput(e.target.value)} />
    <p className="field-help">Non-secret inputs become Ansible extra variables, or GHOSH_INPUT for scripts. Output is stored locally in Runs; scripts must not print secrets. Ansible Check uses --check; tasks and plugins may still execute according to your playbook. Stop does not undo completed changes.</p>
    {prompts.map(prompt => <div key={prompt.node_id}><label htmlFor={`run-credential-${prompt.node_id}`}>{prompt.node_id} · {prompt.credential_prompt}</label><input id={`run-credential-${prompt.node_id}`} type="password" autoComplete="off" maxLength={4096} disabled={busy} value={credentials[prompt.node_id] || ""} onChange={e => setCredentials(current => ({ ...current, [prompt.node_id]: e.target.value }))} /></div>)}
    {!!prompts.length && <p className="field-help">SSH credentials are used only for this run and are never saved.</p>}
    {stale && <p className="form-error" role="alert">Reload or resume the workspace before running.</p>}{error && <p className="form-error" role="alert">{error}</p>}
    <div className="modal-actions"><button type="button" disabled={busy} onClick={dismiss}>Cancel</button><button className="primary" disabled={busy || stale || !targets.length || targets.length > 8}>{busy ? "Connecting…" : prompts.length ? "Connect and start" : mode === "check" ? "Start check" : "Start run"}</button></div>
  </form></Dialog>;
}
