# Implementation Plan: usable operational workflows

Date: 2026-10-19. Status: **Accepted; implementation in progress.**
Approved by the user with “go” on 2026-10-19.
This supplement changes specific policies and extends the approved
[UI/functionality plan](IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md).
That plan remains approved; its completed work is retained. Execute sequentially.
Current implementation is described by the six project memory documents.

## Goal

Make Ghosh useful for its actual workflows: add a node and open its terminal;
author and run scenarios, including the existing server-first-run Ansible project;
renew and distribute certificates; inspect live logs and resource use; prepare
RunPod ComfyUI environments; automate browser sessions and save their results.
Restore workspace state without repeated trust prompts. Provide a subsequent
keyboard-operated 3D scene workflow with animation and video export.

## Progress

- Slice 1: implemented and committed as 73000b3; lifecycle/history/default approval,
  clickable commands and bracket shortcuts pass regression checks.
- Slice 2, direct terminal journey: implemented. Test/open share reviewed file,
  agent and per-call password authentication; encrypted files unlock per call.
  Managed PTYs passed Debian OrbStack and node3; agent/encrypted-key PTYs passed
  node3. Live password, persistent host enrollment, agent identity choice,
  keyboard-interactive, jumps and route-loss managed checks remain. Native release
  split sizing/Unicode passed on both; Debian Ctrl+C and Vim save/exit passed.
- Slices 3–8: pending; no production playbook/certificate operation has run.

## Baseline behavior before this supplement

- Real local xterm/PTY, node editing, a file-key registry and limited reviewed SSH
  login tests work. Managed remote PTYs and execution services do not exist yet.
- New nodes default to agent authentication, but the login service accepts only
  direct, unencrypted file keys. This contradicts the editor and causes the
  reported unsupported-authentication message before a login can be attempted.
- Nodes have Test/Edit/Remove but no managed Open Terminal action. Test also
  requires session-only workspace trust, which disappears on every reopen.
- The command popup already has a click handler. Many entries are deliberately
  disabled placeholders; dispatch/focus and unavailable-state presentation both
  need verification. Changing click styling alone cannot implement those actions.
- Workspace open/create/recents exist; close, restore-last and cleanable history
  do not. Cmd+1–9, Cmd+W and Ctrl+Tab exist; Cmd+[ and Cmd+] do not.
- Browser Profiles is a configuration destination; there is no live Browsers
  session destination. Other requested service destinations remain placeholders.
- The native scenario format is a proposed custom DSL, not an Ansible runtime.
  The previous plan explicitly excluded full Ansible compatibility.
- Read-only inspection of ../server-first-run found seven Ansible entrypoints and
  the local certbot-cloudflare.sh controller script, listed below. Secret-bearing
  inventories, group variables, Cloudflare credentials and certificates were not
  imported into this repository.
- certbot-cloudflare.sh forces renewal of both lineages on every invocation and
  lacks fail-fast sequencing. certs_sync.yml copies the entire certbot/config tree;
  its Nginx notification/reload handler is commented out. It cannot currently
  provide the requested scheduled renewal → distribution → restart workflow.

## Proposed approach

### 1. Workspace lifecycle and approval defaults

Locally created or explicitly opened workspaces are approved by default, as
requested. Remove the repeated Trust prerequisite from Test/Terminal/Play.
Persist restore-last and recent names/paths in application-local storage, with
Close Workspace, New, Open, Recent, Remove Recent and Clear History commands.
Clearing history removes history records; it leaves workspace files intact.
Offer a separate schedule-management link when a removed recent has active timers.

Local workspace approval is separate from agent/page permissions, SSH server-key
verification and operation-specific authorization. Remembering an enrolled server
key avoids repeated host-trust dialogs; a changed server key remains an error.
Manual local UI execution opens its parameter/review form, then Run authorizes
that exact operation. Configured timers retain a durable, explicitly enabled
scope instead of asking on every run. Existing remote-agent approval rules remain.

