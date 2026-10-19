# API contracts

Status: workspace/node/key persistence, operations/approvals and local/SSH PTY implemented.
Scenario execution and later services are future.
Normative target: [ghosh-prompt.md](ghosh-prompt.md), Appendices A/B.

## Workspace persistence
- Package: <name>.ghosh/ghosh.toml; schema=1 or 2, canonical workspace ULID, UTC creation time.
- Manifest/name edits preserve unknown keys/comments; directory name stays unchanged.
- Newer manifest schema opens inert/read-only; newer entity schema fails validation.
- Entity IDs match flat TOML filenames; references/defaults/jump cycles are validated.
- Profile data_store must parse as UUID; distinct per profile and immutable while loaded.
- Generation/clear/native profile activation belongs to M3; clearing must preserve UUID.
- Config paths reject traversal, absolute paths, overlaps and all symlinks.
- Leases bind canonical path and workspace ID across cooperating local processes.
- Revision is a SHA-256 configuration snapshot, not an arbitrary client-provided version.
- It covers manifest and flat nodes/profiles/tunnels/certs/recipes TOML, not browser data/scripts.
- Bounds: 1 MiB/document, 1,024 entity files, 16 MiB aggregate revision input.
- Rename requires LocalUi, matching workspace ID/revision and unchanged on-disk configuration.
- Temporary-file sync + revision recheck + atomic replacement + directory sync commit one file.
- Creation stages a complete package then publishes by atomic no-replace rename.
- No recovery journal/distributed transaction; crashed hidden stages remain inert.
- Hash validation cannot stop an arbitrary editor racing the final check/rename interval.
- External edits invalidate execution reviews and block save; basic local approval remains.
- Schema 1/2 local opens default approved; manual pause survives reload, reopen approves.
- Newer schemas remain unapproved/read-only; opening executes nothing.
- Reload rejects a replaced root or changed established profile store UUID.
- Full future profile/cert/recipe fields are preserved but not activated/fully validated.

## Desktop IPC (implemented)
Commands are restricted to the bundled main shell; debug additionally permits
http://127.0.0.1:1420. Capabilities do not grant remote child views these commands.
All return a typed result or AppError; filesystem operations run off the UI thread.

| Command | Input | Result |
| --- | --- | --- |
| workspace_create | request: {parent, name} | WorkspaceSnapshot |
| workspace_open | path | WorkspaceSnapshot |
| workspace_startup | none | Active snapshot, queued explicit open, last-open restore, or null (in that order) |
| workspace_close | request: {workspace_id, revision} | null; rejects stale context/running jobs, clears last-open and releases watcher/leases/PTYs |
| workspace_forget_recent | path: string or null | Updated recents; null clears all, never deletes package files |
| workspace_snapshot | none | WorkspaceSnapshot or null |
| workspace_rename | request: {workspace_id, revision, name} | WorkspaceSnapshot |
| workspace_reload | request: {workspace_id, revision} | WorkspaceSnapshot |
| workspace_trust | request: {workspace_id, revision, trusted} | WorkspaceSnapshot |
| workspace_pick | kind: workspace or parent | Native selected path or null |
| workspace_recent | none | Up to ten {name, path} entries |
| workspace_pending | none | Next native document path or null |
| operations_snapshot | none | OperationsSnapshot: jobs + approvals |
| jobs_cancel | id | JobsSnapshot |
| approvals_decide | request: {id, decision} | ApprovalsSnapshot |

History: atomic app-local workspaces.json, ten-entry MRU and independent nullable last.
Migrate legacy recent.json only when new state is absent. Close clears last, retaining MRU.
Remove/clear clears matching last; persistence failures preserve the in-memory history.

