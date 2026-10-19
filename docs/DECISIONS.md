# ADR-0001: Documentation-first delivery

Date: 2026-10-18
Status: Accepted

Context: The user requires compact persistent context and architecture before code.
Decision: Maintain six memory documents; deliver architecture, contracts, milestone
sequence and a Base44 screen request. Source work follows an approved concrete plan.
Consequences: Future modules remain labeled; memory files track implemented scope
and actual verification separately from target architecture documents.
Related files: `docs/PROJECT_CONTEXT.md`, `docs/IMPLEMENTATION_PLAN_M1.md`.

# ADR-0002: Shared portable core and platform adapters

Date: 2026-10-18
Status: Accepted

Context: UI, scripts and MCP must behave consistently across future OS backends.
Decision: Use one Rust facade and core-owned ports; concrete adapters depend inward.
Keep all multiwebview types in ghosh-webview behind WebviewProfileBackend.
Consequences: No adapter cycles or duplicate business logic; shell composes services.
Related files: `docs/ARCHITECTURE.md`, `docs/MODULE_OWNERSHIP.md`.

# ADR-0003: Immutable native profile identity and reviewed Tauri upgrades

Date: 2026-10-18
Status: Accepted

Context: macOS 14+ permits identified stores/per-store proxies; multiwebview API is unstable.
Decision: Create one UUID v4 per profile, reject non-UUID load and identity edits,
clear through native data-removal APIs. Enable unstable, pin Tauri in Cargo.lock,
review upgrades and rerun two-profile/one-window cookie-isolation smoke.
Consequences: Exact pinned API integration is a release gate. Native NSView hosting
behind the same trait is the fallback if Tauri child hosting breaks; it needs testing.
Related files: `docs/ghosh-prompt.md`, `docs/WORKSPACE_SCHEMA.md`, `docs/ARCHITECTURE.md`.

# ADR-0004: Mandatory MCP approval validation

Date: 2026-10-18
Status: Accepted

Context: User B1 is authoritative and permits either startup refusal or a warned repair.
Decision: Choose startup refusal when explicit require_approval misses mandatory
tools; report missing names. Defaults contain the full set. Check node.run's
check=true exception per invocation; retain separate execution permissions.
Consequences: M1.1 validates manifest approval lists and the per-call policy helper.
Server startup, reload activation and approval sheets remain M7 integration work.
Related files: `docs/MCP_SURFACE.md`, `docs/WORKSPACE_SCHEMA.md`.

# ADR-0005: Honest route and runtime security boundaries

Date: 2026-10-18
Status: Proposed

Context: Native proxy features do not prove DNS/WebRTC isolation; managed runtimes
can bypass API bindings through ordinary local process permissions.
Decision: No deliberate direct fallback; test route leakage before claiming strong
kill-switch. Treat Python/Node scripts as trusted local code. Recheck signed V8
packaging and choose QuickJS only through an explicit recorded decision.
Consequences: UI discloses limits; no anti-detect or untrusted-code sandbox claims.
Related files: `docs/ARCHITECTURE.md`, `docs/SCRIPTING_API.md`, `docs/MILESTONES.md`.

# ADR-0006: Small first implementation increment

Date: 2026-10-18
Status: Accepted

Context: Initial repository had no application code; full M1 includes SSH/task work.
Decision: Start with M1.1 workspace foundation using core+shell only, lossless TOML,
conflict-safe saves, typed IPC and Keychain boundary. Add later crates when needed.
Consequences: M1.1 is independently reviewable and does not claim M1 completion.
Related files: `docs/IMPLEMENTATION_PLAN_M1.md`, `docs/MILESTONES.md`.

# ADR-0007: Anchored local persistence and staged publication

Date: 2026-10-18
Status: Accepted

Context: M1.1 edits one manifest and creates complete directory packages. Arbitrary
editors and crashed processes must not silently overwrite another user's files.
Decision: Anchor filesystem access, reject symlinks, lease canonical path plus ID,
hash bounded configuration, sync/recheck/replace individual manifest writes, and
publish complete staged packages with no-replace rename. Use no multi-file journal
until multi-file mutations exist. Failed operations clean their own temporary data;
crashed hidden stages remain inert and are not removed automatically.
Consequences: No partial visible package from creation, but no automatic crash-stage
recovery or distributed locking. An unrelated editor can race the last check/rename.
Synced/network volume classification is deferred; the UI/docs advise local storage.
Related files: `crates/ghosh-core/src/workspace/storage.rs`, `README.md`.