Opening a workspace does not execute its contents. Startup catch-up runs only
previously enabled schedules from the application-local schedule registry.
Source/configuration/target changes invalidate affected execution plans or timer
bindings; display a useful repair/review action without revoking basic workspace
approval. Read-only newer schemas and disk-conflict write guards remain.

Restore occurs after native document-open requests; an explicit Finder/open
request wins. Missing/locked last workspaces show recovery options. Closing checks
drafts, live sessions and running operations; it releases the lease/watcher.
Timers have explicit ownership independent of the visible workspace. Show their
running state on Close Workspace and offer pause; quitting stops app-run timers.

### 2. One SSH connection model for login, terminal and tasks

Extend the existing ssh2 transport instead of adding a persistent server agent.
Resolve endpoint, route, identity and known-host policy once in the native layer.
Use that same resolved model for Test, remote PTY, exec, SFTP and Ansible inventory.

Support explicitly selected system/custom file keys, session-only encrypted-key
unlock, ssh-agent identity selection, password/keyboard-interactive challenges,
and declared jump-node chains. File references and managed copies stay in simple
user-local storage with protected permissions; no SSH Keychain dependency.
Private bytes and reusable secrets never enter serializable frontend state,
workspace TOML, process arguments, persisted transcripts or ordinary logs.
Secret-entry IPC is transient and bound to one native challenge; zeroize after use.

Use explicit node host/user/port and approved identity settings. An SSH-config
importer may import allowlisted declarative fields and show what was imported;
it must not silently execute ProxyCommand, Match exec or LocalCommand. Agent
forwarding is opt-in and off initially. Probe route/backend capabilities before
presenting auth choices; report missing agent, locked key and network failure
separately. Known-host enrollment is persisted in the application-owned store.

Generalize terminal session targets to Local | Node(node_id), retaining bounded
sequenced packets, backpressure, resize and close semantics. Remote PTYs use an
SSH channel requesting a PTY and shell; local PTYs retain portable-pty. Reuse the
same emulator UI, split/search/paste controls and session guards. Each session has
its own stable tab ID and target label; reconnect creates a new session and never
replays typed input automatically.

### 3. Scenarios, scripts and existing Ansible

Provide New / From Example / Import / Edit / Validate / Run / Stop, target
selection, variables, live per-node results and persistent run history. Initial
examples: read-only system facts, ensure a directory, deploy a file, restart a
service, stream service logs, certificate renewal/delivery and ComfyUI preparation.
The UI distinguishes native scenarios, external Ansible projects and scripts.

Native scenarios use typed adapters with check/apply/verify. Scripts support local
or node execution, declared Python/JavaScript/shell runtimes, input/output schemas
and artifact destinations. Deno is an optional node-side JavaScript runtime;
preflight reports its version/absence, and installation is an explicit recipe step.
A precompiled remote agent is unnecessary for these SSH-based workflows.

Use a controller-side Ansible Runner adapter for the existing Ansible project;
run its real Ansible implementation rather than translating it into the native
DSL. Select an existing Python environment or explicitly create a managed one;
pin the tested Runner/core/collection versions. Preflight Python, Ansible,
ansible.posix, rsync, connection identity, become and required input files.
Preserve roles/includes/templates/relative paths and project working directory.
Do not modify or copy credentials out of the original project as an import side effect.

Catalog all inspected entrypoints:

| Entry | Form / intended operation |
| --- | --- |
| ssh_playbook.yml | Initial host/user/SSH setup; bootstrap connection and become inputs |
| certbot-cloudflare.sh | Explicit legacy/manual controller command; show forced-renewal behavior |
| certs_sync.yml | Existing certificate-tree synchronization; disclose the whole-tree scope |
| xray_standalone_from_config.yml | Xray/dashboard/Nginx setup with project variables/templates |
| setup_only_dabaga.yml | Runtime/service provisioning without an application artifact |
| setup_dabaga.yml | Initial application setup/deployment with artifact input |
| deploy_dabaga.yml | Subsequent application deployment with artifact input |
| db_dabaga.yml | Database replacement/cache rebuild/service restart; show destructive effects |

