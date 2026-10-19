# Ghosh workspace schemas

Status: workspace/node schemas 1 and 2 implemented; later entity/service fields
remain target architecture. Current boundaries: API_CONTRACTS.md. Date: 2026-10-18.

**Accepted SSH amendment:** [SSH_KEY_STORAGE.md](SSH_KEY_STORAGE.md) requires
existing/custom file keys with no SSH Keychain storage. Schema 2 is implemented:
manifest `[ssh] default_key = "<registry-id>"`; node `[auth] method = "file"` and
optional `key_id = "<registry-id>"`. Omit node key_id to inherit workspace/global.
These are opaque local bindings, never private material or file URIs. Schema 1
examples below are legacy examples; SSH Keychain bindings are inert for transport.
Explicit workspace-identity migration preserves a recovery manifest; opening does
not migrate or export keys. Unrelated SecretRef consumers remain separate.
This document refines Appendix A1–A8 of [the build prompt](ghosh-prompt.md). Rules below resolve ambiguities and
supersede conflicting sample comments. Example endpoints/commits are illustrative;
recipe application requires real immutable revisions and checksums.

## Package and identity

A workspace is a directory, not an archive. Finder registration uses exported UTI
`app.ghosh.workspace`, conforming to `com.apple.package`, extension `ghosh`, and a
Viewer/Editor document association. Distribution must test double-click/open URL
and multiple packages. Bundle identifier is provisionally `app.ghosh.desktop`;
signing-team ownership must be resolved before release.

```text
homelab.ghosh/
  ghosh.toml
  nodes/*.toml
  profiles/*.toml
  tunnels/*.toml
  certs/*.toml
  recipes/*.toml
  playbooks/*.{yaml,yml,js,py}
  recordings/*.json
  scripts/
  templates/
  data/
  trust/known_hosts
  state/                         # ignored runtime data
  .gitignore
```

- `schema` is an integer on each independent document. Workspace/node support 1/2;
  other entity projections remain version 1. Schema-2 workspaces permit unchanged v1 nodes.
- Workspace ID is a canonical ULID, generated once. Entity IDs and filenames match
  `[a-z0-9][a-z0-9_-]{0,63}`. IDs are unique within each entity type.
- Profile `data_store` must parse as a UUID; reject malformed values on load.
  Create a random UUID v4 once per profile, serialize canonically, and keep it
  immutable for that profile. Distinct profiles cannot reuse a store identifier.
  Reject external edits that change a loaded profile’s identifier; creating a
  separate profile/explicit import clone generates a new identity instead.
- `direct` is a reserved route ID, with no required tunnel file.
- Paths in `[paths]` and file API paths are relative, resolved beneath the package;
  reject absolute paths, traversal and symlink escape, including at write time.
- Every configured directory is distinct; nested overlap with `state/` is invalid.
- Node IDs, jumps, profile routes, defaults, cert selectors and script references
  must resolve; reject ProxyJump cycles. Deleted references need explicit repair.
- Unknown keys and comments survive TOML edits using an editable syntax tree.
  JSON recordings preserve extension keys; YAML edits preserve unknown mapping
  values, with textual formatting preservation best effort and disclosed.
- Supported known fields are validated. Unknown fields are retained but inert;
  unknown executable task or recording step types cannot execute.
- `schema > supported` opens read-only, with no autosave or execution.
- Future migrations are explicit, backed up, cancellable before commit and atomic;
  v1 has no fictitious v0 migration. Never downgrade on save.
- Secrets use `secret://keychain/<id>`; values, credentials, browser data and
  certificate private keys never enter configuration, state or exports.
- Browser session data lives in OS-managed stores outside the package; a copied
  workspace does not carry login sessions. Detect duplicate profile UUIDs across
  concurrently open packages and require a clone/import choice before browser use.
- `trust/known_hosts` contains public host pins; mismatches are errors, not TOFU.
  Unknown-host enrollment is an explicit fingerprint review operation.
- `.gitignore` includes `/state/`, `/data/`, `*.tmp`, `.DS_Store`; users may explicitly
  opt selected non-sensitive data exports into version control.

## Field requirements and defaults

