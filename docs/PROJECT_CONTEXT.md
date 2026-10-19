# Ghosh project context

## Identity and status
- Project/repository: Ghosh / ghosh; cowherd in Bengali.
- Purpose: a desktop control plane for remote nodes and browser automation.
- Users: infrastructure/GPU operators, automation authors and authorized agents.
- License: MIT OR Apache-2.0; third-party icon notices are included.
- Current milestone: approved UI/functionality plan in progress; M1 remains incomplete.
- M1.1 workspace foundation, Nightfly shell and jobs/approval core are implemented.
- User approved M1.1 with “go, not in parallel”; execution was sequential.
- Baseline bbac06a contained only .gitignore; v0.1.0 introduces this foundation.
- Working app: create/open/inspect/rename/reload/trust local workspaces.
- File-key library, schema-2 identity migration and node CRUD are implemented.
- Real local PTY/emulator and manual SSH to both authorized targets pass native smoke.
- Managed direct SSH authentication and PTYs connect immediately; native exact-key checks remain.
- File keys, encrypted keys and SSH agent passed live tests; per-call password is implemented.
- Simple checked scenarios, Shell/Python/Deno and upstream Ansible run locally/over SSH.
- ComfyUI wizard generates standard Ansible for pinned plugins and checksum-verified models.
- Browser sessions, monitoring, durable schedules and MCP execution remain pending.

## Read first and navigate
- Six memory files summarize actual behavior; keep them short and current.
- [ARCHITECTURE_INDEX.md](ARCHITECTURE_INDEX.md) maps existing and future modules.
- [MODULE_OWNERSHIP.md](MODULE_OWNERSHIP.md) defines dependency boundaries.
- [API_CONTRACTS.md](API_CONTRACTS.md) summarizes implemented IPC and core contracts.
- [DECISIONS.md](DECISIONS.md) records choices and their consequences.
- [TASK_LOG.md](TASK_LOG.md) records real checks and remaining verification gaps.
- [IMPLEMENTATION_PLAN_M1.md](IMPLEMENTATION_PLAN_M1.md) records approved scope/outcome.
- [IMPLEMENTATION_PLAN_OPERATIONAL_WORKFLOWS.md](IMPLEMENTATION_PLAN_OPERATIONAL_WORKFLOWS.md)
  is approved: lifecycle/default approval, direct SSH PTYs and Ansible implemented; timers/monitoring and expanded workflows pending.
- Full target architecture: ARCHITECTURE.md; sequencing: MILESTONES.md.
- Target schemas/API: WORKSPACE_SCHEMA.md, SCRIPTING_API.md, MCP_SURFACE.md.
- User source: ghosh-prompt.md (user-owned, intentionally ignored by Git).
- BASE44_DESIGN_REQUEST.md describes future screens, not implemented features.

## Implemented stack and entry points
- Primary target: macOS 14+ Apple Silicon; Linux x86_64 build foundation added, acceptance pending; Windows deferred.
- Rust 1.96.0, edition 2024; toolchain and Cargo.lock checked in.
- Tauri 2.12.4 / tauri-build 2.7.3 pinned by Cargo.lock.
- React 19, TypeScript, Vite 7; package-lock.json is authoritative.
- Recommended frontend runtime: Node.js 24+; local build used Node 26.8.
- Rust workspace: crates/ghosh-core, crates/ghosh-term, crates/ghosh-ssh and src-tauri.
- SSH: ssh2 0.9.6/libssh2 with vendored OpenSSL; transport owns verification/auth only.
- Terminal renderer: xterm 6 with fit/search addons; native PTY: portable-pty 0.9.
- Desktop entry: src-tauri/src/main.rs and lib.rs; UI: src/main.tsx.
- Typed desktop API: src/app/ipc.ts; generated DTOs: src/generated/contracts.ts.
- Generate contracts/schemas from core; never hand-edit generated files.
- Local release bundle: target/aarch64-apple-darwin/release/bundle/macos/Ghosh.app.
- README.md contains setup, build, checks and limitations.

## Workspace behavior
- Workspace is a directory package named <name>.ghosh with ghosh.toml.
- Opening is inert: no scripts, hooks, network requests or Keychain reads.
- Schemas 1/2 validate canonical ULID identity, UTC creation time and safe paths.
- Schema 2 adds opaque file-key bindings; explicit migration retains manifest backup.
- Opening legacy schema 1 is inert and does not export Keychain identities.
- Flat node inventory validates IDs/auth/jump chains/tunnel references.
- Profile stores must parse as UUID, be distinct, and remain immutable while loaded.
- Future entity fields are preserved; full activation validation belongs to later services.
- Unknown TOML keys/comments survive display-name edits through toml_edit.
- Newer manifest schemas open read-only; newer entity schemas are rejected.
- Display-name changes do not rename the directory package.
- All configuration paths are contained, non-overlapping and reject symlinks.
- Canonical path and workspace-ID leases prevent cooperating duplicate writers.
- Revision covers manifest, flat entity TOML and flat Playbook/Script source files.
- Writes require matching workspace ID, revision, current disk state and LocalUi.
- Save syncs a temporary file, rechecks hashes, replaces atomically and syncs directory.
- Creation publishes a complete hidden staging directory using no-replace rename.
- Failed creation cleans its own stage; a crash can leave an inert hidden stage.
- There is no multi-file recovery journal or automatic crashed-stage cleanup.
- Native watcher debounces changes; bounded two-second revision probes cover missed events.
- External edits invalidate execution reviews and block save until validated reload; local approval remains.
- Failed reload preserves the last valid snapshot and the user's draft.
- Replaced/moved roots are rejected instead of writing through a stale directory handle.
- Bounds: 1 MiB/TOML, 1,024 config files, 16 MiB aggregate config.
- Prefer local disks; no distributed locking or automatic synced-volume classification.
- Arbitrary editors can still race the final hash check/rename; this is not filesystem CAS.