Inventory choice is explicit: selected Ghosh nodes mapped to a generated static
inventory, or an explicitly selected existing project inventory with a preview of
its resolved hosts and --limit. Detect unexpected hosts and delegated/local tasks;
they are part of the execution scope. Native secrets are passed through a bounded
native broker/Runner prompt interface; encrypted-key sessions can use a temporary
private agent. Password transports preflight any helper dependency. No secrets in
CLI arguments or persistent Runner artifacts; isolate private job files and clean
them on completion/crash recovery. Do not disable SSH host-key checking.

Ansible execution can run controller plugins/lookups/local actions. Validation and
--check are not classified as inherently read-only or granted the native safe-check
approval bypass. Review the controller environment/source as executable input;
mask no_log data and avoid enabling verbose diffs by default. Bind each plan to
source/dependency hashes, environment, inventory, targets, variables and identity.
Unresolved dynamic dependencies prevent silently reusing a saved authorization.
Runner events feed durable runs with bounded/redacted output and task/host status;
cancellation distinguishes stopped, partially applied and unknown remote outcome.

### 4. Certificates, app-running timers and health checks

Add a new certificate workflow alongside the unchanged legacy commands:
preflight → renew due lineages → verify certificate/key/expiry → stage on targets
→ activate a complete certificate/key pair → nginx -t → restart Nginx → verify
origin TLS. Use Certbot renewal state with explicit absolute config/work/log and
credential paths. Normal timers do not force renewal. Abort before delivery if
issuance fails; handle each lineage and target result separately.

New selective delivery sends the required certificate material, preserves the
expected live/archive path relationship and protects private-key permissions.
Stage/version each node's material; restore the previous pair if validation fails.
A successful target must pass nginx -t before restart. Restart policy defaults to
nodes whose certificates changed; offer an explicit all-selected restart action.
Verify the origin using its endpoint plus SNI, so a CDN certificate is not mistaken
for successful origin installation. Partial failures remain retryable without
forcing another issuance or restarting already successful nodes.

Timers are an application service, independent of Codex automations. Store enabled
schedule bindings/history locally, with workspace identity/path, task/source hash,
explicit targets, timezone, next/last occurrence and missed-run policy. Scan known
enabled schedules for recent workspaces on startup/wake, without changing the
visible workspace. Coalesce missed occurrences into one catch-up run; prevent
overlap/duplicate execution using durable claims. Handle offline nodes, moved or
changed workspaces, crashes, DST and clock changes. Persist uncertain outcomes for
review rather than blindly replaying a potentially completed deployment.
The app must be open for execution; on next launch it evaluates overdue tasks.

Health-check templates cover SSH reachability, HTTP status, service state,
certificate expiry and disk thresholds, with debounce/recovery state. Telegram is
an integration design deliverable: event schema, destination/token setup, redacted
message preview, quiet/recovery rules and test strategy. Actual bot delivery and
inbound remote commands require a later implementation slice; no messages are
sent during this work's inspection or tests.

### 5. On-demand logs and resource panels

Open Monitoring or Logs from a node row, terminal tab or command. Start collection
only when requested; Pause/Stop closes streams. Linux collectors read CPU counter
deltas, memory availability and cgroup limits where applicable; distinguish host
from container metrics, missing data from zero, and reconnect gaps from samples.
Use native local OS counters for the macOS local-machine panel.

Stream selected systemd units with journal cursors; support bounded file-tail
sources on nodes without journald, with rotation handling. Include timestamp,
source, severity/filter/search, pause/follow and explicit export. Bound polling,
retained samples, line sizes, sessions and IPC queues; render logs as inert text.
No remote daemon is required. This provides the requested live operational panels;
long-term Prometheus/Loki storage and arbitrary Grafana dashboard imports are
separate extensions.

### 6. ComfyUI / RunPod recipes from the user's examples

First recipe families: **MiniMax H3, Wan 2.2, LTX 2.3 and imported LoRAs**.
RunPod SSH endpoint/port/user, ComfyUI path, Python environment, persistent model
storage and launch command are recipe parameters, not guessed global constants.
Preflight existing install, CUDA/GPU/VRAM, disk, writable paths and model access.
Pin ComfyUI and plugin commits, Python constraints, model revisions and hashes in
an inspectable lock file. Preserve an existing installation through plan/diff and
backup/recovery steps; show plugin code and dependency changes before Apply.