- Snapshot: id/name/path/revision/schema/read_only/trusted/external_changed/nodes/notices.
- NodeSummary: id/host/port/user/labels; no authentication or secret values.
- Node drafts/details, key metadata and identity selection use the dedicated commands below.
- workspace-files-changed event: {workspace_id, revision} identifies the loaded snapshot.
- This event is an invalidation hint, not an authoritative new revision; fetch snapshot/reload.
- Watcher debounces native events; bounded two-second probes cover missed notifications.
- workspace-open-requested has no document payload; drain workspace_pending after subscribing.
- Native queue exists before service setup, holds ten paths FIFO and performs no loading.
- Recent history failure adds a notice, without failing an otherwise successful open/save.
- Errors: {code, message}; static safe messages, snake_case codes from contracts.rs.
- Codes: invalid_document, invalid_path, missing_reference, unsupported_schema, conflict,
  busy, capacity, io, denied, cancelled, secret_unavailable, key_unavailable, internal,
  connection_failed, host_key_changed, authentication_failed, unsupported_connection.
- Generated source: src/generated/contracts.ts; exporter supports --check for freshness.
- JSON Schemas are structural projections; runtime adds policy/cross-file validation.

## Trust and secrets (implemented boundary)
- Local approval defaults on for supported schemas; pause survives reload and resets on reopen.
- External changes retain that setting but block writes/effects and invalidate execution reviews.
- Only LocalUi edits configuration; trust alone cannot authorize remote execution.
- Trusted local UI may request the separate direct-authentication review/test below.
- SecretRef parses secret://keychain/<id>, with bounded alphanumeric/underscore/hyphen ID.
- SecretValue zeroizes its bytes and implements neither Debug nor Serialize.
- SecretStore resolve/put/delete takes an explicitly authorized SecretAccess.
- SecretAccess requires NativeService, trusted=true and valid workspace identity.
- macOS adapter scopes service to app.ghosh.workspace.<workspace_id>, account to reference ID.
- Opening does not construct SecretAccess or read Keychain; there is no secret IPC command.
- Mandatory approval validation rejects explicit lists missing any B1 Approval=yes tool.
- node.run check=true bypasses only tool approval, evaluated per invocation.
- Native UI reviews exact-bound grants; actual server/execution integration arrives with services.

## Operation snapshots and grants (implemented)
- operations-changed is a payload-free invalidation hint; fetch operations_snapshot.
- Jobs/approvals have independent increasing sequence numbers; clients discard older snapshots.
- Subscribe before initial fetch, refetch on visibility and periodically to recover missed hints.
- Up to 128 jobs with 32 closed-vocabulary stages each; no command output or private paths.
- Workspace writes are non-cancellable once dispatched; late cancel never implies rollback.
- Up to 128 approval records and 256 memory-only audit entries; do not evict pending requests.
- Only NativeService creates/consumes bindings; only bundled LocalUi decides. No request/consume IPC.
- Bind workspace/revision/native UUID session/requester/tool/source SHA-256/sorted unique targets/route/identity.
- Recompute bindings immediately before effects; once grants cannot replay, session grants can be revoked.
- Five-minute monotonic expiry also bounds session grants; displayed wall time is informational.
- Fresh disk/trust checks precede decisions; configuration/trust changes invalidate live grants.
- Human waiting occurs outside workspace/node locks. No executor is connected yet.
- Command popup routes typed app actions without evaluating arbitrary shell input.

## File keys, nodes and migration (implemented)
- SSH file keys supersede Keychain-based SSH examples; SecretRef stays separate.
- `keys_list`, `keys_preview`, `keys_commit`, `keys_default`, `keys_forget`,
  `keys_pick`, `keys_generate`, `keys_relink` manage only explicit selected files.
- Registry revision and revalidated preview bind commit; never trust client metadata.
- Reference/managed storage; status available/missing/changed/unsafe. Public algorithm,
  fingerprint, encryption flag and selected path can cross IPC; private bytes cannot.
- `workspace_key`, `workspace_key_preview`, `workspace_key_commit`, `node_identity`
  expose effective public bindings and explicit schema-2 migration with manifest backup.
- Schema 2: `[ssh].default_key` on manifest, node `[auth] method="file"` with optional
  `key_id` (omitted means inherit). Schema-1 key_ref remains legacy/inert for transport.