## UI and native shell
- English Nightfly monospace shell, navigation, stable tabs, command popup and settings.
- Physical-key shortcuts work in RU/EN: Cmd+K/Shift+P, 1–9, brackets, guarded W, search/save.
- Composition/Alt guards remain; terminal Ctrl shortcuts reach the real PTY.
- Shared themed selectors: taller rows, padded SVG arrows, keyboard/typeahead, viewport bounds.
- Lucide SVG controls, SVG Shift and aligned mixed-state node selection replace glyphs.
- Native background and shrinkable dark layout prevent white/uncontained page backgrounds.
- Node inventory supports search, add/edit/import/remove with preview/revision checks.
- SSH Keys supports explicit system/custom files, managed copies, generation and relink.
- Terminal has up to eight sessions, split/search, paste review and guarded termination.
- Shell runs with normal user permissions/startup/history; no transcript persistence.
- Jobs show actual workspace operations; approvals show native-bound requests/audit.
- Appearance preferences persist locally; workspace drafts survive tab switching.
- Unsaved edits require confirmation before switch, reload or window close.
- Native file dialogs choose parent folders and .ghosh document packages.
- Finder cold/warm document-open requests enter a bounded queue before services initialize.
- UI subscribes before draining; requests wait while a dialog/operation is active.
- workspaces.json stores ten recent names/paths and a separate last-open path; migrates recent.json.
- Startup restores last unless a native document-open request is queued. Close disables restoration.
- Recent search/remove/clear leave package files intact; Close releases watcher/leases/PTYs.
- History persistence failure is a notice and does not prevent workspace opening.
- One active workspace per process; filesystem work runs off the UI thread.
- Restricted command capabilities, caller/origin validation and CSP protect shell IPC.
- Remote child views have no privileged IPC grants; none exist in M1.1.

## Automation authoring
- Playbooks/Scripts catalogs support create, paste, examples, edit, validate, save and recoverable remove.
- Flat YAML/Ansible links and Shell/Python/Deno sources are revision-bound, at most 256 KiB each.
- Invalid external sources open inert with validation issues; they cannot be saved through the editor.
- Simple schema: 1 scenarios support check/shell/verify and bounded handlers, without Ansible.
- Standard Ansible YAML is interpreted by upstream ansible-core 2.20.10 / Runner 2.4.3.
- Controller uses an explicitly selected Python 3.12+ virtualenv, requirements-controller.txt.
- Linked projects retain inventory/group_vars/roles/includes/handlers in their original folder.
- Each selected node runs separately, with an exact host-key pin and private transient Runner data.
- Linked inventory alias must match node ID; full cross-host batch semantics are not implemented.
- External linked project contents are not frozen by the workspace revision.
- ComfyUI needs an existing installation, public URLs, Git commits and SHA-256 hashes.
- GPU acceptance, paid Pod provisioning, service restarts and model catalog are pending.
- Native Check/Apply/verify, handler/retry/skip rules, up to eight local/SSH targets and Stop work.
- Shell/Python/Deno execute on installed runtimes; no runtime/dependency installation.
- Runs stream local/SSH logs, show step progress, survive navigation and export bounded durable phase reports.
- Offline Monaco provides maximize, syntax/completion and local non-executing diagnostics; IDs have explicit rules.
- Keep the app open; no daemon or continuation after exit. Unknown outcomes never replay automatically.