| Family | First template behavior |
| --- | --- |
| MiniMax H3 | Local native ComfyUI workflows; separate required diffusion, text encoder and video/audio VAE files; distinguish T2V/I2V and reference variants |
| Wan 2.2 | Select a supported T2V/I2V/TI2V workflow and its matching model set; keep variant-specific loaders/files together |
| LTX 2.3 | Use the publisher's ComfyUI-LTXVideo integration and matching example workflow with its required model assets |
| LoRA | Import URL/repository revision/local file; record base-model family/version, hash, license, destination and workflow slot/strength |

“NSFW LoRAs” identifies an intended model category, not a particular repository.
Provide the same user-supplied asset importer; do not invent download URLs,
compatibility or license rights, or ship unsolicited model weights/previews.
Concrete LoRA URLs and the actual RunPod endpoint can be supplied in the form when
used; their absence does not block the recipe editor or other workflow families.

Downloads show bytes/progress, resume only against the same immutable asset,
verify integrity before atomic publish and reuse matching cached models. Tokens
stay in protected local credential files/references. Respect storage persistence;
warn about ephemeral target paths before large downloads. Installation success
requires dependencies and expected /object_info nodes, not just a successful git
clone. A second apply should report no change for the same lock state.

Allow importing ComfyUI API-format workflows, binding prompt/reference inputs,
submitting them through an authenticated SSH-local-forwarded endpoint, watching
prompt-specific events/history and saving verified output artifacts. A small smoke
workflow tests execution without automatically buying GPU time or downloading every
large model. Actual GPU/model-family acceptance needs an available chosen RunPod
node and its permitted downloads; Debian validates orchestration and small fixtures.

### 7. Live browsers, routes, scripts and recordings

Add a distinct **Browsers** session destination alongside Browser Profiles.
Profiles own immutable UUID data stores, cookies and declared user-agent/locale
settings; sessions own pages/tabs and selected automation-agent capability grants.
Tunnels own supervised SSH/SOCKS/VLESS routes with actual health and exit checks.
Keep all multiwebview platform work behind WebviewProfileBackend and the pinned
Tauri unstable feature; verify version-specific store/proxy capabilities first.
Test two profiles' cookie isolation and direct/proxy behavior; unsupported routing
fails visibly rather than silently falling back to direct traffic.

Implement real CRUD/start/stop in Profiles/Tunnels/Scripts/Recordings and connect
commands/buttons to the same service. Scripts consume typed action results and
save artifacts through scoped native APIs. Record/replay page interactions with
inspectable selectors, waits and input schemas. Browser-agent/page content receives
no shell privileges and cannot authorize its own actions.

Krea acceptance: manually authenticate in a chosen persistent profile; provide text
and/or an image reference; inspect and run a script/recording; observe request
completion; save the resulting image and metadata into the chosen workspace output.
Selectors and download handling are site-version-dependent. Use a local fixture
for automated tests, then the real site with the user's available session; do not
submit paid generations or upload private images as an incidental smoke test.

### 8. 3D scene automation

A subsequent Scenes destination owns versioned scene documents, asset references,
objects/materials/lights, camera paths, timeline/keyframes and scriptable commands.
Use Three.js for viewport/rendering; support glTF scenes, imported skeletons,
bone posing and animation clips, plus basic bone hierarchy/keyframe editing.
Keyboard forms and an object tree provide every transform/camera/timeline action
without requiring dragging. Save/open/replay must preserve the scene state.

Export a deterministic frame sequence with fixed simulation time and selected
9:16, 16:9 or 1:1 dimensions, then encode video using an explicitly configured
FFmpeg executable with reviewed packaging/license handling. Preview and export
use the same camera/timeline. Test actual native WebGL/GPU behavior and duration,
resolution, cancellation and output cleanup. Advanced mesh skinning, automatic
rigging, sculpting and Blender-file compatibility are later capabilities.

## File changes

