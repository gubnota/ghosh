# Implementation Plan: Ghosh reference UI and working native functionality

Date: 2026-10-18. Status: **Approved by the user's “go”; implementation in progress.**
Execution: sequential, one bounded increment at a time; no parallel agents/builds.

Progress: shell/keyboard, jobs/approvals, revisioned node CRUD, file-key library and
schema-2 identity migration implemented. Phase 5 real local PTY/emulator is built;
62 Rust and 14 browser tests pass (two opt-in Rust tests skipped by default). Native local, Debian OrbStack and node3 manual
SSH TTY smoke pass, with screenshots in README. Split sizing and keyboard session
switching pass after a native layout fix. Phase 4 encrypted unlock/consumer display,
phase 6 managed PTY/host enrollment/SFTP/facts remain open. Direct managed SSH
authentication now passes native integration and keyboard UI tests on both hosts,
with reviewed file identities and strict existing host-key verification. Native vi edit/save/exit passes.
Native shell 200% evidence exists; minimum OS and broader layout gates remain.
The baseline below is historical; current implementation is mapped in ARCHITECTURE_INDEX.md.

## Goal

Reproduce the navigation, terminal-like Nightfly appearance and useful interactions
of [the Base44 reference](https://ghosh-herd-os.base44.app/) in the existing native
Ghosh app, then connect every shipped action to real local/remote functionality.
Prioritize a complete keyboard-only workflow: open workspace → choose file-based
SSH identity → add/test node → real local/node terminal → create/import/edit a
YAML scenario → check/plan → review/approve/apply → inspect results/history.
Complete the browser/routes, certificates, scripts/recordings, ComfyUI and MCP
surfaces afterward. Preserve current workspace safety and portability contracts.

## Baseline before this increment

- Only M1.1 is implemented: Tauri desktop, React UI, core workspace model, nine
  workspace IPC commands, lossless TOML rename, revision/conflict checks, native
  pickers/open queue, recents, trust and read-only node inventory.
- Cargo currently has two members (`ghosh-core`, desktop); the operational crates
  in architecture docs are future modules. Tauri resolves to 2.12.4 in Cargo.lock;
  the desktop manifest currently enables no Tauri features. Frontend is React 19,
  TypeScript/Vite 7. Keep lockfiles authoritative and review dependency additions.
- `App.tsx` currently owns shell, dialogs, drafts and workspace coordination.
  `WorkspaceView.tsx` owns overview/editing. No command registry, tab workspace,
  SSH runtime, terminal emulator, playbook engine, browser or MCP runtime exists.
- Core SecretRef currently parses only `secret://keychain/<id>`; native Keychain
  adapter exists for that contract. SSH file keys need a separate typed contract,
  registry and explicit legacy migration; inventing a file URI is insufficient.
- Existing policy validates mandatory tool approvals, but no execution server or
  human approval service exists. Existing trusted=true is not an execution grant.
- The live reference is a UI prototype. Its terminal, Check, tunnel creation,
  renewal, recorder and recipe Apply use placeholders. Plan Apply does not apply,
  and opening history `r-1042` displays `r-1043`. Many settings are just headings.
  Copy their intended journeys; correct these failures in the native product.
- Evidence: [interaction inventory](base44/reference-app/INTERACTION_INVENTORY.md),
  [33 screenshots](base44/reference-app/README.md). Current native screenshots
  remain separate in [the earlier handoff](base44/README.md).

## Proposed approach

### Product fidelity and shared UI

Use the reference's top command bar, workspace switcher, grouped left navigation,
document/session tab strip, dense panels/tables, bottom status and focused forms.
Destinations: Overview, Nodes, Playbooks, Runs, SSH Keys, Browser Profiles, Tunnels,
Certificates, Scripts, Recordings, ComfyUI Recipes, MCP, Approvals, Settings.
Retain welcome/create/open/recents and first-use key→node→playbook→run guidance.

Create reusable buttons, labeled icon controls, form fields, status badges, table
selection, empty/error/loading panels, tabs, modal host, diff/log views and command
popup. Prefer ordinary HTML controls and the existing native `<dialog>` approach.
No inaccessible custom grid unless real grid keyboard behavior is implemented.
Tabs use stable workspace/entity/session IDs, not sample titles or array indices.
Persist harmless layout preferences; never persist trusted status or approvals.
Differentiate a document tab from a live terminal/browser session when closing.

Nightfly Default tokens: background `#011627`, foreground `#bdc1c6`, bright text
`#d6deeb`, border `#1d3b53`, blue `#82aaff`, green `#a1cd5e`, yellow `#e7d37a`,
red `#fc514e`, magenta `#c792ea`, cyan `#7fdbca`; selection `#b2ceee` on `#080808`.
Use [the saved palette](base44/reference/nightfly-default.toml) for complete ANSI
colors and [the user's theme reference](https://terminalcolors.com/themes/nightfly/default/).
Use SF Mono/Menlo/Monaco/Consolas fallback without network fonts; bundle JetBrains
Mono only with license review if needed. UI 14px (no core text below 13px), editors
and terminals 14px, headings 20–24px, normal tracking, weights 400/500/600. Avoid the
reference's tiny low-contrast labels; test contrast and 125–200% zoom. No fake
macOS title buttons inside content, Base44 badge or decorative success fixtures.

Keyboard behavior is foundational, not a final polish phase:

| Context | Required behavior |
| --- | --- |
| Global | Cmd+K / Cmd+Shift+P commands; Cmd+O open; Cmd+N add node when applicable; Cmd+Shift+E navigation; Cmd+S save current document; Cmd+F surface search |
| Tab workspace | Ctrl+Tab / Ctrl+Shift+Tab switch; Cmd+W close current app tab with dirty/live-session guard, then window behavior only when appropriate |
| Commands | Search labels/aliases and typed syntax; arrows choose, Enter opens form/review, Esc backs out/closes; no shell eval or implicit execution |
| Modals | Accessible title, useful initial focus, trapped focus/inert background, Tab/Shift+Tab, field errors, safe Cancel default, return to invoker |
| Tables/menus | Keyboard selection, Space toggles, range selection where supported, Shift+F10/Menu actions; all row operations have named keyboard targets |
| Editors/terminal/webviews | Context-aware dispatcher; preserve typing, terminal Ctrl+C/Ctrl+D, editor shortcuts and page input; explicit way back to shell navigation |
| Preferences | Shortcut search/remap, duplicate/reserved-chord checks; visible focus, reduced motion; no hover-only action |

One action registry backs menu, popup, shortcut and ordinary button paths. Register
all 24 observed commands, plus missing entity CRUD/actions as their service ships.
Parser returns typed action+arguments; unknown/incomplete syntax stays in popup
with explanation. Example `terminal open edge1` preselects the target in a form;
`run apply install-nginx --target edge1,edge2` opens a fresh plan/review. Enter alone
never bypasses an approval. Unavailable actions show a concrete reason.

### Native ownership, storage and operation contracts

Keep React as presentation/drafts; Tauri as composition/validated IPC; portable
policy, execution models and ports in core; platform services in owning crates.
Use existing caller validation, generated DTOs, safe errors and off-UI-thread work.
Remote browser views receive no native shell IPC. UI, scripts, CLI and MCP reuse
the same Rust services and policy, never independent execution paths.

Before adding each service, define its typed inputs/results and export contracts.
Proposed semantic operations below are **new contracts**, not existing commands:

| Owner | Operations / data passed across UI boundary |
| --- | --- |
| Workspace/entity storage | list/get/create/update/remove/import-preview, expected revision, entity ID; explicit conflict result; contained native-selected files |
| Key registry | list/register-selected/copy/generate/set-default/relink/remove-reference; public metadata/key IDs only; native secure entry for memory-only unlock |
| SSH | test/probe/facts/transfer and strict host-enrollment requests; bounded outcomes, timestamps, route/identity metadata; no raw auth |
| Terminal | open(target,shell,size,label), input(session,bytes), resize, ack, close; output(sequence,bytes), exit/error; session IDs scoped to owning shell/workspace |
| Playbooks/runs | parse/validate/save/import, resolve-targets, check/plan, apply(plan digest,approval), cancel, snapshot/history/export; structured task/node results |
| Jobs/approvals | list/snapshot/subscribe/cancel; inspect/allow-once/session/deny/revoke with bound requester, revision, targets, route and content digest |
| Browser/routes | profile CRUD/tab lifecycle/navigation/store clear/capabilities; route config/health/start/stop/restart/consumer review; safe handles only |

Use request IDs, deadlines and cancellation for long work; replace the current
single global busy flag with per-operation/entity state. Do not hold node/workspace
locks while waiting for human approval. Service snapshots are authoritative;
sequenced events are progress hints, support resubscribe/snapshot recovery.
Persist redacted run/audit history, with bounded live logs and explicit retention.
Do not persist terminal input/output by default. Distinguish cancelled, failed,
disconnected and outcome_unknown rather than claiming successful rollback.

Extend configuration safely as operational files become active. Playbook/script/
recording content hashes must bind execution approvals even though the current
workspace revision covers only TOML configuration. Preserve existing revision
semantics or introduce an explicit versioned revision contract; do not quietly
change what an old client authorizes. Entity writes use the anchored storage and
conflict pattern. Multi-file mutations need staged recovery or an explicit partial
failure contract; do not pretend one atomic rename updates a whole package.

SSH keys follow [SSH_KEY_STORAGE.md](SSH_KEY_STORAGE.md): current-user `~/.ssh`
files usable globally; custom reference/copy and generation; local registry outside
packages; native-only private bytes; directories 0700 / files 0600 for managed storage;
passphrases memory-only and no Keychain use for SSH. Node override > workspace
default > global default, unresolved explicit identity fails. Agent is a distinct
explicit mode. No recursive credential scan, automatic `ssh_config` execution,
private-key export, overwritten source keys or hidden authentication fallback.
Reference removal does not delete the original. Managed deletion shows consumers.
Portability means explicit relinking on a different machine, not embedded keys.

### Real terminals and scenario execution

Use xterm.js as renderer with a native Rust local PTY backend (portable-pty) and
SSH PTY channels from ghosh-ssh. Select and lock compatible versions at the first
terminal increment. It is an emulator bound to real shells, with search, multiple
tabs/splits, resize, UTF-8/ANSI/alternate screen, bracketed paste and full-screen
programs. Use bounded transport with explicit consumed-byte acknowledgements and
high/low watermarks; renderer callbacks alone do not bound native queues. See
[xterm's flow-control guide](https://xtermjs.org/docs/guides/flowcontrol/).
Reserve an application-owned escape/focus chord; never steal terminal Ctrl+C.
Show local vs node/user/host/key/route identity, not just a label. Route failure
disconnects visibly and cannot reconnect Direct. Destructive multiline paste gets
review. Explain that the real shell may save its own history independently of
Ghosh's no-history setting. Handle child exit, split cleanup and app shutdown.

Scenarios are the existing A7 declarative YAML format, not arbitrary Ansible file
compatibility. Library/new/import/templates must lead to a real editable source
and builder. Support the documented action set: shell, file, template, apt, brew,
pip, npm, systemd, git, download, docker, line_in_file. Validate typed variables,
selectors and OS support; the sample `integer default:auto` must fail validation.
Builder supports task add/edit/duplicate/delete/reorder, guards/timeouts/retries,
handlers/notify and target selection. Unsupported source constructs remain intact
in source mode; never silently discard them on a builder round trip.

Run journey: validate local source → resolve/explain targets → authorize vetted
read-only probes → check/diff/plan → review exact changes → approve → revalidate
revision/content/identity/routes → apply with per-node leases → post-check/handlers
→ durable report. Check cannot mutate or create remote temp files; arbitrary
shell text is not automatically a safe probe. Unknown diff stays unknown. Apply
uses A7 guard/probe exit-code and retry contracts. Cancellation reports partial
change or unknown outcome. Retry failed nodes creates a new linked run and fresh
plan, never reruns the historical run invisibly. An idempotency test must prove a
second apply has zero changes, not merely successful exit codes.

### Complete feature coverage

Each row includes CRUD/forms, validation, keyboard paths, permissions, honest
loading/error/empty states and persistence where applicable. Details in the
[full design request](BASE44_DESIGN_REQUEST.md) and REF inventory remain in scope.

| Feature | Required functional completion |
| --- | --- |
| Workspace/Overview | Existing behavior retained; real metrics/activity/recents, switcher, pending-open/dirty guards, trust reset and conflict recovery; no startup execution |
| Nodes | Create/edit/import-preview/remove, filter/route/selection/context menu, test, facts freshness, selected-node plan, SFTP listing/upload/download with scoped paths and overwrite/progress/cancel review |
| Keys | System/custom library, fingerprints/locked/missing states, use/default overrides, secure memory unlock, reference/copy/generate/relink/removal and key consumers |
| Playbooks/Runs | Authoring/templates/import, diagnostics, variables/handlers/targets, check/plan/diff/approval/apply/cancel, correct task/node counters, history/search/retry/export |
| Browser | Profile create/edit, immutable validated UUID v4 stores, UA/capabilities, mixed profile tabs, address/back/forward/reload/loading/error, open/close/restart persistence, scoped clear confirmation |
| Tunnels | Direct/HTTP/SOCKS5/VLESS CRUD/import, validated secrets references, native supervised xray lifecycle/readiness/retry/backoff/stop, consumer list and route-switch review, redacted logs, no silent Direct fallback |
| Certificates | Domain/provider/renewal policy editor, metadata-only token references, issue/renew via DNS-01 staging first, challenge cleanup, app-running schedule, staged cert/key distribution, post-install reload/TLS/SNI/fingerprint verification and partial-failure recovery |
| Scripts | Actual offline editor/library/save/import, runtime availability, source-hash capabilities review, explicit scoped human session grants, run/stop/output/errors; trusted-process disclosure, not a sandbox claim |
| Recordings | Create/record/pause/stop with real steps, edit/reorder/selectors, masking and secret references, variables/profile/route/timing replay review, per-step progress/failure, reviewed JSON/CSV export |
| ComfyUI | Recipe create/import/edit, pinned commits/models/custom nodes, OS/GPU/disk preflight, correct aggregate size, resumable bounded downloads/hash verification, shared model mapping, plan/apply/cancel/verify/export, idempotent reapply |
| MCP | Native disabled-by-default lifecycle, stdio attachment + authenticated loopback HTTP, read-only filter and dispatch enforcement, normative dotted B1 tools/resources/prompts, external client configuration/tests, no unsupported-tool stubs |
| Approvals/Jobs | Exact requester/arguments/targets/route/diff/digest, allow once/session/deny/timeout, revocable scoped grants, workspace-close invalidation, redacted audit; active job progress/cancel/details beyond runs alone |
| Settings | Real appearance/zoom/fonts/shortcuts, workspace defaults, SSH trust/keys, other secrets, runtimes/sidecars, terminal, native browser capability report, notifications, retention/privacy and actual versions/licenses/redacted diagnostics |

## File changes

Paths below are concrete ownership targets. Create each only with its owning
increment; do not scaffold empty modules. Tests live with the behavior they verify.
Later private helpers are chosen within these files when implementation warrants
them. No runtime files are changed by this planning task; no deletions proposed.

| File / module entry | Action and concrete responsibility |
| --- | --- |
| `src/styles.css` | Modify: Nightfly shared tokens, dense responsive layout, monospace, focus/zoom/reduced motion; retain accessible modal states |
| `src/app/App.tsx` | Modify: compose shell/providers; preserve native-open/draft/conflict lifecycle, move reusable presentation into components |
| `src/app/commands.ts`, `src/app/CommandPopup.tsx` | Create: typed action catalog/parser/context availability, searchable command popup and keyboard dispatch |
| `src/app/AppShell.tsx`, `src/app/tabs.ts` | Create: grouped navigation, workspace switcher, stable document/session tabs and guards, job/approval/status entry points |
| `src/components/Dialog.tsx`, `src/components/DataTable.tsx` | Create: extract proven modal focus behavior, named controls and keyboard selection/context actions |
| `src/features/workspace/WorkspaceView.tsx` | Modify: real overview/first-use presentation through existing workspace contracts |
| `src/features/keys/KeysView.tsx`, `src/features/nodes/NodesView.tsx` | Create: file-key library/forms/defaults; editable inventory, details/facts/transfers, target/session entry points |
| `src/features/terminal/TerminalView.tsx` | Create: xterm lifecycle, per-session tabs/splits/search/resize/flow-control/close state |
| `src/features/playbooks/PlaybooksView.tsx`, `src/features/playbooks/PlaybookEditor.tsx` | Create: library/new/import/templates; builder/source/variables/handlers/targets/diagnostics |
| `src/features/runs/RunsView.tsx`, `src/features/runs/RunView.tsx` | Create: real history/filter/retry/export; plan/diff/apply and live node/task results/logs |
| `src/features/approvals/ApprovalsView.tsx`, `src/features/jobs/JobsView.tsx` | Create: native request/grant/audit presentation, active jobs progress/cancel/details |
| `src/features/browser/BrowserView.tsx`, `src/features/tunnels/TunnelsView.tsx` | Create: profile/library/native tab chrome, route configuration and consumer review/health |
| `src/features/certificates/CertificatesView.tsx` | Create: certificate CRUD/renewal/scheduler/distribution review and verification |
| `src/features/scripts/ScriptsView.tsx`, `src/features/recordings/RecordingsView.tsx` | Create: editor/runtime/grants/output and recorder/editor/replay/export journeys |
| `src/features/recipes/RecipesView.tsx`, `src/features/settings/SettingsView.tsx` | Create: recipe preflight/download/plan/verify; functional settings categories, diagnostics/licenses |
| `src/features/mcp/McpView.tsx` | Create at MCP phase: server lifecycle/transport/read-only configuration, external client editing/testing and shared approval links |
| `src/app/ipc.ts` | Modify incrementally: typed commands/subscriptions, scoped streams and normalized safe errors |
| `crates/ghosh-core/src/contracts.rs`, `policy.rs` | Modify: export operation/status/approval contracts, scoped grants and execution checks; preserve strict B1 validation |
| `crates/ghosh-core/src/workspace/model.rs`, `document.rs`, `storage.rs`, `mod.rs` | Modify: explicit SSH file-key bindings/migration, entity writes/import, content-digest contracts and conflict-safe anchored persistence |
| `crates/ghosh-core/src/operations.rs`, `approvals.rs` | Create: jobs/event sequencing/cancellation and bound grant/decision/audit services |
| `crates/ghosh-core/src/playbook.rs`, `engine.rs`, `adapters/mod.rs` | Create: A7 parse/validation/targets, plan/check/apply/handlers/results and typed OS resource adapter dispatch |
| `crates/ghosh-core/src/lib.rs`, `examples/export_contracts.rs` | Modify: expose implemented ports/DTOs only and generate schemas/types |
| `crates/ghosh-core/tests/keys.rs`, `playbooks.rs`, `execution.rs`, `approvals.rs` | Create: migration/precedence, source round-trip/validation, check/idempotency/cancel/staleness, zero-effect denied operations |
| `crates/ghosh-ssh/Cargo.toml`, `src/lib.rs`, `src/keys.rs`, `tests/ssh.rs` | Create key registry with phase 4; add transport/tests with phase 6: strict host/auth/jump/pool/exec/SFTP/SSH PTY service and isolated sshd fixtures |
| `crates/ghosh-term/Cargo.toml`, `src/lib.rs`, `tests/pty.rs` | Create at terminal phase: local PTY lifecycle + injected SSH PTY, bounded streams/resize/cleanup tests |
| `crates/ghosh-tunnel/Cargo.toml`, `src/lib.rs`, `tests/routes.rs` | Create at routes phase: leases/generations and supervised transport readiness/failure tests |
| `crates/ghosh-webview/Cargo.toml`, `src/lib.rs` | Create at browser phase: WebviewProfileBackend, all multiwebview/platform store/proxy types and capability reporting |
| `crates/ghosh-certs/Cargo.toml`, `src/lib.rs` | Create at cert phase: issuance/cleanup/scheduler/deployment and verification orchestration |
| `crates/ghosh-script/Cargo.toml`, `src/lib.rs`; `crates/ghosh-recorder/Cargo.toml`, `src/lib.rs` | Create at script/record phase: runtime/grant boundary and real browser recording/replay |
| `crates/ghosh-comfy/Cargo.toml`, `src/lib.rs` | Create at recipes phase: pinned recipe/preflight/download/verification orchestration via shared ports |
| `crates/ghosh-mcp/Cargo.toml`, `src/lib.rs`; `crates/ghosh-cli/Cargo.toml`, `src/main.rs` | Create at MCP phase: protocol/client adapters and running-app attachment; no duplicate business engine |
| `src-tauri/src/lib.rs`, `services.rs`, `commands.rs` | Modify per phase: native composition, scoped service lifetimes/caller checks, off-thread IPC and stream dispatch |
| `src-tauri/Cargo.toml`, capabilities/ and permissions/ | Modify only with required service: exact scoped IPC rights; never grant remote webviews shell/native access |
| `Cargo.toml`, `Cargo.lock`, `package.json`, `package-lock.json` | Modify per increment: actual member/dependency additions, locked versions and meaningful test scripts; no bulk speculative installation |
| `src/generated/contracts.ts`, `schemas/` | Regenerate from exporter as contracts activate; never hand-edit |
| `tests/ui/`, `tests/native/` | Create focused harness with native test composition, deterministic non-production fixtures, keyboard journeys and native smoke evidence; runner choice checked on this Tauri/macOS setup |
| `docs/API_CONTRACTS.md`, `WORKSPACE_SCHEMA.md`, `SSH_KEY_STORAGE.md`, `MILESTONES.md`, six context files | Update with accepted schemas/sequence and actual implementation/test evidence per increment; keep proposed vs implemented explicit |

## Implementation steps

Each numbered item is a sequential delivery gate. Keep code changes small by
separating contracts/storage, service behavior and connected UI; tests accompany
their behavior. Do not commit or push unless requested. Stop a phase at a failed
gate, fix it and record evidence before starting its dependent phase.

The following file scopes bind the steps to the ownership table above. Each phase
also updates its generated contracts, native composition/IPC permissions, focused
tests and compact docs when those contracts change.

| Phase | Primary file scope |
| --- | --- |
| 1 | This plan, reference inventory, API_CONTRACTS.md, WORKSPACE_SCHEMA.md, SSH_KEY_STORAGE.md, DECISIONS.md |
| 2 | styles.css; app/App.tsx, AppShell.tsx, tabs.ts, commands.ts, CommandPopup.tsx; components/Dialog.tsx, DataTable.tsx; workspace/WorkspaceView.tsx; tests/ui/ |
| 3 | core operations.rs, approvals.rs, policy.rs and workspace storage; jobs/JobsView.tsx, approvals/ApprovalsView.tsx; core tests/approvals.rs |
| 4 | core workspace model/document/storage and tests/keys.rs; ghosh-ssh/src/keys.rs and crate entry; keys/KeysView.tsx, nodes/NodesView.tsx |
| 5 | ghosh-term crate and tests/pty.rs; terminal/TerminalView.tsx; tests/native/ |
| 6 | ghosh-ssh transport and tests/ssh.rs; ghosh-term SSH port; nodes/NodesView.tsx; SSH/terminal native fixtures |
| 7 | core playbook.rs, engine.rs, adapters/mod.rs and tests/playbooks.rs, execution.rs; playbooks/PlaybooksView.tsx, PlaybookEditor.tsx |
| 8 | runs/RunsView.tsx, RunView.tsx; core execution/history contracts; approvals UI; tests/ui/ and native run fixtures |
| 9 | ghosh-tunnel and ghosh-webview crates; browser/BrowserView.tsx, tunnels/TunnelsView.tsx; native browser/route fixtures |
| 10 | ghosh-certs crate; certificates/CertificatesView.tsx; staging/deployment integration fixtures |
| 11 | ghosh-script and ghosh-recorder crates; scripts/ScriptsView.tsx, recordings/RecordingsView.tsx; native capture/runtime fixtures |
| 12 | ghosh-comfy crate; recipes/RecipesView.tsx; recipe/download integration fixtures |
| 13 | ghosh-mcp and ghosh-cli crates; mcp/McpView.tsx; policy/approval and protocol fixtures |
| 14 | settings/SettingsView.tsx; affected service preferences/diagnostics; tests/ui/, tests/native/ and release evidence |

1. **Reference baseline and contract decisions.** Review REF-01–32 plus the full
   screen brief; freeze the visual/interaction inventory. Record narrow decisions
   for key registry/binding migration, operation/event envelopes, source hashes and
   native test harness. Preserve user `.gitignore` edits and ignored source prompt.
   Acceptance: approved plan, explicit old/new schema compatibility and fixtures;
   no unresolved private-key IPC or automatic legacy export behavior.
2. **Nightfly shell and keyboard foundation.** Extract existing dialogs, add shared
   shell/components/tab state/command catalog and shortcut contexts. Connect real
   workspace/open/save/trust/conflict behavior first; unsupported sections state
   availability honestly. Implement navigation/overview/settings appearance and
   command forms. Acceptance: existing workspace flows pass; mouse-free create/
   open/rename/save/switch/conflict recovery and modal focus verified at narrow and
   wide sizes. No sample metrics presented as real.
3. **Operational storage, jobs and approval core.** Add typed entity mutation/import,
   operation snapshots/sequences, cancellation, bounded logs, content digests,
   native approval queue/grants/audit. Build connected Jobs/Approvals surfaces.
   Acceptance: stale revision/digest/target requests fail; denial/timeout causes
   zero business effects; UI resubscription recovers current state; no lock held
   while waiting for approval. Grants expire on revision/workspace/session changes.
4. **File-key library and node management.** Implement explicit schema migration,
   native registry/metadata, reference/copy/generation/defaults/relink/memory unlock;
   node CRUD/import and effective-identity preview. Existing Keychain-only refs
   remain legacy until explicit relinking; unrelated credentials retain their own
   provider. Acceptance: no SSH Keychain calls, safe perms/no overwrite, absent key
   error and precedence tests, keyboard-only key→node creation with no connection.
5. **First real local terminal.** Add ghosh-term local PTY and xterm renderer/session
   stream, then search/tabs/splits/resize/close guard. Do this before larger remote
   orchestration so the user's local-machine test is a real product milestone.
   Acceptance: execute the local terminal journey below **inside Ghosh**; record
   screen evidence, buffer limits and child cleanup. A host-shell test is insufficient.
6. **Strict SSH and remote node workflows.** Add ghosh-ssh transport, known_hosts
   enrollment/mismatch checks, key/agent/memory-password auth, verified jump hops,
   identity/route-scoped pooling, bounded exec/facts/SFTP and SSH PTY. Connect node
   Test/Facts/Files/Terminal. Start with Direct, explicit supported HTTP/SOCKS route
   ports; unavailable VLESS fails closed until phase 9. Acceptance: isolated sshd
   fixture for auth/host mismatch/jumps/timeouts/disconnect and remote PTY; node
   identity always visible, no hidden key/agent/Direct fallback.
7. **Playbook authoring and real engine.** Implement A7 schema/source editor/templates/
   import/builder and typed variables/handlers/targets, then probes/plans/adapters/
   leases/apply/post-check. Deliver adapter groups with focused fixtures, covering
   all documented actions before declaring M1 complete. No new `symlink` action
   solely to match a prototype label: model it through reviewed supported schema.
   Acceptance: malformed/unsupported YAML diagnostics, lossless source handling,
   safe check/no remote temp files, stale plans rejected, second apply=no changes,
   correct handlers/retries/OS errors/cancellation partial outcomes.
8. **Run review/history and first complete remote scenario.** Connect task×node diff,
   plan options, native approval, apply, live matrix/logs/counters, history/filter/
   retry-failed/export. Replace any interim unavailable action only when service
   works. Acceptance: keyboard-only create/import→target→check→plan→approve→apply
   on fixture nodes; select any historical run and see that exact ID/outcome;
   failed-node retry is a new linked approved run. This closes the first usable slice.
9. **Routes and native browser feasibility, then UI.** Spike exact locked Tauri/Wry/
   WebKit initialization for custom UUID store+proxy+two child views in one window
   on macOS 14 and a later OS. Enable `unstable` only inside the bounded browser
   integration; isolate all multiwebview code in ghosh-webview. Record API names,
   availability, thread/lifetime requirements and upgrade breakage recovery plan.
   Then implement tunnel supervision/leases/route review, profile CRUD and mixed
   real tabs. Acceptance: persistence and bidirectional same-origin cookie isolation,
   clear only selected store **without UUID rotation**, route separation, visible
   down-route blocking, modal overlays/focus and no native IPC in remote content.
   If selected APIs fail, first keep/revert to last verified locked versions; choose
   a reviewed native adapter only behind WebviewProfileBackend. Do not weaken
   isolation or silently replace the requirement with a shared webview.
10. **Certificates.** Implement DNS-01 staging issuance and credential cleanup, renewal
    policy/app-running scheduling, distribution plan/stage/atomic pair activation/
    reload/TLS verification and recovery. Acceptance: ACME staging fixtures,
    interrupted/partial deployment and SNI/fingerprint checks; app closed means
    no renewal unless a separately reviewed background service is later added.
11. **Scripts and recordings.** Validate embedded runtime packaging/termination first;
    implement editor/runtime/grant/run/output, then real WebKit capture/step editor/
    masked variables/replay/exports. Use shared services for every bound API call.
    Acceptance: source edits invalidate grants, unauthorized calls fail before
    effects, limits/cancel work, private fields absent from recording/log/export,
    real page capture/replay with selector drift and failure reports. Never equate
    capability review with an automatically granted Run button.
12. **ComfyUI recipes.** Implement recipe authoring/pinned export, remote preflight,
    shared paths, verified/resumable downloads, plan/apply/verify/cancel and correct
    totals. Acceptance: interrupted download resumes safely, checksum/auth/space
    failures are explicit, unresolved pins prevent export, second apply=no changes.
13. **MCP server/client and CLI attachment.** Expose supported B1 tools/resources/
    prompts through shared services, add real start/stop/configuration, authenticated
    loopback/stdio attachment and external client editor. Validate complete required
    approval list at startup; strict rejection of omissions. Evaluate node.run safe
    `check=true` exception per invocation, never just by tool-list membership.
    Acceptance: B3/Inspector conformance, read-only direct-call bypass rejected,
    every required browser tool invokes human review, revoked/denied requests have
    no business effects; prototype aliases cannot replace normative dotted names.
14. **Complete settings and release parity.** Connect every settings category to its
    owning service, diagnostic export preview/redaction/licenses, retention, shortcut
    remapping and runtime/sidecar capability detection. Finish empty/loading/error/
    permission/conflict states for every REF row. Acceptance: signed/native packaging
    evidence where distributed, complete mouse-free walkthrough, actual local and
    SSH terminal evidence, no simulated success anywhere, known limits documented.

This refines delivery order without relabeling M1.1 as complete M1: shell/keyboard
move forward, and the local part of M2 comes before remote engine completion.
M3–M7 retain their module gates; Linux/Windows remain M8.

## Verification

Run relevant checks sequentially after each behavior change: formatting, Clippy,
core/native tests, generated-contract freshness, frontend type/build and targeted
UI/service integration. Use the existing README commands; keep build jobs/test
threads bounded. Add UI test tooling only with the first meaningful interactive
tests. Frontend fakes verify UI states, never substitute for native PTY/SSH/WebKit.
Do not label an unavailable minimum-OS runner or skipped fixture test as passing.

| Gate | Required evidence |
| --- | --- |
| Existing safety | Open remains inert, real recents/native-open queue, read-only schema, draft/close guards, external conflict/reload/trust invalidation |
| Visual | Native screenshots welcome/overview/table/editor/diff/terminal/modal at 734×975 and 1391×988 or comparable sizes; 125/150/200% zoom, visible focus and reduced motion; compare reference structure with documented readability improvements |
| Keyboard | Mouse-free first-use journey plus all 24 commands, row actions, builder task operations, tabs/splits, browser address/route, approvals, settings; focus restore, no editor/terminal shortcut theft |
| Keys/SSH | Fresh fixture keys only, restrictive perms, source preservation, missing/encrypted refs/default precedence/relink, strict host changes/jump checks, no private IPC/logs/exports or implicit fallback |
| Engine | Check zero mutation, probe failure!=no-change, typed variables/targets, drift/post-check, idempotency, handlers, retries/deadlines, stale approval, cancellation/unknown outcome |
| Event/history | Correct run/entity/session IDs, monotonic progress/recovery, bounded logs/streams, counters match task results, restart/partial run recovery and reviewed exports |
| Browser/routes | Two profiles/two views/one window, same-origin isolation both directions/restart/clear, route down/switch/restart, minimum OS and later OS, overlay/focus/resize/DPI; report DNS/WebRTC limitations |
| Remaining services | ACME staging/deploy recovery; actual script runtime/grants; real capture/replay; verified recipe downloads/no-op; MCP B3/denial/read-only protocol tests |

Local terminal manual acceptance, **inside the built Ghosh app on this machine**:

1. Use Cmd+K → `terminal open`; review target/shell and open an actual session.
2. Run `pwd`, `printf 'Ghosh PTY ✓\n'` and `stty size`; verify real current directory,
   Unicode output and PTY dimensions, then resize/split and verify updated sizes.
3. Run a full-screen editor such as `vi`, edit a disposable file, exit; verify
   alternate-screen restoration, cursor, keyboard input and selection/copy/paste.
4. Exercise a long-running output producer with bounded buffers; Ctrl+C must
   interrupt promptly while UI remains usable. Test Ctrl+D/exit status, bracketed
   paste/multiline review, search and focus movement without mouse.
5. Open two independent local sessions and a node session; verify input never
   reaches the wrong session, close guards work and no orphaned child remains.
   Record OS/build/version and actual screenshots/results, not a simulated toast.

Separate real SSH terminal acceptance uses an isolated sshd fixture or explicitly
authorized test node: enroll host, authenticate selected key, allocate remote PTY,
verify remote identity/resize/full-screen/signals, then disconnect/reconnect and
route loss. Do not enable macOS Remote Login or alter the user's original
`/Users/vm/Documents/1.ghosh` as an implicit testing prerequisite.

## Risks / edge cases

- Prototype source was not supplied. This plan recreates observed UI/interaction
  semantics in native architecture, not the Base44 source implementation. Visible
  but unexercised actions are explicitly tracked in the inventory.
- Tauri multiwebview `unstable` APIs and WebKit store/proxy lifecycle must be proven
  against locked dependencies before UI promises. The public version-specific
  Tauri doc page was unavailable during this inspection; no new API signature is
  asserted here. Overlay/focus of native child views needs native tests.
- Key registry migration changes authentication contracts. Keep legacy refs explicit,
  never silently extract Keychain material or resolve imported absolute key paths.
- Terminal output can flood memory, keys can leak through logging, and real shells
  have their own history. Use bounded queues/native secret handling and accurate
  settings descriptions. Optional keyboard protocols are reported only if tested.
- Apply may partially succeed before disconnect/cancel; file transfers/cert pair
  activation have crash windows. Model partial/unknown outcomes and recovery;
  do not promise generic rollback for arbitrary remote commands.
- Systemd/apt and brew/macOS behavior differ. Platform support is adapter-specific;
  unsupported actions fail before changes rather than guessing replacements.
- Do not claim proxy leak prevention, browser fingerprint protection, OS scripting
  sandboxing, tamper-proof audit storage or unattended certificate renewal without
  the corresponding implementation and evidence.
- This is a multi-milestone product plan, not a one-patch estimate. Finish each
  usable slice and its tests before widening scope; no deadline estimate without
  the native feasibility results.

## Out of scope

- Runtime implementation, dependencies/config changes, commits, publishing or
  deployment in this planning task; no Base44 account/project edits or uploads.
- Copying Base44 branding/edit badge, fixture identities, broken sample behavior
  or claiming access to private source. No third-party site auth or execution.
- Root-owned/all-user SSH key management, SSH private keys/passphrases in Keychain,
  credential vault features, automatic ssh_config commands or secret backup.
- Full Ansible compatibility, automatic remote agent installation, automatic SSH
  server enablement, generic rollback and unrestricted shell command evaluation.
- Linux/Windows native feature parity before M8; background daemon/scheduler and
  strong network isolation/anti-detect claims without a separate reviewed scope.

## Bounded build exception (2026-10-18)
The approved UI/release continuation brings forward Linux x86_64 build packaging
from M8 only. Tag-only CI checks/builds macOS then Linux. Linux legacy Keychain
operations fail closed; no new plaintext secret store. Interactive Linux acceptance
and later managed services remain separate unfinished gates.