## Trust, secrets and approval policy
- Local schema 1/2 workspaces default approved; manual Pause survives reload, reopen approves again.
- Newer schemas remain read-only/unapproved; approval alone never executes contents.
- Test/Terminal connect without SSH confirmation per user request; unknown keys allowed per connection.
- Native single-use handshake tickets still reject changed known keys and pin server identity.
- Review binds workspace/revision/native session/key-registry revision/exact server key; 120s TTL.
- Trust, reload and mutations rotate the native session and invalidate reviews/inflight context.
- Configuration edits are authorized only for the local human UI principal.
- Secret references use secret://keychain/<id>; raw values never reach frontend DTOs.
- SecretValue is zeroized and has no Debug/Serialize implementation.
- Core owns SecretStore; macOS shell implements it using security-framework.
- Linux legacy SecretStore fails closed; SSH file storage remains separate.
- Tag-only CI builds macOS arm64; Linux job temporarily commented out per user after v0.1.2 failure.
- Archives/checksums via scripts/build-release.sh; Linux local packaging remains available.
- Secret resolution requires an explicitly authorized trusted NativeService context.
- Error envelopes use static messages; parser/OS input is not echoed to the UI.
- Mandatory approvals include every B1 Approval=yes tool; incomplete explicit lists fail load.
- node.run check=true bypasses only its tool approval, checked per call.
- Native approval inbox supports once/session grants, denial/revocation and bounded audit.
- Exact bindings cover workspace/revision/native session, requester/tool/source/targets/route/identity.
- Five-minute monotonic expiry; disk edits, trust changes and workspace mutations invalidate.
- Only native services can request/consume agent grants; manual local UI runs use explicit Run.
- Jobs/grants/audit are memory-only; automation reports persist separately in app-local runs/.
- Remote/page/MCP output must always remain untrusted data, never instructions.

## Approved requirements and remaining services
- Nightfly Default dark theme; readable monospace UI, terminal-like density.
- Keyboard-only operation; global command popup (Cmd+K), focused action modals.
- Primary flow: key -> node -> YAML playbook -> targets -> plan/approve -> run.
- SSH keys: existing user files plus custom referenced/managed files; no Keychain
  for SSH keys/passphrases. Global, workspace and node assignment; see SSH_KEY_STORAGE.md.
- Existing Keychain-only SecretRef stays separate from implemented file-key registry.
- Registry is user-local ssh/registry.json; managed ssh/<uuid>.key files are 0600/0700.
- Effective file key: node > workspace > global; explicit missing key cannot fall back.
- Encrypted files retain encryption; per-connection unlock works without saving passphrases.
- Local PTY exists and passed local/native manual SSH tests on debian.orb.local
  and node3.dabaga.cc. Managed authentication also passed on both, including native UI.
- Direct file keys, SSH agent and schema-2 per-call passwords share reviewed authentication.
- Known hosts: user + OrbStack files, read-only; changed keys fail; unknown is per-connection opt-in.
- Managed SSH PTYs passed input/resize/Unicode/exit/cleanup on both authorized hosts.
- SSH exec and native scenarios pass both authorized hosts; persistent enrollment, jumps/SFTP remain pending.
- Base44 handoff: docs/base44/; ten real JPEG screenshots, brief, offline gallery and ZIP.
- Original workspace opened without edits/trust; fixture used for conflict/node captures.
- Live Base44 reference inspected via Browser; 33 captures and observed/placeholder
  interaction inventory are in docs/base44/reference-app/ (not native test evidence).
- IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md is approved: 14 sequential phases.
- Shell implemented; phase 3 jobs/approval infrastructure implemented, entity storage pending.
- Phase 4 key/node storage implemented; phase 5 local PTY implemented.
- Remaining gates: persistent host enrollment, jumps/SFTP/facts and later phases.

## Future platform gates
- SSH: strict known_hosts, scoped pooled channels, bounded operations and cancellation.
- Terminal: portable-pty/xterm plus injected managed SSH PTY implemented.
- Scripting: future embedded JS plus trusted bridges.
- Multiwebview belongs only in future ghosh-webview behind WebviewProfileBackend.
- Enable Tauri unstable only when multiwebview is introduced; review pinned upgrades.
- Native store/proxy API integration must be checked against that selected version.
- M3 gate: two profiles/two views/one window, cookie isolation and route tests on macOS 14.
- Store clearing must use native removal APIs, preserving the profile UUID.
- No anti-detect, untrusted-code sandbox or strong DNS/WebRTC isolation claim.

## Agent workflow and verification
- Read only these six files first; choose focused sources through the index.
- Approve a concrete plan before the next software increment; avoid future scaffolding.
- Keep small coherent changes; preserve user-owned prompt and .gitignore entries.
- User requested sequential work: no parallel agents or builds for this increment.
- Latest checks: formatting/Clippy, 88 Rust tests (3 opt-in skipped), 32 browser UI tests, contract freshness.
- Separate upstream controller integration and read-only Ansible over SSH passed on both authorized nodes.
- Arm64 release built; native local/Debian/node3 TTY tests pass, screenshots in README.
- Browser IPC fixtures verify presentation; they do not prove native execution.
- Optional native Keychain create/read/delete test passed outside the restricted sandbox.
- Native smoke passed: create/save/conflict/reload/trust/restart and cold/warm document open.
- Native Nightfly/keyboard/draft/conflict smoke passed at 1160×780 on macOS 27.
- Native 200% shell captured; minimum macOS 14 and broad terminal layout matrix pending.
- Tag-only macOS CI remains enabled; Linux job stays commented out. Signing/notarization are not configured.
- Update memory/contracts/task evidence whenever behavior or scope changes.