Paths below are concrete ownership boundaries for implementation; new service
crates are created only when their slice starts. No empty service stubs or disabled
UI counted as completion. No files are deleted. Existing generated DTOs/schemas
and permissions are regenerated from their owning definitions.

| Files | Change and responsibility |
| --- | --- |
| src/app/App.tsx; commands.ts; CommandPopup.tsx; tabs.ts; AppShell.tsx; src/styles.css | Modify lifecycle orchestration, executable palette actions, bracket shortcuts, per-session tabs and bounded layouts |
| src/app/ipc.ts; crates/ghosh-core/src/contracts.rs; lib.rs; examples/export_contracts.rs | Modify typed lifecycle/auth/terminal/run/stream contracts and exports |
| crates/ghosh-core/src/workspace/model.rs; mod.rs; inventory.rs | Modify versioned scenario references, default local approval and validation/migration |
| crates/ghosh-core/src/approvals.rs; policy.rs; ssh.rs; terminal.rs; ssh_keys.rs | Modify operation intent bindings, native auth challenges and typed terminal targets; preserve remote-agent policy |
| src-tauri/src/services.rs; services/ssh.rs; commands.rs; lib.rs; build.rs; permissions/commands.toml; capabilities/desktop.json | Modify service composition, history/restore/close, command exposure and permission boundaries |
| crates/ghosh-ssh/src/lib.rs; crates/ghosh-term/src/lib.rs | Modify authentication, routing/channels/exec/SFTP and common local/remote session lifecycle |
| src/features/nodes/NodesView.tsx; NodeEditor.tsx; ConnectionTest.tsx; src/features/terminal/TerminalView.tsx; src/features/settings/SettingsView.tsx | Modify Save→Test/Open Terminal, actionable auth errors, sessions, restore/history/timer preferences |
| crates/ghosh-core/src/tasks.rs; runs.rs; schedules.rs; monitoring.rs | Create portable task/run/event/schedule/telemetry models and bounded validation |
| src-tauri/src/services/execution.rs; ansible.rs; schedules.rs; monitoring.rs; scripts/ansible-runner-bridge.py | Create native execution composition, private controller bridge, schedule claims and stream lifecycle |
| src/features/playbooks/PlaybooksView.tsx; src/features/runs/RunsView.tsx; src/features/monitoring/MonitoringView.tsx | Create working editor/import/target/run/history/log/metrics flows |
| examples/playbooks/system-facts.yml; ensure-directory.yml; certificate-delivery.yml; examples/scripts/save-result.js | Create documented, runnable small examples with explicit inputs/outputs and no secrets |
| examples/ansible/server-first-run.catalog.toml | Create entrypoint descriptors and variable/input guidance referencing the selected external project; no copied inventory or credentials |
| crates/ghosh-certs/Cargo.toml; src/lib.rs; src/features/certificates/CertificatesView.tsx | Create certificate planning/renewal/delivery/verification and timer setup UI |
| crates/ghosh-comfy/Cargo.toml; src/lib.rs; src/features/recipes/RecipesView.tsx; examples/recipes/minimax-h3.toml; wan-2.2.toml; ltx-2.3.toml; lora-import.toml | Create recipe plan/apply/verify, lock state and parameterized family templates |
| crates/ghosh-webview/Cargo.toml; src/lib.rs; crates/ghosh-tunnel/Cargo.toml; src/lib.rs | Create isolated native profile/session backend and supervised route service |
| crates/ghosh-script/Cargo.toml; src/lib.rs; crates/ghosh-recorder/Cargo.toml; src/lib.rs | Create scoped result/artifact scripting and record/replay services |
| src/features/browser/BrowsersView.tsx; ProfilesView.tsx; src/features/tunnels/TunnelsView.tsx; src/features/scripts/ScriptsView.tsx; src/features/recordings/RecordingsView.tsx | Create real session/entity controls and browser automation editor/results |
| src/features/scenes/ScenesView.tsx; model.ts; renderer.ts; src-tauri/src/services/scenes.rs | Create scene state/viewport/keyboard editing and native frame/video export |
| docs/TELEGRAM_INTEGRATION_PLAN.md | Create integration design, configuration/event contracts and verification plan; no live messaging |
| crates/ghosh-core/tests/workspace.rs; approvals.rs; inventory.rs; tests/ui/shell.spec.ts | Modify regression coverage for lifecycle, policy, node actions, commands and keyboard focus |
| crates/ghosh-ssh/tests/session.rs; tests/operational/ansible.py; certificates.py; tests/ui/operational.spec.ts | Create isolated SSH/Ansible/certificate and full UI workflow acceptance fixtures |
| Cargo.toml; Cargo.lock; src-tauri/Cargo.toml; package.json; package-lock.json | Modify workspace membership/pinned runtime dependencies when owning slices begin |
| README.md; docs/WORKSPACE_SCHEMA.md; SCRIPTING_API.md; six memory files; docs/evidence/operational/README.md | Modify actual contracts/status, setup/use cases and verified native screenshot/test evidence |
| scripts/check.sh; scripts/build-release.sh; .github/workflows/ci.yml | Extend appropriate checks/packaging; retain tag-only, sequential macOS arm64 → Linux x86_64 release builds |