# ADR-0008: Bounded native invalidation and early document queue

Date: 2026-10-18
Status: Accepted

Context: Native smoke exposed a missed startup filesystem event and macOS document
delivery before Tauri setup. Managed services do not exist at that delivery point.
Decision: Add a bounded two-second configuration revision probe to debounced native
notifications. Allocate a ten-entry FIFO document queue before builder/setup; events
only enqueue/notify, then the ready UI drains through normal validation and guards.
Consequences: External edits remain visible after missed events; cold Finder open
does not access uninitialized Tauri state. Queue tests and native cold/warm smoke pass.
Related files: `crates/ghosh-core/src/workspace/watch.rs`,
`src-tauri/src/services.rs`, `src-tauri/src/lib.rs`, `src/app/App.tsx`.

# ADR-0009: Defer unused execution and multiwebview machinery

Date: 2026-10-18
Status: Accepted

Context: M1.1 has bounded synchronous filesystem operations and one local shell;
there are no task runtimes, profile views or live execution services to supervise.
Decision: Use a shared atomic cancellation primitive and Tauri blocking workers now.
Defer tokio-util operation supervision/tracing integration and Tauri unstable until
their services exist; retain locked dependencies and the future ghosh-webview boundary.
Consequences: Cancellation is checked before commit; it is not rollback after commit.
M1.2+ must add deadlines/progress/structured supervision before remote execution.
Related files: `crates/ghosh-core/src/contracts.rs`, `src-tauri/src/commands.rs`,
`docs/IMPLEMENTATION_PLAN_M1.md`.

# ADR-0010: File-based SSH identities and keyboard-first terminal UI

Date: 2026-10-18
Status: Accepted

Context: User could not create/run playbooks, rejected typography and requested
ordinary SSH key files, Nightfly decoration, command modals, full keyboard control,
and real per-node terminals plus local-machine terminal testing.
Decision: Design readable monospace Nightfly UI around Cmd+K app commands and
focused forms. Make key/node/playbook/plan/run the primary journey. Use existing
user keys or custom referenced/managed files; no SSH Keychain/passphrase storage.
Require real local PTY and per-node SSH PTY with executed native acceptance tests.
Consequences: Supersedes older SSH Keychain design only. Current M1.1 parser/UI
remain unchanged; schema/registry/terminal source work needs a narrow reviewed plan.
A Base44 browser prototype simulates native services and cannot prove terminal tests.
Related files: `docs/BASE44_DESIGN_REQUEST.md`, `docs/SSH_KEY_STORAGE.md`,
`docs/base44/README.md`, `docs/MILESTONES.md`.

# ADR-0011: Approved sequential native UI delivery

Date: 2026-10-18
Status: Accepted

Context: The user approved IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md with “go”.
Decision: Deliver its dependency gates sequentially. Copy the observed interaction
inventory with real native services, retaining explicit unavailable states until
each service works. Use browser fixtures for interaction regressions and the actual
release app for WebKit, filesystem and PTY/SSH acceptance; keep their evidence separate.
Consequences: Approval covers the plan's implementation; no repeated approval gate.
The original user workspace is read-only test input; mutations use disposable fixtures.
Related files: `docs/IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md`, `tests/ui/shell.spec.ts`.

# ADR-0012: Execution identity, bounded jobs and transient grants

Date: 2026-10-18
Status: Accepted

Context: Workspace revision currently hashes bounded TOML configuration, excluding
future executable YAML/JS. Trust alone cannot authorize execution of those files.
Decision: Preserve revision v1; independently bind SHA-256 source/plan, sorted exact
targets, route and identity digests, requester, workspace and native session epoch
to approvals. Native services create requests; the local shell may decide/revoke,
but cannot create requests or consume grants. Use monotonic expiration, single-use
or exact-binding session grants; reset on workspace/revision/trust/session changes.
Jobs have bounded snapshots, monotonic change sequences and structured stages;
events invalidate snapshots and reconnect reloads them. Never log arbitrary input.
Consequences: No lock waits for a human decision. Execution services must revalidate
disk/source/target context immediately before effects; this core is not an executor.
Live jobs/grants/audit are memory-only; durable redacted run history belongs to phase 8.
Related files: `crates/ghosh-core/src/operations.rs`, `crates/ghosh-core/src/approvals.rs`.