| Document | Required fields | Defaults / validation |
| --- | --- | --- |
| Manifest | schema; workspace.name/id/created | RFC3339 UTC created; paths use package defaults; name nonempty |
| SSH defaults | none | user=root, port=22, connect_timeout_s=15, concurrency=8, known_hosts=workspace |
| Script defaults | none | runtime=js, timeout_s=600, permissions=ask |
| Secrets | none | backend=keychain on macOS; service defaults to workspace-ID namespace |
| Node | schema/id/host/user/auth.method | port=22; tunnel=direct; labels=[]; jump omitted/empty means none |
| Profile | schema/id/label/color/data_store | direct; kill_switch=true; UA preset default; follow-window viewport |
| Tunnel | schema/id/kind | vless requires engine and uri_ref; listen loopback only, health_interval_s=15 |
| Certificate | schema/id/domains/provider/method/email/token_ref/sync_to/remote_path | renew_before_days=30; mode=0640; owner=root:root; reload=node-hook |
| Recipe | schema/id/install_dir/python/venv/comfy.repo/comfy.commit | requirements optional; models=[]; min_free_gb=0 unless set |
| Playbook | schema/name/targets/tasks | vars={}; handlers={}; check mode never runs handlers |
| Recording | schema/id/profile/steps | bounded step count; each type validates its own fields |

Ports are 1–65535, except managed tunnel listeners allow 0. Counts/timeouts are
positive and bounded by application limits. Colors are `#RRGGBB`; zoom > 0;
viewport width 0 follows the window. Domains are DNS names, including leading
wildcards. Permissions use octal strings; certificate private keys reject world
readability and require an explicit group when group readable. Models require
64-hex SHA-256 and relative destinations below install/shared model roots.
Pins must resolve to immutable git commits before any apply. Remote paths may
use `~` only through remote home expansion, never local expansion or shell eval.

For schema-1 node auth: `key` requires key_ref; `password` requires password_ref;
`agent` requires neither. Schema 2 uses `key_id` or an inherited file key; password
auth without password_ref asks per connection and never persists the entered secret.
Legacy secret references remain inert for the native direct transport. `agent_socket` is an optional local path, validated at use.
ProxyJump composes each hop's declared route; host identity remains the final
hostname/port, not the local proxy endpoint. Agent forwarding is off by default.

For tunnel kind `http` or `socks5`, use `endpoint` (credential-free URL) and optional
`credentials_ref`. `vless` requires `uri_ref` and engine `xray | sing-box`; URI
options are engine-validated. `direct` ignores proxy fields. Health requests must
traverse the actual proxy and report both process readiness and remote reachability.

## A1. ghosh.toml (workspace manifest, at `<name>.ghosh/ghosh.toml`)

```toml
schema = 1

[workspace]
name = "homelab"
id = "01J9Z3K8X0Q5T7R2M4N6P8V1WA"      # ULID, generated once
created = 2026-10-18T10:00:00Z
default_profile = "work"
default_tunnel = "direct"

[paths]
nodes = "nodes"
profiles = "profiles"
tunnels = "tunnels"
certs = "certs"
playbooks = "playbooks"
recordings = "recordings"
recipes = "recipes"
state = "state"                          # gitignored
scripts = "scripts"
templates = "templates"
data = "data"

[defaults.ssh]
user = "root"
port = 22
connect_timeout_s = 15
known_hosts = "workspace"                # "workspace" | "system"
concurrency = 8

[defaults.script]
runtime = "js"                           # "js" | "python" | "node"
timeout_s = 600
permissions = "ask"                      # "ask" | "deny" | "allow-listed"

[secrets]
backend = "keychain"                     # only supported value on macOS
service = "app.ghosh.homelab"            # keychain service name

[mcp]
enabled = false
transport = ["stdio", "http"]
http_bind = "127.0.0.1:7431"
read_only = true                         # mutating tools are hidden unless false
require_approval = ["node.exec", "node.run", "cert.issue", "cert.sync", "recipe.apply", "profile.open_tab", "page.eval", "page.click", "page.type", "page.run", "script.run", "term.run"]

[[mcp.clients]]
name = "filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "./data"]

[ui]
theme = "dark"
terminal_font = "JetBrains Mono"
terminal_font_size = 13
```

## A2. nodes/<id>.toml