## Implementation steps

1. **Workspace and commands.** Implement lifecycle/default approval contracts,
   restoration priority, clearable history, bracket shortcuts and palette dispatch.
   Test dirty/live-session guards, inert open, restart restoration, missing paths,
   native mouse and keyboard activation, small layouts and 200% zoom.
2. **Usable node terminals.** Add native auth challenges/identity resolution,
   enrollment, shared connection capabilities and managed remote PTYs. Connect
   Save→Test→Open Terminal and node/context/palette actions. Test file/agent/locked
   key/password/jump fixtures, host-key mismatch, resize, Ctrl+C, editors, disconnect
   and cleanup; then local macOS, debian.orb.local and node3.dabaga.cc sequentially.
3. **Author and run scenarios/scripts.** Add tasks/runs/events, bounded execution,
   examples and actual Play/Runs screens; integrate the Ansible controller/catalog.
   Test fixture Ansible project, per-host failures, injected strings, no_log,
   cancellation, source drift and controller-side actions; run a harmless facts
   scenario against both authorized nodes. Review external production plans without
   applying bootstrap/deployment/database changes merely to test Ghosh.
4. **Certificate workflow and timers.** Add renew/deploy/verify and schedule claims,
   recovery/missed-run handling, health templates and Telegram design. Test with
   local/staging certificates and disposable Nginx targets, forced failures and
   partial retries; no production forced renewals or fleet restarts for smoke tests.
5. **Live monitoring.** Add stream subscriptions/cursors/samples, actual node charts
   and logs. Compare readings with node tools; test container semantics, rotation,
   reconnect, caps and Stop releasing resources on both authorized test hosts.
6. **ComfyUI recipes.** Ship the four template families and editor/lock/download/
   verification/run/artifact flow. Test small assets and mock ComfyUI API first;
   validate chosen real families on a supplied RunPod GPU when available. Record
   untested GPU configurations explicitly instead of reporting fixture success as
   real installation/generation acceptance.
7. **Browsers and automation.** Complete profile isolation, sessions/routes,
   scripting/result saving and recording replay with local site fixtures and real
   native WebKit tests; verify the Krea journey when the session is available.
8. **Scenes and release acceptance.** Build scene editing/animation/export, test all
   ratios and native GPU output. Capture current working screens, update README
   with setup and complete use-case walkthroughs. Commit bounded changes to main;
   after checks, publish a new consistent version/tag and inspect both tag artifacts.
   Record a slice as complete only when its end-to-end acceptance passes.

## Verification

For each source slice, run relevant targeted checks first, then the existing suite
sequentially: npm run typecheck, npm run build, npm run test:ui; cargo fmt --all
-- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test
--workspace. Follow scripts/check.sh for the repository's target/profile settings.
Use the exporter --check mode from scripts/check.sh to verify generated DTO/schema
freshness; regenerate through npm run contracts only when changing owning contracts.
Native Mac acceptance is required in addition to browser-mocked IPC tests.