# ADR-0013: Explicit file-key schema migration

Date: 2026-10-18
Status: Accepted

Context: v1 accepts legacy SSH Keychain SecretRef values; the user requires key files.
Decision: Introduce schema 2 only with phase 4's parser/registry and fixtures. v1
packages remain readable and inert; migration is an explicit preview/confirmation,
never a rewrite on open. v2 stores opaque local registry binding IDs, not private
material or imported absolute paths. Unresolved IDs require relinking. Legacy SSH
Keychain bindings remain unresolved until the user chooses a file; never export
Keychain material. Keep unrelated SecretRef providers unchanged. Migrate one file
atomically at a time with a recoverable backup; multi-file transactions require a
journal before activation. Schema-2 support is now enabled by the phase-4 implementation.
Consequences: No invented file URI in v1, no silent key access or auth fallback.
Phase 4 must prove compatibility/precedence/relinking with isolated key fixtures
before enabling new writes; those fixtures and explicit migration are now implemented.
Related files: `docs/SSH_KEY_STORAGE.md`, `docs/WORKSPACE_SCHEMA.md`.


# ADR-0014: Real local PTY with bounded acknowledged transport

Date: 2026-10-18
Status: Accepted

Context: The user requires an actual emulator, keyboard operation and local/remote
terminal tests; prototype output cannot establish terminal functionality.
Decision: ghosh-term owns portable-pty login shells, bounded input/output queues,
exact packet replay/acknowledgement, resize and process-group cleanup. React uses
xterm with fit/search; acknowledge after parsing. Review multiline paste and
confirm destructive session termination. Keep scrollback in memory, no transcript.
Consequences: Shell startup files, normal user permissions and shell history apply.
A local shell can run system SSH using user configuration, independently of future
managed node identity/host-verification/approval services. No sandbox claim. Native
local/Debian/node3 tests prove that terminal path only. macOS bundle declares local
network usage; minimum OS and managed remote acceptance remain separate gates.
Related files: `crates/ghosh-term/src/lib.rs`, `crates/ghosh-core/src/terminal.rs`,
`src/features/terminal/TerminalView.tsx`, `src-tauri/Info.plist`.


# ADR-0015: Single-use reviewed direct SSH authentication

Date: 2026-10-18
Status: Accepted

Context: The approved phase 6 needs a real managed connection path, distinct from
manual SSH inside the local terminal. Both authorized hosts already have trusted
ED25519 records; libssh2's default ECDSA preference must not create false mismatches.
Decision: Isolate ssh2/vendored-OpenSSL transport in ghosh-ssh. Load user and OrbStack
known_hosts read-only and prefer matching recorded algorithms before handshake.
Require trusted fresh workspace context, a two-minute single-use native ticket and
review of the endpoint, resolved file identity and server fingerprint. Reconnect
and compare the exact reviewed raw key before resolving/revalidating native private
bytes. Unknown servers require explicit one-test acceptance; mismatches fail closed.
Rotate the native session epoch on trust/reload/configuration changes to invalidate
both stored tickets and inflight previews. Do not run remote commands in this slice.
Consequences: Both real hosts pass native authentication and keyboard UI tests.
No system SSH-config/agent/Keychain integration; encrypted unlock, managed PTY,
jumps, persistent host enrollment, immediate in-flight cancellation and pooling
remain separate gates.
Related files: `crates/ghosh-ssh/src/lib.rs`, `src-tauri/src/services/ssh.rs`,
`src/features/nodes/ConnectionTest.tsx`, `docs/API_CONTRACTS.md`.

# ADR-0016: Shared themed controls and sequential tagged builds

Date: 2026-10-18
Status: Accepted