```toml
schema = 1
id = "gpu1"
host = "203.0.113.10"
port = 22
user = "ubuntu"
labels = ["gpu", "comfy", "prod"]
tunnel = "vless-eu"                      # tunnel id or "direct"
jump = ""                                # optional node id for ProxyJump

[auth]
method = "key"                           # "key" | "agent" | "password"
key_ref = "secret://keychain/ssh-gpu1"   # legacy v1; file-key amendment pending

[facts]
cache_ttl_s = 3600

[hooks]
reload_cert = "systemctl reload nginx"
```

## A3. profiles/<id>.toml (browser profile)

```toml
schema = 1
id = "work"
label = "Work"
color = "#3B82F6"                        # tab badge color
data_store = "3f2b8c1e-5a47-4d9e-9b6c-1e0a7d4c2f88"  # random UUID v4, created once, immutable
start_url = "https://example.com"
tunnel = "direct"                        # tunnel id or "direct"
kill_switch = true                       # block tab if tunnel is down

[user_agent]
preset = "safari-macos-17"               # or "custom"
custom = ""
platform = "MacIntel"
languages = ["en-US", "en"]
timezone = "Europe/Berlin"
locale = "en-US"

[viewport]
zoom = 1.0
width = 0                                # 0 = follow window

[policy]
allow_downloads = true
allow_popups = false
js_init = ["scripts/profile-init.js"]    # relative to workspace
```

The loader rejects a non-UUID `data_store`. Clearing a profile never rotates this
identifier: coordinate all tabs, stop active access, and use the native website
data removal API. Full deletion uses the identified data-store removal API after
closing its views; reopening uses the same UUID. Selective cookies/storage/cache
clears use supported native data-removal operations and preserve other profiles.
The exact calls and error/availability behavior require the pinned native backend
smoke test, including persistence and clear/reopen isolation.

## A4. tunnels/<id>.toml

```toml
schema = 1
id = "vless-eu"
kind = "vless"                           # "direct" | "http" | "socks5" | "vless"
engine = "xray"                          # "xray" | "sing-box"
uri_ref = "secret://keychain/vless-eu"   # the vless:// URI is stored in the Keychain
listen = "127.0.0.1:0"                   # 0 = pick a free port
health_check_url = "https://www.gstatic.com/generate_204"
health_interval_s = 15
restart_on_failure = true
```

## A5. certs/<id>.toml

```toml
schema = 1
id = "wildcard-example"
domains = ["example.com", "*.example.com"]
provider = "cloudflare"
method = "certbot"                       # "certbot" | "acme-native"
email = "ops@example.com"
token_ref = "secret://keychain/cf-token"
renew_before_days = 30
sync_to = ["labels:prod", "node:gpu1"]   # selectors
remote_path = "/etc/ssl/ghosh/example.com"
file_mode = "0640"
owner = "root:root"
reload = "node-hook"                     # use [hooks].reload_cert of each node
```

## A6. recipes/<id>.toml (ComfyUI)

```toml
schema = 1
id = "flux-basic"
install_dir = "~/ComfyUI"
python = "3.11"
venv = "~/ComfyUI/venv"

[comfy]
repo = "https://github.com/comfyanonymous/ComfyUI"
commit = "pin-me"

[[custom_nodes]]
repo = "https://github.com/ltdrdata/ComfyUI-Manager"
commit = "pin-me"
requirements = true

[[models]]
url = "https://huggingface.co/<org>/<repo>/resolve/main/model.safetensors"
sha256 = "<hex>"
dest = "models/checkpoints/model.safetensors"
auth = "secret://keychain/hf-token"      # optional

[preflight]
min_free_gb = 50

[hooks]
post = ["systemctl --user restart comfyui"]
```

## A7. playbooks/\*.yaml (declarative, idempotent)

```yaml
schema: 1
name: base-setup
targets: ["labels:prod"]
vars:
  pkg: [curl, git]
tasks:
  - name: Install packages
    apt: { name: "{{ pkg }}", state: present }
  - name: Write config
    file:
      path: /etc/myapp/app.conf
      content: "{{ file('templates/app.conf') }}"
      mode: "0644"
    notify: restart_myapp
handlers:
  restart_myapp:
    systemd: { name: myapp, state: restarted }
```

Every task result is one of `ok | changed | skipped | failed`, with a unified diff when changed.

## A8. recordings/<id>.json

