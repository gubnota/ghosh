# Module ownership

Status: workspace/node/key persistence, Nightfly shell, jobs/approvals, local/SSH PTYs,
simple checked scenarios, scripts and upstream Ansible execution are implemented.
Future services remain explicitly listed below.

## Implemented ownership
| Owner | Responsibility | Allowed dependencies |
| --- | --- | --- |
| ghosh-core/contracts | Safe DTOs/errors and cancellation primitive | Portable serde/TS tooling |
| ghosh-core/operations, approvals | Bounded job metadata; exact native request/grant policy and audit | Core contracts/policy/portable identity/hash/time |
| ghosh-core/policy | Human edit authorization, secret-service access, mandatory approval set | Core contracts/portable identity |
| ghosh-core/secrets | Zeroized values, reference validation, SecretStore port | Core policy/contracts; no platform APIs |
| ghosh-core/workspace | TOML validation/patches, references, leases, revision/atomic persistence, watcher/trust | Core contracts/policy and portable filesystem libraries |
| src-tauri/services | Active workspace/watch/lifecycle, app-local MRU/restore state, open queue, jobs and fresh-disk reviews | Core; Tauri events; injected queue |
| src-tauri/commands | Local caller check and typed IPC; offload blocking work | Services/core and native file dialog |
| ghosh-core/ssh_keys | User-local registry, safe files, key metadata and precedence | Portable filesystem/crypto; injected generator/inspector |
| ghosh-ssh | Direct handshake, exact host/agent identity, file/agent/password auth and SSH PTY; no workspace/UI policy | Core SSH/Pty ports; ssh2/crypto |
| src-tauri/services/ssh | Trusted fresh context, bounded single-use review tickets, native-only key resolution | Core, ghosh-ssh, active workspace serialization |
| ghosh-term | Shared local/remote sessions, bounded replay/input, resize and local child cleanup | Core Pty port; portable-pty/libc |
| src-tauri/platform | OpenSSH key adapters; separate workspace-scoped Keychain adapter | Core key ports/SecretStore; native tools/framework |
| src-tauri/lib.rs | Desktop composition; queue exists before setup/native Opened events | Shell/core |
| React src | Presentation, drafts/dialogs/busy/error state and event coordination | Generated DTOs and typed src/app/ipc.ts |

- App shell owns command registry, accessible modal focus and shortcut dispatch;
  commands use the same typed services as controls, never shell eval.

## Dependency and communication rules
- Adapters depend inward on core-owned contracts/ports; core never imports Tauri/React.
- Native Keychain values cannot cross desktop IPC; only native services receive SecretStore.
- UI cannot implement storage validation, raw key resolution, process spawning or SSH execution.
- Terminal UI necessarily carries user-entered PTY bytes; it never receives private key files.
- ghosh-term owns shell/process lifecycle; React owns emulator parsing and acknowledgement.
- Rust rechecks caller identity, workspace ID, revision and filesystem state for edits.
- No filesystem work on the UI thread; commands offload it to blocking workers.
- Services serialize active-workspace mutations; watcher callbacks emit safe invalidation hints.
- Document-open events queue before initialization; they do not load/execute packages directly.
- Frontend subscribes before draining; unsaved changes gate switching/reload/close.
- Remote/page/external MCP output stays untrusted data, never executable instructions.
- Keep peer adapters acyclic; cross-service work uses injected core ports/handles.

## Future ownership
- Extend ghosh-ssh: jumps/pool/SFTP and persistent host enrollment; core/route ports.
- Existing file-key registry/adapter stays separate from Keychain-only SecretStore.
  Per-call encrypted-key unlock works; there is no persisted unlock cache.

- ghosh-tunnel: route generations/leases/sidecars/health; core/platform process bindings.
- ghosh-webview: UUID store lifecycle, every native multiwebview type and page operation.
- Only ghosh-webview may use multiwebview APIs, behind WebviewProfileBackend.
- Existing ghosh-term accepts the core Pty port for local and managed SSH sessions.
  Terminal input ownership and reserved app shortcuts are explicit in the shell.
- Native terminal smoke owns local shell/full-screen/resize/signals; SSH fixtures
  separately own managed PTY/auth/route/disconnect tests. Local/remote manual
  TTY and full-screen editor smoke passed; native managed authentication passed both
  authorized hosts. Managed PTY passed both hosts; route/disconnect acceptance remains pending.
- ghosh-certs: issuance/deployment/TLS verification; SSH/secret ports.
- ghosh-script/recorder/comfy: orchestration through shared core APIs, not duplicate logic.
- ghosh-mcp/cli: protocol/command adapters; cannot bypass core policy.
- Never hold workspace/node locks while awaiting a human approval; consume under fresh binding checks.
- Future script subprocesses are trusted local code; binding grants are not OS isolation.

## Build and testing ownership
- Root tooling owns locked toolchain/dependencies, CI, licenses and build scripts.
- src-tauri owns package identity, UTI, icons, ACL/CSP and future signing/notarization.
- Core exporter owns generated TS/JSON Schemas; consumers never hand-edit outputs.
- Core tests own persistence/schema/conflict/secret/policy behavior (see TASK_LOG.md for current results).
- Shell tests own caller restrictions and early/bounded document queue and fresh-disk approval invalidation (see TASK_LOG.md for current results).
- Native Keychain test is opt-in, creates a temporary item and cleans it up.
- Browser fixtures own 28 UI/focus/reconciliation journeys; do not claim native effects.
- Shared Select owns themed menus, keyboard selection/typeahead and viewport positioning.
- Platform adapters own legacy-secret availability; Linux fails closed, file keys stay separate.
- Release tooling owns version guards, native packaging/checksums and tag-only CI.
  Linux CI is temporarily commented out per user; macOS arm64 remains enabled.
- Interactive smoke owns real Finder/file-dialog/dirty/trust/restart behavior and UI layout.
- Future ghosh-webview owns two-profile/one-window cookie and routing tests.
- Future ghosh-mcp owns protocol/Inspector B3; core owns shared execution/policy fixtures.
- Update all six memory files and record actual checks for every milestone increment.

## Automation authoring boundary
- Core automation owns bounded YAML/script validation; workspace/automation owns safe source paths and exact revision/digest writes.
- Native automation exposes four typed local-shell operations; frontend owns draft/example presentation.
- Catalog/validation/open remain inert. Core runs owns portable task state transitions; it never resolves keys.
- Native runs owns worker/target validation, revision checks, private reports and cancellation.
- Local adapter owns process groups/pipes; SSH adapter owns non-PTY channels and uncertain disconnect outcomes.
- Frontend owns target/input/credential presentation and phase report polling; no direct process or key access.
- Native ansible.rs owns reviewed identities and process/report integration; ansible_bridge.py delegates all Ansible semantics to pinned upstream Runner.
- Simple schema: 1 check/apply/verify stays in core runs; standard Ansible is never translated into that format.
- ComfyUI frontend owns reviewable standard YAML generation; shared Run services execute it.
- Controller integration tests own real module/idempotence/no_log/include/role/inventory/handler/cancel acceptance.
- GPU/template compatibility and live agent/password Runner acceptance remain separate gates.
- SSH UI auto-consumes native tickets without human confirmation; transport owns key pinning and changed-host rejection.