- `nodes_list`, `nodes_preview`, `nodes_commit` perform revisioned CRUD/import;
  opening/listing/saving nodes never connects. See [SSH_KEY_STORAGE.md](SSH_KEY_STORAGE.md).
- Local registry IDs/public metadata only cross IPC; node > workspace > global
  defaults. No persisted SSH passphrases, no implicit key fallback or export.
- Command popup is an app action router; it cannot evaluate arbitrary shell text
  or bypass trust, target resolution, fresh plans and execution approvals.

## Direct SSH authentication and PTY (implemented)
- `node_connection_preview(request: {context: RevisionRequest, node_id})` returns
  NodeConnectionPreview {ticket, node_id, host, port, user, identity, host_key, credential_prompt}.
- `node_connection_test(request: {ticket, allow_unknown, credential})` returns
  NodeConnectionResult {node_id, host_key, elapsed_ms}; each is a local-only blocking job.
- Public identity includes source/fingerprint/status; host_key includes algorithm,
  SHA-256 fingerprint and known/unknown trust. Private key bytes never cross IPC.
- Direct file-key authentication supports encrypted keys with a per-connection passphrase.
  Node > workspace > global identity; no fallback for broken explicit references.
- Agent mode reviews the first supported public identity from an explicit absolute
  socket or SSH_AUTH_SOCK, then authenticates only that identity. No forwarding/fallback.
- Schema-2 password mode asks per connection; legacy password_ref is not resolved.
- credential is optional, bounded to 4096 bytes/no NUL, immediately moved to zeroizing
  native storage and never persisted/logged. UI clears its field on dispatch/completion.
- Native preview performs a bounded unauthenticated handshake; checks current-user
  and OrbStack known_hosts read-only, prefers existing matching host algorithms,
  and fails changed known keys. Unknown needs allow_unknown for this connection only.
  The local UI sets it automatically per user policy, with no confirmation dialog.
- Up to eight UUID tickets; single-use, monotonic 120-second expiry. Bind exact raw
  host key, endpoint, node, workspace/revision/native epoch and key-registry revision.
- Commit consumes first, reconnects, verifies the reviewed key and revalidates disk,
  trust and native context before resolving native-only Zeroizing key bytes.
- Reload, workspace mutation and trust changes rotate the native epoch and clear
  reviews; key resolution rechecks revision, fingerprint, safe mode/owner/file identity.
- Connection total timeout 5s, session I/O timeout 5s, bounded DNS workers/results.
  Authentication serializes active mutations; cancellation is cooperative at stage
  boundaries/DNS waits, not an interrupt of blocking socket/auth calls.
- Agent calls use at most two worker threads and a five-second caller deadline.
  A stuck local agent may occupy a worker until it returns; no channel is attached
  after cancellation/timeout. Password mode does not implement keyboard-interactive.
- No jumps, SSH-config/ProxyCommand, persistent host enrollment, exec or SFTP yet.

## Local and managed SSH PTY (implemented)
- `terminal_open_local(dimensions)` returns TerminalSession {id, target, shell, cwd}.
- `terminal_open_node({connection: NodeConnectionCommit, dimensions})` consumes
  the same fresh SSH review/authentication flow and attaches a real remote shell.
- Core Pty port is injected into ghosh-term; ghosh-ssh owns the nonblocking channel.
  PTY uses xterm-256color; resize is coalesced while preserving in-flight arguments.
- `terminal_poll(id, ack)` returns TerminalPacket {sequence, bytes, exited, exit_code, failed}.
- `terminal_input(id, bytes)`, `terminal_resize(id, dimensions)` and `terminal_close(id)`
  operate only on native-owned session IDs; bundled local caller checks apply to all six.