```json
{
  "schema": 1,
  "id": "login-and-export",
  "profile": "work",
  "steps": [
    { "t": "goto", "url": "https://example.com/login" },
    {
      "t": "type",
      "sel": ["[data-testid=email]", "#email"],
      "text": "{{secret:email}}"
    },
    { "t": "click", "sel": ["button[type=submit]"] },
    { "t": "waitFor", "sel": ["#dashboard"], "timeout_ms": 10000 },
    {
      "t": "extract",
      "sel": ["table.rows tr"],
      "fields": { "name": "td:nth-child(1)", "qty": "td:nth-child(2)" },
      "as": "rows"
    },
    {
      "t": "saveData",
      "path": "data/export.csv",
      "from": "rows",
      "format": "csv"
    }
  ]
}
```


## A7 refinement: task execution contract

Every task has exactly one action key from `shell`, `file`, `template`, `apt`,
`brew`, `pip`, `npm`, `systemd`, `git`, `download`, `docker`, `line_in_file`.
Common fields: `name` required; `check`, `changed_when`, `creates`, `unless`,
`retries`, `timeout_s`, `notify` optional. `retries` is extra attempts (default 0),
`timeout_s` defaults to 600 per attempt. Run-wide deadline bounds all retries.

Built-ins supply read-only checks and change determination. Arbitrary shell tasks
require an explicit read-only `check`; no implicit always-run shell action.
Check exit codes: 0=in sync; 10=drift; any other code=failed probe, never apply.
`creates` existing or `unless` exiting 0 skips. `unless` exit 1 permits checking;
other exits are failures. When both guards exist either skip condition wins.
`changed_when` is a restricted expression over exit code/stdout/stderr/previous
facts, not arbitrary JS or a second mutating command. The engine verifies drift
again before apply, then post-checks. Failed verification reports failed with any
partial change recorded, not a successful changed status.

```yaml
schema: 1
name: example-service
targets: ["node:gpu1"]
tasks:
  - name: Enable service
    check: { command: "systemctl is-enabled example; rc=$?; if [ $rc -eq 0 ]; then exit 0; elif [ $rc -eq 1 ]; then exit 10; else exit $rc; fi" }
    shell: { command: "systemctl enable example" }
    changed_when: "exit_code == 0"
    retries: 1
    timeout_s: 30
    notify: restart_example
handlers:
  restart_example:
    systemd: { name: example, state: restarted }
```

Package actions accept `name: string | string[]`, `state: present | absent | latest`,
and optional pinned `version`; update-cache is explicit. `latest` resolves the
planned version before approval and rechecks it before apply; do not silently
change an approved target version. YAML shell actions use `command`; the scripting
API uses `cmd`. Bindings normalize both to the same core command field. `file` accepts path, content,
mode, owner and state (file/directory/absent); content and templates use atomic
same-filesystem rename. `template` adds src and vars; secret material is ephemeral
and excluded from diffs. `git` accepts repo/dest/commit; dirty trees fail unless an
explicit reviewed reset policy exists. `download` accepts url/dest/sha256/auth;
verify partial content, Range/ETag behavior and full digest before rename.
`systemd` accepts name/state/enabled/daemon_reload; unsupported remote platforms
produce structured errors. `docker` compares declared image digest/config before
change. `line_in_file` accepts path/regexp/line/state and an atomic rewrite.

Handlers are deduplicated per node in declaration order after successful changes;
failed nodes do not run pending handlers. Generic restart handlers run only after
notification; they need no fictitious drift probe. Retrying a mutation requires an
idempotent adapter or a successful re-check. Dry-run reports predicted changes
with `check=true`, performs no mutation, creates no remote temporary files, and
marks unavailable diffs explicitly. `RunReport` counters match terminal task
statuses; cancellation is a run lifecycle state with unfinished tasks skipped
and reason=cancelled. No dry-run task may claim an observed apply.

## A8 refinement: complete recording step variants

| t | Required data | Optional data |
| --- | --- | --- |
| goto | url | wait_until=domcontentloaded |
| click | sel: string[] | button=left, modifiers=[] |
| type | sel, text OR value_ref | replace=true; never both text and value_ref |
| key | key | sel, modifiers=[] |
| scroll | x, y | sel; viewport if absent |
| waitFor | sel | timeout_ms=10000, state=visible/attached |
| wait | duration_ms | bounded by operation timeout |
| assert | assertion object | message |
| extract | sel, fields, as | limit=1000 |
| saveData | path, from, format | sqlite table defaults to data |