Context: User requested Base44 SVGs, keyboard controls, tall custom selectors,
small-layout containment and macOS arm64/Linux x86_64 artifacts only on tags.
Decision: Use one keyboard combobox/listbox with Nightfly styling, pinned Lucide
icons and an SVG Shift key. Keep native and web backgrounds dark and layout
children shrinkable. Add a Linux fail-closed legacy-secret adapter; SSH file keys
remain independent. Build native artifacts sequentially for version-matched v* tags.
Consequences: No native select popup or plaintext fallback. Linux foundation is
brought forward from M8; interactive Linux and minimum-macOS acceptance remain.
Related files: src/components/Select.tsx, src/styles.css,
src-tauri/src/platform/unavailable.rs, scripts/, .github/workflows/ci.yml.

# ADR-0017: Optional future remote Deno without a persistent agent

Date: 2026-10-18
Status: Accepted

Context: User asked about Deno on nodes and a precompiled server agent.
Decision: Retain SSH as the transport; no persistent Ghosh server is required.
Future JS/TS jobs may invoke a pinned Deno runtime with reviewed dependencies,
explicit noninteractive permissions, cancellation/deadlines and bounded output.
No silent runtime install or allow-all. Treat subprocess/FFI as escaping runtime
restrictions and control static import/dependency acquisition explicitly.
Consequences: Remote executor remains pending; local embedded deno_core is a
separate planned API. A short-lived compiled helper needs its own concrete use case.
Related files: README.md, docs/IMPLEMENTATION_PLAN_UI_RELEASE.md.

# ADR-0018: Operational workflow supplement and local approval defaults

Date: 2026-10-19
Status: Accepted

Context: User reports unusable SSH/scenario flows and requests existing Ansible
playbooks, default workspace approval, cross-workspace timer catch-up, live
monitoring, named ComfyUI models, browser sessions and future scene automation.
The approved plan excludes full Ansible compatibility and retains session trust.
Decision: Adopt default local workspace approval, operation-scoped authorization,
managed SSH PTYs/auth, a real controller-side Ansible Runner adapter, enabled
app-local schedule bindings and sequential usable workflow slices. Initial ComfyUI
families are MiniMax H3, Wan 2.2, LTX 2.3 and user-imported LoRAs. Keep Telegram at
integration-design scope. Existing runtime contracts remain until implementation.
Consequences: User approved this supplement with “go” on 2026-10-19; previous
approved work remains approved. No production deployment or original-project
modification is part of diagnosis. RunPod endpoint/paths and LoRA URLs are form
inputs, not prerequisites for the editor or fabricated example assets.
Related files: IMPLEMENTATION_PLAN_OPERATIONAL_WORKFLOWS.md,
IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md; future changes to approvals/workspace/SSH
contracts are enumerated in the supplement.


# ADR-0019: Shared terminal manager and ephemeral SSH credentials

Date: 2026-10-19
Status: Accepted (SSH confirmation superseded by ADR-0020)

Context: Nodes could only test unencrypted file keys while the editor offered agent
auth. Users need a real terminal after saving a node and an unlocked-key path.
Decision: Inject a core Pty port into the existing bounded sequenced terminal
manager. Local portable-pty and direct ssh2 PTYs share xterm, resize, replay and
cleanup. Reuse single-use, revision-bound host reviews for test/open. Native-only
file bytes may remain encrypted until transport authentication; transient IPC
passwords/passphrases are zeroized natively and never persisted. Review and pin
one agent public identity, with two bounded workers and a five-second deadline.
Consequences: Both authorized hosts pass managed PTY acceptance. Agent and encrypted
file keys pass node3; live password, keyboard-interactive, identity selection, jump
routes and persistent host enrollment remain separate acceptance gates. A stalled
agent worker can outlive its deadline but cannot attach a terminal afterward.
Related files: crates/ghosh-core/src/terminal.rs, crates/ghosh-core/src/ssh.rs,
crates/ghosh-ssh/src/lib.rs, crates/ghosh-term/src/lib.rs,
src-tauri/src/services/ssh.rs, src/features/nodes/ConnectionTest.tsx.

# ADR-0020: Immediate SSH and inert automation authoring

Date: 2026-10-19
Status: Accepted