- TerminalSize {rows, cols}: rows 2..500, cols 2..1000; at most eight sessions.
- Fixed login shell/home cwd, TERM=xterm-256color/COLORTERM=truecolor; no workspace hooks.
- Output: 16 KiB packets, bounded queue of eight, replay until exact acknowledgement.
  Renderer acknowledges only after parsing; backpressure reaches the real PTY.
- Native input: four queued 16 KiB chunks; frontend bounded to 64 KiB and retries capacity.
- Exit/failure is explicit; close is idempotent and reaps the owned child/process group.
- Close tab/workspace/window guards terminate local/SSH sessions; quit cleans native state.
- No transcript persisted; shell/remote history is independent. Terminal data is sensitive
  user input/output and intentionally crosses this IPC boundary.
- Manual system SSH works inside this shell; it does not consume the Ghosh key registry
  or exact-bound native execution grants. Managed SSH uses the reviewed service above.
- Keyboard-only coverage/focus and executed native terminal tests are acceptance
  gates in [BASE44_DESIGN_REQUEST.md](BASE44_DESIGN_REQUEST.md).

## Future contracts — documented, not executable
- TaskResult: ok | changed | skipped | failed; diff/message, probes before apply.
- Cancellation/outcome_unknown, node write leases, deadlines and bounded outputs are required.
- Full schemas/defaults: [WORKSPACE_SCHEMA.md](WORKSPACE_SCHEMA.md).
- Script API/runtime bridge: [SCRIPTING_API.md](SCRIPTING_API.md).
- WebviewProfileBackend/TunnelBackend/Pty ports will hide native implementations.
- Multiwebview types remain inside ghosh-webview; unsupported features are explicit.
- MCP transports/tools/resources/prompts and approval binding: [MCP_SURFACE.md](MCP_SURFACE.md).
- MCP read_only hides mutators and rejects bypass calls; agents cannot self-approve.
- All future UI/script/CLI/MCP adapters must reuse the Rust service/policy boundary.

## UI and platform additions
- Select is a button combobox with a top-layer listbox; no native HTML select popup.
- Keyboard: open with Return/Space/arrows, highlight via arrows/Home/End/typeahead,
  Return commits, Escape cancels without dismissing its parent dialog, Tab advances.
- Cmd+[ / Cmd+] cycle tabs with wrap; Cmd+Shift+W closes the workspace with draft/session guards.
- Cmd+1–9 selects displayed app tabs; absent positions do nothing. Cmd+W retains
  dirty/session guards; Cmd+Q retains ordinary native behavior.
- Node select-all applies to visible rows and preserves hidden selections; mixed
  state is exposed. Neutral dots mean configured/unmonitored, not live online status.
- Linux legacy SecretStore resolve/put/delete return SecretUnavailable; no fallback.
- Release tag version must match package.json, Cargo workspace and Tauri config.
  Workflow starts only for pushed v* tags; macOS job precedes Linux job.

## Automation authoring (implemented; execution pending)
- AutomationKind: playbook | ansible | shell | python | deno.
- AutomationDocument {id, kind, source}; max 256 KiB UTF-8, safe entity ID, no NUL.
- automations_list(RevisionRequest) -> AutomationSummary[]; automation_read({context,id,kind}) -> document.
- automation_preview({context,document,replace,remove}) -> {digest,validation:{valid,name,task_count,issues}}.
- automation_commit({mutation,digest}) -> WorkspaceSnapshot; rechecks source/path/disk/revision and caller.
- Local UI only; paths derive from manifest folders and safe IDs, never caller-supplied write paths.
- Save preserves source bytes; remove uses the existing recoverable .removed convention.
- Flat .yml/.yaml, .ansible.yml, .sh/.py/.ts/.js participate in revision hashing.
- YAML parser errors remain static; invalid external sources are inert and editable, not executable.
- Native schema 1: name/tasks/check.command/shell.command; optional targets, timeout_s,
  retries, creates, unless, changed_when and named handlers/notify. Unknown fields fail.
- Ansible schema 1: name, absolute project, relative playbook and optional inventory;
  validation does not access the linked project. No script execution/interpreter checks yet.