Assertions are typed: URL equality, selector attached/visible, or text comparison;
no arbitrary executable expression in a declarative assertion. Selector fallbacks
run in order and fail on ambiguity rather than choosing an arbitrary target.
`sel` is normalized to an array on load. Text interpolation supports explicit
`{{vars.name}}` and `{{secret:id}}`; the latter resolves only with a permission and
is never recorded as plaintext. Password fields and configured sensitive inputs
record value_ref prompts by default. Arbitrary non-password PII cannot be
infallibly detected: recordings/exports are local sensitive data with review.
Recorder supports app-controlled pages; inaccessible cross-origin frames/native
pickers are marked manual steps, never silently recorded as success.

## Additive v1 configuration

Appendix B1 of [the build prompt](ghosh-prompt.md) is the approval authority.
At MCP startup, an explicit `require_approval` must cover every unconditional
Approval=yes tool plus `node.run`; otherwise startup fails with missing tool names.
Omitted configuration uses the complete built-in default. Extra approvals may
strengthen policy. `node.run` skips tool approval only for `check = true`, checked
on every call; execution permissions still apply. See [MCP policy](MCP_SURFACE.md).

- `[mcp]`: add token_ref (Keychain), allowed_origins=[], allow_remote=false,
  request_timeout_s=600, max_output_bytes=262144, requests_per_minute=60,
  max_inflight=8. Remote mode is reserved and rejected in initial releases;
  future explicit enablement must require a separately reviewed secure deployment.
- `[[mcp.clients]]`: exactly one of command/url; args=[], cwd relative; HTTP uses
  headers_ref only, never persisted plaintext headers. Commands/args must not
  contain credentials. Default subprocess environment is a declared minimal set.
- `[permissions]`: default=ask; workspace opened untrusted initially; declarations
  live in `scripts/<name>.permissions.toml` (schema=1, nodes/profiles/tunnels/paths/
  secret_refs/mcp_clients arrays, local_process/network booleans default false).
  Stored grants live outside the package, keyed by workspace identity, canonical
  path, script hash and capability scope. Imported manifests cannot grant access.
- `[scheduler]`: cert_renew_enabled=false, poll_interval_s=3600; missed schedules
  are evaluated once on reopen, without overlapping runs. Recording schedules
  declare recording_id, cron, timezone and grant scope; app-running only.
- `[downloads]`: max_parallel=4; shared_model_dir optional remote absolute path.
- `[cert]` per certificate: verification endpoints use `[[verify]]` entries with
  node, host, port=443, server_name; optional expected_fingerprint selection.
  Deployment does not conflate SSH host and TLS hostname. TLS probes use the node's
  route and CA/hostname verification, then compare leaf SHA-256 and notAfter.
- `[recipe]` per recipe: optional shared_model_dir; model entries may add symlink
  destinations confined to the declared model root. No cross-device atomicity claim.

## Persistence, watch and state

Load into a validated typed projection backed by lossless TOML documents. Save
only edited keys, compare the original content hash, write/fsync a same-directory
temporary file, rename and fsync directory where supported. A single app-owned
writer/lock per canonical workspace serializes mutations; stale locks require
owner checks. Autosave debounces 500 ms; file watching debounces 250 ms. External
edits are validated before adoption; dirty conflict opens a comparison UI rather
than silently overwriting. Multi-file changes use a journal and recovery, not an
untrue all-files-atomic claim. Invalid edits preserve the last valid model, disable
new affected operations and show file/field diagnostics. Running operations keep
a revision snapshot; changes invalidate pending approvals and plans.

`state/runs/<run_id>/{report.json,events.jsonl}` stores bounded redacted results;
`state/facts/<node>.json` caches timestamped facts; `state/audit.jsonl` records
redacted decisions; `state/idempotency/` contains non-secret checksums only.
`state/ui.json` restores tab descriptors, never terminal input or secrets. The
application log has bounded rotation; retention defaults to 30 days, 1 GiB total.
Cancellation stops scheduling and requests remote stop; disconnected operations
are reported outcome_unknown until a subsequent probe determines state.

Machine-readable schemas are generated from Rust contracts during implementation,
with fixtures and drift tests. This document is the v1 specification; none are
claimed to exist today.
