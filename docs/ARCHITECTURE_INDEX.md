# Architecture index

## Existing application
| Path | Responsibility / entry points |
| --- | --- |
| crates/ghosh-core/src/contracts.rs | DTOs, static error envelope, cancellation token |
| crates/ghosh-core/src/ssh_keys.rs | File-key registry, metadata, precedence, preview/commit and generator/inspector ports |
| crates/ghosh-core/src/workspace/inventory.rs, identity.rs | Revisioned node CRUD, file identities and explicit schema migration |
| crates/ghosh-core/src/terminal.rs | Bounded session/packet/size IPC DTOs |
| crates/ghosh-core/src/ssh.rs | Public endpoint/server-key/review/result DTOs; no private authentication bytes |
| crates/ghosh-ssh/src/lib.rs | Bounded direct transport, known-host algorithm preference, fingerprint verification and file-key authentication |
| src-tauri/src/services/ssh.rs | Trusted context, expiring single-use reviews, native key resolution and authorized-host opt-in test |
| src/features/nodes/ConnectionTest.tsx | Keyboard review, unknown-host opt-in and authentication results |
| crates/ghosh-term/src/lib.rs | Real local PTY lifecycle, bounded streams, resize and process-group cleanup |
| crates/ghosh-core/src/operations.rs | Bounded jobs, cancellation, closed stage vocabulary |
| crates/ghosh-core/src/approvals.rs | Native contexts, exact grants, monotonic expiry and audit |
| crates/ghosh-core/tests/approvals.rs | Binding/denial/replay/revocation/capacity tests |
| crates/ghosh-core/src/policy.rs | Principals, edit/secret authorization, required approvals |
| crates/ghosh-core/src/secrets.rs | SecretRef/SecretValue/SecretStore boundary |
| crates/ghosh-core/src/workspace/mod.rs | Workspace create/open/snapshot/rename/reload/trust |
| crates/ghosh-core/src/workspace/model.rs | Manifest/node projections, identity/path/reference checks |
| crates/ghosh-core/src/workspace/document.rs | Lossless TOML AST, safe patches, secret-field rejection |
| crates/ghosh-core/src/workspace/storage.rs | Anchored filesystem, leases, revision hashing, atomic publication |
| crates/ghosh-core/src/workspace/watch.rs | Bounded debounced native watcher and revision fallback |
| crates/ghosh-core/tests/workspace.rs | 24 filesystem/schema/conflict/trust/watcher behavior tests |
| crates/ghosh-core/tests/policy.rs | Three policy/reference/redaction tests |
| crates/ghosh-core/examples/export_contracts.rs | Deterministic TS/JSON Schema generation and --check |
| src-tauri/src/main.rs, lib.rs | Desktop entry, service composition and early Finder document queue |
| src-tauri/src/services/history.rs | Bounded atomic MRU/last-open storage and legacy migration |
| src-tauri/src/services.rs | Active workspace/watch, lifecycle/startup priority, bounded pending requests/tests |
| src-tauri/src/commands.rs | 34 workspace/key/node/SSH/terminal/operation IPC commands, caller validation/test, blocking-work bridge |
| src-tauri/src/platform | Native OpenSSH key generator/inspector; separate Keychain adapter and optional integration test |
| src/main.tsx | React entry |
| src/app/App.tsx | Welcome/create/open/close/restore/recent, busy/error/draft/document-open guards |
| src/app/AppShell.tsx, commands.ts, tabs.ts | Nightfly shell, action registry and stable tabs |
| src/components/Dialog.tsx, src/app/CommandPopup.tsx | Keyboard popup, trapped modal focus and restoration |
| src/app/useOperations.ts | Subscribe-before-fetch operation snapshots and sequence reconciliation |
| src/features/operations | Native jobs and approval review/audit views |
| src/features/nodes, src/features/settings | Node CRUD/editor and local appearance/keyboard settings |
| src/features/keys | Key library and workspace-identity migration dialogs |
| src/features/terminal/TerminalView.tsx | xterm sessions, split/search/paste review and terminal focus |
| tests/ui/shell.spec.ts, playwright.config.ts | 14 sequential browser presentation tests with explicit IPC fixtures |
| src/app/ipc.ts | Typed commands and event subscriptions |
| src/features/workspace/WorkspaceView.tsx | Name editing, inventory, local approval/pause, conflict/reload/details |
| src/styles.css | Nightfly responsive keyboard-first desktop presentation |
| src/generated/contracts.ts | Generated core DTO projection |
| schemas/workspace.schema.json, node.schema.json | Generated structural schemas |
| assets/ghosh.svg, src-tauri/icons | Source brand mark and bundle icons |