Context: User explicitly requests no confirmation when connecting to a node.
Automation destinations lacked a way to create scripts or use examples.
Decision: Test/Terminal immediately obtain and consume a native pinned ticket.
Unknown hosts are allowed for that connection; changed known keys remain errors.
Only required credentials prompt. Keep bounded inert source authoring in core;
use typed revision/digest IPC, serde_yaml_ng and exact-source atomic persistence.
Consequences: No SSH enrollment checkbox; no newly observed host persistence yet.
Playbook/Script editors work before execution adapters. Validation never runs source
or opens a linked Ansible project. Script syntax/runtime checks remain pending.
Related files: src/features/nodes/ConnectionTest.tsx,
crates/ghosh-core/src/automation.rs, crates/ghosh-core/src/workspace/automation.rs,
src/features/automation, docs/FEATURE_STATUS.md.


# ADR-0021: Sequential native execution and durable bounded results

Date: 2026-10-19
Status: Accepted

Context: Authoring alone cannot operate nodes; the approved workflow requires local
and SSH execution, cancellation, idempotence and inspectable outcomes.
Decision: Share a portable check/apply/verify engine through an Executor port.
Native services resolve pinned SSH tickets, recheck exact source/revision before
invocation and persist bounded private snapshots plus metadata events. Local
process groups are reaped; SSH interruption is unknown, with no automatic replay.
Run authorizes manual local UI execution without a second SSH confirmation.
Consequences: Native scenarios and installed Shell/Python/Deno runtimes work.
Ansible remains disabled until its real controller adapter is implemented. Reports
are phase based, sensitive, local-only and lack retention cleanup. No agent daemon
or runtime install is needed; Deno permissions/dependencies remain deliberately explicit.
Related files: crates/ghosh-core/src/runs.rs, crates/ghosh-ssh/src/exec.rs,
src-tauri/src/services/runs.rs, src-tauri/src/services/local_exec.rs,
src/features/automation/RunDialog.tsx, src/features/automation/RunsView.tsx.

# ADR-0022: Temporarily suspend Linux CI

Date: 2026-10-19
Status: Accepted

Context: v0.1.2 macOS checks/build/artifact passed; Linux failed in scripts/check.sh
before packaging. User explicitly requested skipping Linux and commenting it out.
Decision: Comment out the complete Linux job; keep tag-only macOS arm64 CI enabled.
Consequences: No Linux artifacts are promised. Preserve the job and local packaging
support for later diagnosis and re-enablement; do not conceal the failed checks.
Related files: .github/workflows/ci.yml, README.md, docs/FEATURE_STATUS.md.


# ADR-0023: Reuse upstream Ansible and retain simple checked scripts

Date: 2026-10-19
Status: Accepted

Context: Existing server-first-run projects must retain normal Ansible YAML,
roles and inventory. The user also wants small scripts with success checks.
Decision: Standard play lists and project links execute through ansible-core
2.20.10 / ansible-runner 2.4.3 in an explicitly selected local Python environment.
Keep simple schema: 1 check/apply/verify scenarios as a separate lightweight option.
ComfyUI forms generate reviewable standard Ansible, with pinned Git commits and
SHA-256 model downloads. Reuse existing native authentication and exact server pins.
Consequences: No new Ansible dialect, persistent server daemon or runtime auto-install.
Invocation is sequential per selected node, not a full multi-host Ansible batch.
Linked inventory aliases must match node IDs; original external source is not frozen.
Private Runner files are temporary, no_log results are censored, bounded reports
can be exported as JSON. GPU-template acceptance and richer result bindings remain.
Supersedes ADR-0021’s temporary statement that Ansible is disabled.
Related files: requirements-controller.txt, src-tauri/src/services/ansible.rs,
src-tauri/src/services/ansible_bridge.py, src/features/comfy/playbook.ts,
src/features/automation/examples.ts.

# ADR-0024: Physical RU/EN shortcuts

Date: 2026-10-19
Status: Accepted

Context: The user clarified Keyboard layout means hotkeys work in RU and EN.
Decision: Normalize physical event.code for letters, digits and bracket shortcuts;
retain event.key fallback, composition/Alt guards and terminal Ctrl input ownership.
Consequences: No shortcut remapping UI is introduced. Existing Cmd shortcuts keep
working on a Russian layout; keyboard-only fixtures cover both views and terminals.
Related files: src/app/keyboard.ts, src/app/App.tsx, tests/ui/shell.spec.ts.