Keep production-altering and large-download/GPU tests separate and explicit.
SSH smoke commands on the two authorized hosts are limited to benign identity,
TTY, file fixtures in a dedicated temporary directory and on-demand read-only
metrics/logs. Check screenshots for private data. Test artifact cleanup, queue caps,
secret redaction and crashed-run recovery. Build macOS arm64 and Linux x86_64
artifacts on their actual platforms; distinguish build success from feature parity.
README must show actual native screens and instructions for workspace lifecycle,
key/node setup, terminals, script/scenario examples, importing server-first-run,
certificates/timers, logs/metrics, ComfyUI and implemented browser/scene features.

## Risks / edge cases

- This is several operational milestones; avoid another cosmetic-only release.
  Each slice ends in a usable verified journey before the next service expands.
- libssh2 may not support every key/security-token format or SSH configuration;
  capability probes must fail with specific recovery guidance. Ansible uses a
  separate SSH process, so equivalent identity/host/jump settings need explicit tests.
- Ansible controller plugins and check_mode:false tasks can execute during check.
  Third-party scripts/plugins and dependency installation are executable code.
- Certificate account trees contain broader secrets than leaf private keys.
  Legacy full-tree synchronization and new selective delivery must be distinct.
- App sleep/quit, encrypted keys and expired identities can defer a due timer.
  Status must show the cause; execution after a crash may have an unknown outcome.
- Model ecosystems change rapidly; pin verified revisions instead of guessing VRAM,
  plugin compatibility or universal LoRA interchangeability. Upgrades are reviewed.
- WebKit proxy/data-store support, website changes and native GPU/render/codec
  behavior require real platform tests. No placeholder success or sample run history.

## Out of scope

- Modifying server-first-run or the user's original 1.ghosh workspace as an import
  side effect; copying their private variables, keys, inventories or certificate data
  into Ghosh Git, screenshots or public release artifacts.
- A reimplementation of Ansible, a persistent remote server agent, unattended
  execution while Ghosh is closed, and automatic RunPod purchasing/provisioning.
- Live Telegram messaging/inbound bot commands in the initial operational slices;
  this request calls for integration planning.
- A complete Grafana/Prometheus/Loki replacement, strong browser anti-detection
  guarantees, automatic website login/captcha bypass or generic task rollback.
- Automatic full character skinning/rigging, sculpting, Blender renderer/file parity
  and unverified cross-platform browser/3D feature parity.

## Primary references checked for this proposal

- [ssh2 session authentication](https://docs.rs/ssh2/0.9.6/ssh2/struct.Session.html):
  the existing library exposes agent, passphrase, password and interactive auth.
- [Ansible Runner interface](https://docs.ansible.com/projects/runner/en/stable/python_interface/):
  event/status callbacks and cancellation inform the controller bridge.
- [Ansible check-mode semantics](https://docs.ansible.com/projects/ansible-core/devel/playbook_guide/playbooks_checkmode.html):
  task overrides mean external checks cannot use the native guaranteed-safe bypass.
- [Certbot renewal](https://eff-certbot.readthedocs.io/en/stable/using.html#renewing-certificates):
  normal renewal and renewal/deployment hooks inform the certificate pipeline.
- [MiniMax H3 native ComfyUI workflows](https://docs.comfy.org/tutorials/video/minimax/minimax-h3-native),
  [Wan 2.2 integration](https://github.com/Wan-Video/Wan2.2),
  [LTX integration](https://github.com/Lightricks/ComfyUI-LTXVideo):
  family-specific model/workflow packaging; verify exact locked assets when implementing.
- [ComfyUI server routes](https://github.com/Comfy-Org/docs/blob/main/development/comfyui-server/comms_routes.mdx):
  prompt IDs, WebSocket events, history and output retrieval inform the run adapter.
- [systemd journalctl](https://www.freedesktop.org/software/systemd/man/258/journalctl.html):
  structured output/cursors inform the journal stream adapter; probe installed versions.
- [Three.js animation system](https://threejs.org/manual/pages/animation-system.html):
  imported clips, bones and animation mixing inform the bounded scene editor.