## Existing build and configuration
- Rust: Cargo.toml/Cargo.lock, rust-toolchain.toml; four members: core, term, ssh and desktop.
- Frontend: package.json/package-lock.json, tsconfig.json, vite.config.ts, index.html.
- Shell: src-tauri/tauri.conf.json, Info.plist, build.rs, capabilities/, permissions/.
- CI: .github/workflows/ci.yml; tag-only macOS arm64 then Linux x86_64 jobs.
- Release: scripts/verify-release.mjs, check.sh, build-release.sh; ignored artifacts/.
- Shared UI: src/components/Select.tsx, Icon.tsx, Shortcut.tsx; styling in src/styles.css.
- Linux legacy-secret boundary: src-tauri/src/platform/unavailable.rs; no plaintext fallback.
- Instructions/licenses: README.md, LICENSE-MIT, LICENSE-APACHE, .gitignore.
- Runtime package: <workspace>.ghosh/ghosh.toml and configured entity directories.
- App data: Tauri app_data_dir; workspaces.json (legacy recent.json migration), ssh/registry.json, ssh/<uuid>.key and locks, outside packages.
- Build output (ignored): target/aarch64-apple-darwin/release/bundle/macos/Ghosh.app.
- Dependency/output caches (ignored): node_modules/, .cache/, dist/, target/.

## Documentation
| File | Purpose |
| --- | --- |
| [ghosh-prompt.md](ghosh-prompt.md) | User-owned ignored specification, normative A/B |
| [PROJECT_CONTEXT.md](PROJECT_CONTEXT.md) | Current product/scope/constraints |
| [MODULE_OWNERSHIP.md](MODULE_OWNERSHIP.md) | Implemented ownership and future boundaries |
| [API_CONTRACTS.md](API_CONTRACTS.md) | Current API/persistence/security contracts |
| [DECISIONS.md](DECISIONS.md) | Accepted/proposed architecture decisions |
| [TASK_LOG.md](TASK_LOG.md) | Actual results, limitations and next increment |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Full target architecture and native risks |
| [WORKSPACE_SCHEMA.md](WORKSPACE_SCHEMA.md) | Target A1–A8 schemas; later fields not fully activated |
| [SCRIPTING_API.md](SCRIPTING_API.md) | Future A9 runtime/API contracts |
| [MCP_SURFACE.md](MCP_SURFACE.md) | Future MCP B transport/tools/resources/approval contracts |
| [MILESTONES.md](MILESTONES.md) | M1–M8 sequencing and gates |
| [IMPLEMENTATION_PLAN_M1.md](IMPLEMENTATION_PLAN_M1.md) | Approved M1.1 plan, completion and deviations |
| [BASE44_DESIGN_REQUEST.md](BASE44_DESIGN_REQUEST.md) | Keyboard-first Nightfly UI; 76 screens + key/playbook details; real terminal gates |
| [SSH_KEY_STORAGE.md](SSH_KEY_STORAGE.md) | Implemented file-key contract; remaining unlock/SSH integration |
| [base44/README.md](base44/README.md) | Capture guide, ten native JPEGs, offline gallery and handoff ZIP |
| [IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md](IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md) | Approved sequential native UI/services plan; 14 phases and acceptance gates |
| [IMPLEMENTATION_PLAN_OPERATIONAL_WORKFLOWS.md](IMPLEMENTATION_PLAN_OPERATIONAL_WORKFLOWS.md) | Accepted operational supplement: SSH auth/PTYs, Ansible, certificate timers, metrics/logs, named ComfyUI recipes, browsers and scenes |
| [base44/reference-app/INTERACTION_INVENTORY.md](base44/reference-app/INTERACTION_INVENTORY.md) | Live Base44 reference: 32 interaction groups, commands/settings, observed limits |
| [base44/reference-app/README.md](base44/reference-app/README.md) | 33 reference screenshots, manifest and checksums; separate from native captures |

## Future modules — no source directories yet
- Extend existing ghosh-ssh: pool/channels/SFTP/jumps/persistent host enrollment (M1.2).
- ghosh-core tasks/adapters: execution engine and resource adapters (M1.3).
- ghosh-term exists for local PTY; managed SSH terminal integration remains pending.
- ghosh-webview and ghosh-tunnel: all multiwebview types, profile stores/routes (M3).
- ghosh-certs: issue/deploy/verify certificates (M4).
- ghosh-script and ghosh-recorder: runtimes/editor/recording/replay (M5).
- ghosh-comfy: pinned recipe orchestration (M6).
- ghosh-mcp and ghosh-cli: shared-core adapters and running-app attachment (M7).
- Cross-service/native conformance fixtures will be added with their owning services.
