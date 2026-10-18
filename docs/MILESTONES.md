# Ghosh milestone plan

Status: M1.1 complete; approved UI/functionality refinement in progress. Date: 2026-10-18. Scope follows the user's M1–M8 sequence.
Source implementation begins only after the bounded implementation plan is
approved. M1 core includes policy/cancellation contracts so later MCP is an adapter.
No future feature receives a success-shaped stub in a shipped UI.

The [reference UI/functionality plan](IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md)
is an approved 14-phase delivery refinement after live Base44 inspection: move the
shell/keyboard foundation and local PTY forward, finish keys→SSH→playbooks/runs,
then complete the remaining modules. Nightfly shell and native jobs/approval infrastructure are implemented. Other
phases remain incomplete; milestone completion gates below remain authoritative.

| Milestone | Shipped behavior | Completion evidence |
| --- | --- | --- |
| M1 | Workspace create/open/recent/autosave/watch; strict SSH with pooling/jumps/routes; declarative idempotent tasks, check/diff and run log UI | parser/engine unit suite; local sshd integration; second apply=no changes; cancel/conflict/secret tests |
| M2 | Local/SSH PTY tabs, xterm renderer, search/splits/resize, send/expect | actual PTY lifecycle/backpressure/resize and SSH reconnect; renderer fallback; honest protocol capability matrix |
| M3 | One-window mixed profile tabs, UUID stores, UA capabilities, per-profile direct/http/socks5/VLESS routes | native macOS 14+ isolation/proxy/down-route/focus/DPI tests; route switch and restart evidence |
| M4 | DNS-01 certbot/Cloudflare, optional native adapter, sync/reload/TLS verify and app-running scheduler | ACME staging; temporary credentials cleanup; interrupted deployment; leaf fingerprint/expiry/SNI fixtures |
| M5 | JS runtime + trusted Python/Node bridges, offline editor, recording/replay/export/data | signed runtime spike; API parity/permission/timeout tests; WKWebView capture/replay; no secrets in recordings |
| M6 | ComfyUI pinned recipes, preflight, parallel resumable verified models, shared paths, plan/apply/verify/export | fixture node install; auth/download interruption/checksum mismatch; no-op second apply; unresolved export pins fail |
| M7 | MCP stdio shim and loopback HTTP server; external clients; resources/prompts/approvals | Inspector B3 plus automated protocol/policy suite; denied requests have zero business effects |
| M8 | Linux WebKitGTK and Windows WebView2/secret/process backends | same portable contracts and platform integration suites; signed distribution where applicable |

## M1 increments

Current status (2026-10-18): **M1.1 implemented and locally verified**; see
TASK_LOG.md. M1.2–M1.4 and M2–M8 remain planned. This does not complete M1.

1. M1.1 workspace foundation: minimal runnable Tauri/React app, validated v1
   manifest/nodes model, lossless edits, conflict-safe persistence, workspace trust,
   Keychain port/adapter, schema/type generation, policy/error/operation contracts.
   Only workspace UI is exposed. Plan is in IMPLEMENTATION_PLAN_M1.md.
2. M1.2 SSH transport: first plan file-key registry/schema/legacy migration per
   SSH_KEY_STORAGE.md (no SSH Keychain); existing/custom files, memory-only unlock,
   global/workspace/node defaults; strict known_hosts enrollment/mismatch, key/agent/password,
   pool, channel execution/SFTP, jump chains, explicit direct/HTTP/SOCKS routes;
   container-backed tests. VLESS route transport implementation arrives in M3;
   unavailable VLESS nodes return ROUTE_UNAVAILABLE rather than connecting direct.
3. M1.3 engine and adapters: guards/probes/diffs/check/apply/post-check, per-node
   write leases, retries/timeouts, cancellation/outcome_unknown; shell/file/template/
   apt/brew/pip/npm/systemd/git/download/docker/line_in_file, handlers/notify.
   Remote OS determines supported adapters, with explicit unsupported failures.
4. M1.4 operational UI/history: Nightfly readable monospace, keyboard-only Cmd+K
   command forms; discoverable add-node/create/import playbook, YAML authoring,
   inventory, targets, plan review, live per-node
   logs/results, run history, retention, first-run README and end-to-end acceptance.

Each increment has a narrowly reviewed file plan before source changes. M1 is not
complete after M1.1. Small commits separate foundational DTOs, persistence,
policy, adapters and UI; keep tests with each behavioral change.

## Native feasibility work and release gates

Before M3, test combined custom store + per-store proxy + two native child views
on minimum macOS 14 and a currently supported later macOS. The smoke must open
two views with different profiles in one window and verify bidirectional cookie
isolation at the same origin. Keep multiwebview types inside ghosh-webview behind
WebviewProfileBackend. Inspect the exact pinned Tauri/Wry initialization sequence;
record Cargo.lock versions and rerun native smoke for every reviewed upgrade. Demonstrate profile persistence across restart,
network route separation, no direct fallback, route edits affecting all profile
tabs, overlay/focus/resize correctness, and store purge isolation. Record traffic
leak limitations explicitly; strong network kill-switch remains unavailable until
its evidence exists. Do not wait until M7 to discover native profile incompatibility.

Before M5, build a signed hardened-runtime arm64 V8 sample and measure startup,
RSS and termination. Choose/record QuickJS explicitly if the packaging/resource
tradeoff wins; do not silently change runtime semantics.

## Verification, CI and distribution

Every milestone includes formatting, Rust lint/test, frontend typecheck/build and
focused functional tests; execute only checks applicable to the files changed.
M1+ CI uses locked dependencies and an arm64 macOS runner, plus service-backed SSH
integration where available. Docker-based sshd tests may use Linux CI for portable
core behavior, with macOS client/native smoke separately. Linux/Windows builds are
initially explicit placeholders and do not claim supported backend conformance.
A minimum-OS native runner is a separately provisioned requirement, not assumed
available on hosted CI. Never label a skipped fixture/native test as passing.

Release docs include dual-license files, dependency/sidecar license inventory,
resource caps, known limits, signed nested sidecars and notarization evidence.
M8 may revise distribution mechanisms per OS without changing core business APIs.

## UI/terminal acceptance amendment — 2026-10-18

BASE44_DESIGN_REQUEST.md is the revised product UI contract: keyboard-only action
coverage, Nightfly palette, file-based SSH identities, focused command modals and
playbook creation. Base44 may simulate native boundaries; the shipped app must
have a real local PTY and SSH PTY per node, not a log viewer. M2 cannot complete
without the executed local-machine terminal smoke and separate SSH PTY tests
specified there (resize, full-screen program, signals, focus and disconnect).
These additions are requirements, not completed runtime features or tests.
