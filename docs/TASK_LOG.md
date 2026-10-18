# Task log

## 2026-10-18 — Approved UI/functionality implementation in progress

- User approved the 14-phase plan with “go”; all implementation and verification
  remain sequential. Latest authorized SSH targets: debian.orb.local and node3.dabaga.cc.
- Implemented Nightfly shell, stable tabs, 24-command popup, keyboard navigation,
  focus-safe forms, appearance settings, native jobs and exact-context approvals.
- Implemented file-key library/reference/copy/generation/default/relink, schema 2
  workspace identity migration with recovery copy, node CRUD/import and inherited
  public identity preview. No SSH Keychain use; encrypted memory unlock still pending.
- Implemented real local PTY with xterm emulator, eight-session bound, lossless
  acknowledged output, bounded input, resize, split/search, paste review and close guards.
- Verification: 62 Rust tests pass, two opt-in tests skipped (Keychain and live SSH); formatting,
  warning-denied Clippy, generated contracts, strict TypeScript and production build pass.
- Fourteen UI tests pass using explicit IPC fixtures, including terminal input/output,
  split bounds, session cleanup and last-tab close guard. Fixtures do not prove SSH.
- Latest arm64 release bundle: PASS, 11.65 MiB, macOS 27 / deployment target 14.0.
  Final README/copy build and 14 UI regressions repeated successfully; fresh release
  launch PASS. App left at Welcome, with no active workspace or terminal sessions.
- Phase 6 slice: new ghosh-ssh crate (ssh2 0.9.6, vendored OpenSSL), two typed native
  IPCs, trusted direct-file-key review/authentication modal and node-test command.
  Tickets expire in 120s, are single-use and bind fresh workspace/native/key state;
  reconnect checks the exact reviewed key. Trust/reload/mutations rotate context.
- Opt-in native authorized_nodes_native_authentication PASS: debian.orb.local 110ms,
  node3.dabaga.cc 2654ms, user vm, known ED25519 hosts, unencrypted existing key.
  Test uses temporary registry/workspace and verifies consumed-ticket rejection.
- Rebuilt release UI PASS via keyboard: Debian 106ms and node3 2298ms; reviewed
  inherited workspace identity/server fingerprint before authenticating. Evidence
  20–22 shows the actual native review and results, with public metadata only.
- Native smoke registered the explicitly selected existing ~/.ssh/id_ed25519 by
  reference and assigned it in the disposable workspace. Global default stayed unset;
  no key copied/exported, no remote command executed by managed authentication.
- Initial libssh2 ECDSA preference conflicted with existing ED25519 known_hosts.
  Fixed host-algorithm selection to prefer matching recorded keys; changed/unknown
  keys and reviewed-key comparison retain strict checks. Added regression tests.
- Native local terminal: TTY=YES, ANSI color, UTF-8, 21×101 initial size, Ctrl+C,
  alternate-screen restoration, two independent sessions and resize to 21×48 PASS.
- Native full-screen vi: edited disposable /private/tmp/ghosh-pty-editor.Wc8RSd,
  Escape, typed :wq and returned to the shell; file contains GHOSH_EDITOR_OK. PASS.
  Clipboard paste is bracketed paste in Vim; editor commands were typed as keys.
- Native system SSH inside Ghosh: both authorized targets PASS with BatchMode,
  strict existing host-key checks, 8-second connection timeout and forced TTY.
  Debian OrbStack: vm / Linux ARM64; node3: vm / Debian Linux x86-64. Both report
  TTY=YES, 21×101, UTF-8/ANSI output and normal return to the local prompt.
- Initial app-only Debian connection returned No route to host. Added the macOS
  local-network usage description and rebuilt; subsequent native connection passed.
  This does not establish the original failure's cause. No host trust was weakened.
- Split layout initially overflowed in native WebKit; fixed intrinsic width/overflow,
  rebuilt and verified both visible panes, TTY resize and keyboard session selection.
- README now includes actual screenshots, setup/build, keyboard/key/node workflows,
  local/manual remote terminal instructions, use cases and honest service limitations.
- Native evidence 01–18 and 20–22 in evidence/ui-shell; reference Base44 evidence is separate.
  Original /Users/vm/Documents/1.ghosh untouched; disposable UI fixture used throughout.
- No remote installation/configuration/file writes, no new remote authorized keys,
  no SSH Keychain access, no commit/push/publication. System SSH used existing config.
- Remaining: encrypted-key unlock/consumers, managed SSH PTY/host enrollment/SFTP/
  facts, playbooks/runs and subsequent phases; minimum macOS 14 execution, route-loss
  recovery and release signing/notarization. Manual SSH is not managed-node integration.

## 2026-10-18 — M1.1 workspace foundation completed

- Authorization: approved M1.1 plan; user requested “go, not in parallel”. All work
  and verification were sequential, without subagents or concurrent builds.
- Added core + Tauri desktop + React/TypeScript app, locked toolchains/dependencies,
  generated DTOs/schemas, icons/UTI, dual licenses, README and sequential macOS CI.
- Implemented inert create/open, inventory, display-name save, reload, session trust,
  recent workspaces, native dialogs, dirty-state guards and Finder document opening.
- Core validates identities/time/paths/auth/references/approvals/profile UUIDs;
  preserves unknown TOML/comments and rejects inline-secret fields without echoing values.
- Storage anchors paths, rejects symlinks/root replacement, leases path plus identity,
  hashes bounded config, writes atomically with revision checks and stages package creation.
- No multi-file journal: creation publishes a completed stage with no-replace rename;
  crashed hidden stages remain inert. Arbitrary-editor races and network filesystems
  are not transactional. Automatic synced-volume classification is deferred.
- Added static safe errors, local-shell-only IPC ACL/caller validation, secret-safe
  port and native workspace-scoped Keychain adapter; no raw-secret desktop command.
- Native smoke uncovered missed filesystem startup notifications: added a bounded
  two-second revision fallback to the debounced native watcher.
- Native cold-open smoke uncovered early RunEvent::Opened before service setup:
  fixed by allocating the bounded document queue before initialization. Added FIFO
  and capacity regression tests; final release cold/warm document opens pass.
- Updated all six compact memory files, implementation outcome and milestone status.

### Executed verification
- Host: Apple Silicon macOS 27 / Xcode 27; Rust 1.96.0, Node 26.8/npm 11.19.
- Deployment target: macOS 14.0; target aarch64-apple-darwin; one build/test worker.
- Rust formatting and Clippy (workspace/all targets, warnings denied): PASS.
- Default workspace suite: PASS, 30 tests (24 persistence/workspace, 3 core policy,
  1 IPC caller, 2 document queue); 1 optional Keychain test ignored by default.
- Native Keychain create/read/delete/missing-after-delete: PASS in an explicit
  standalone run outside the restricted sandbox. The first sandbox run correctly
  returned SecretUnavailable; no prompt or Keychain read occurs during package open.
- Generated TypeScript + workspace/node JSON Schema freshness (--check): PASS.
- TypeScript strict typecheck and Vite production build: PASS, 37 modules.
- Arm64 release application bundle: PASS, Ghosh.app 7.14 MiB, local development build.
- Native UI: created /private/tmp/Ghosh-Smoke-M11-1018.ghosh as Untrusted; opened and
  cancelled native parent picker; renamed/saved and verified manifest on disk.
- Native external conflict: unsaved draft survived external name edit; save/trust
  blocked; reload confirmation cancellation retained draft, confirmation loaded disk.
- Explicit trust confirmation worked; restart/reopen reset trust; recent entry survived.
- Final release cold Finder open immediately displayed the requested package;
  app remained running, stderr had no panic, and warm document open stayed stable.
- Native light layout/contrast and focus/dialog behavior inspected through real UI.
- Final document/configuration checks: PASS, 69 text files without trailing whitespace,
  39 relative links, balanced Markdown fences, JSON/TOML parsing, CI YAML parsing
  and native Info.plist validation. User-owned ignored prompt was not rewritten.

### Limits and next work
- Minimum macOS 14 native execution is NOT verified on this macOS 27 host.
- Dark theme CSS compiles but was not visually smoke-tested; no browser automation
  suite substitutes for native UI evidence here.
- CI definition exists but no remote CI run, deployment, publication or push occurred.
- Distribution signing/notarization and dependency/sidecar license inventory are future gates.
- Full profile/cert/recipe validation, migrations beyond read-only future schemas,
  SSH/tasks/terminals/browser/routes/scripts/certificates/recipes/MCP remain future work.
- All source/docs are uncommitted; user-owned ignored prompt and .gitignore entries preserved.
- Next: narrowly plan M1.2 SSH transport before implementation; M1 remains incomplete.


## 2026-10-18 — Architecture, contracts and design request

- Request: architecture first, M1–M8 plan and Base44 request for every needed screen.
- Read the attached brief; then reconciled with `docs/ghosh-prompt.md` supplied by user.
- Bootstrapped all six memory files before narrow repository exploration.
- Baseline: initial bbac06a; no app/source/build/dependency/test configuration.
- Completed architecture: crate/port ownership, shared facade and Mermaid flows,
  workspace/execution/native routing, security/packaging, risks and test gates.
- Completed A1–A8 schema refinements, full A9 scripting API and MCP B contract.
- Included immutable UUID/store clear semantics and non-UUID rejection tests.
- Chose strict MCP startup rejection for missing B1 approvals; per-call node.run check.
- Added browser-sheet coverage and native two-profile/one-window cookie smoke gates.
- Recorded unstable feature, ghosh-webview boundary, Cargo.lock pin/review/fallback.
- Verified API availability against public WebKit headers and Tauri docs; no chosen
  dependency version or compiled integration yet. Recheck exact selected APIs at M3.
- Completed M1–M8 delivery/evidence plan and concrete M1.1 implementation file plan.
- Completed Base44 design request: 76 screens, state coverage and eight walkthroughs.
- Verification passed: 37 relative links, balanced fences and whitespace in 14
  documents; 12 TOML, 2 JSON and 3 YAML samples parsed (including source prompt).
- Verified both A1 approval lists against the B1 table, both A3 UUID v4 samples,
  full A9 declarations modulo formatting, 76 sequential screens and context length.
- These are document checks, not implemented parser/policy/runtime test results.
- Application builds/native/SSH/MCP tests: not run; no application exists yet.
- Source/configuration/dependencies: unchanged by this task; preserve user .gitignore edit.
- Next: user review of architecture and explicit M1.1 plan approval before source work.

## 2026-10-18 — Current-screen captures and revised Base44 handoff

- User requested screenshots/current UI request, discoverable remote playbooks,
  system/custom SSH key files without Keychain, better fonts, Nightfly decoration,
  terminal-like appearance, command popups and complete keyboard-only operation.
- Added real local PTY and per-node SSH PTY requirements plus explicit native
  local-machine and separate SSH acceptance steps after follow-up clarification.
- Revised BASE44_DESIGN_REQUEST.md: preserved 76 original entries; added K1–K9,
  P1–P12, command grammar/modal focus, shortcut scopes and keyboard walkthroughs.
- Verified Nightfly palette from user's reference/download; saved a transcription.
- Added SSH_KEY_STORAGE.md; aligned architecture/schema/milestones and six memory
  files while labeling current Keychain-only parser and new file-key target distinctly.
- Captured ten native window states as original JPEGs (2320×1560), including real
  user workspace and disposable fixture; no synthetic redesigned app screenshots.
- User workspace /Users/vm/Documents/1.ghosh was opened/inspected only, remained
  unchanged/untrusted and was reopened afterward. Fixture creation added a recent
  entry; fixture used for draft/conflict/trust-cancel/populated-inventory captures.
- No SSH keys read/created/imported/deleted, no remote connections, no Keychain use,
  no actual execution, and no Base44 upload or external message.
- Added offline gallery, capture provenance/manifest/checksums and portable ZIP in
  docs/base44/. Gallery uses requested palette as review decoration only.
- Checks: 12 edited Markdown docs/link/fence/whitespace checks; all 76 original
  screen IDs + nine key + twelve playbook views; palette TOML; ten JPEG signatures,
  dimensions/size and gallery targets/descriptions; archive CRC/content checks.
- Browser preview unavailable: local file URL blocked by browser policy; no
  alternative browser route attempted. Static gallery checks passed.
- Runtime source and generated schemas unchanged; no rebuild required. In-app
  local/remote terminal tests remain pending because no terminal backend exists.
- Next: Base44 design handoff, then narrow reviewed file-key/SSH/terminal plans;
  documentation acceptance does not mean M1.2/M2 functionality is implemented.

## 2026-10-18 — Live Base44 inspection and native implementation plan

- User requested a sequential plan to reproduce the reference app UI/interactions
  and implement real functionality. Used requested in-app Browser; all 14 sidebar
  destinations, command catalog, settings categories and principal forms inspected.
- Recorded 32 interaction groups, 24 commands, settings coverage and unexercised
  controls in base44/reference-app/INTERACTION_INVENTORY.md; observed working
  command shortcuts/filter selection alongside simulated terminal/check/apply,
  placeholder modules/settings and incorrect run-history identity mapping.
- Saved 33 original reference JPEGs, gallery/index, capture manifest, SHA-256
  checksums and command/settings accessibility evidence. These are prototype
  observations, not native backend test results. Earlier native capture ZIP unchanged.
- Added IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md: 14 dependency-ordered phases,
  concrete file ownership, contracts/migration, all reference surfaces and real
  service acceptance; keyboard/Nightfly, no-Keychain SSH keys and actual local/SSH
  PTY tests retained. Original workspace untouched; no credentials uploaded.
- Verified new/updated document links/fences/whitespace, required plan headings,
  REF coverage, screenshot manifest/dimensions/checksums and gallery coverage.
- Runtime source/config/dependencies unchanged; builds/runtime tests not repeated
  for this documentation task. No commits, deployment or Base44 project changes.
- Next: user review/approval of the proposed implementation plan, then execute
  bounded phases sequentially with evidence; implementation has not begun.

## 2026-10-18 — UI polish and tagged-build foundation

- Approved continuation executed sequentially. All nine text selectors now use a
  shared Nightfly combobox/listbox: line-height 1.7, taller rows, padded SVG arrow,
  selected check, keyboard/typeahead, disabled options and bounded popup placement.
- Native inspection found macOS button-click focus behavior; explicit trigger
  focus was added. Native Tab/Return/Escape opens/cancels without changing values.
- Pinned Lucide SVGs on implemented controls, SVG Shift, Activity/Inbox actions,
  aligned checkbox/dot/Node cells and mixed visible-row select-all. Neutral dots
  deliberately mean configured/unmonitored. Cmd+1–9 follows displayed tabs;
  Cmd+W retains guards; Cmd+Q remains the platform quit command.
- Added native Nightfly background and shrinkable/wrapping/ellipsis layouts;
  browser coverage includes 360/820px and 200% zoom. Continuous native resizing
  with live terminal output still needs observation; no claim of eliminating every
  platform compositor transient. Current native window is fullscreen/user-opened.
- Linux legacy-secret adapter fails closed for all operations; no plaintext secret
  fallback. File SSH key storage remains separate. Native Linux CI is pending.
- Tag-only version guards, sequential macOS arm64/Linux x86_64 workflow, archive/
  checksum scripts and upstream icon licenses added. Deno is documented as an
  optional future SSH-invoked runtime; no persistent server agent/runtime install.
- Screenshot 23 is the actual current native dropdown; README includes it, build
  instructions, keyboard use and limitations. Original workspace not modified.
- Checks: strict frontend build, 18 UI cases, release-profile clippy, 63 Rust
  tests and generated-contract drift check pass. Two environment-dependent tests
  remain opt-in (native Keychain and live SSH); prior authorized SSH evidence is
  retained. Final format, script syntax and local doc links pass.
- Release script built the final macOS arm64 app/archive; executable architecture,
  minimum OS 14.0 metadata, archive contents and SHA-256 verification pass.
  It supports native Rust installations with or without rustup. Invalid version
  tags are rejected. Linux native build and interactive acceptance remain pending.
- Staged private-key pattern matches inspected: parser header literals and
  generated deliberately invalid test data only; no real key file included.
- Publication: reviewed foundation prepared for main and annotated v0.1.0;
  observe the tag-triggered workflow before claiming remote artifact availability.
  Managed SSH PTY/playbooks and other future services remain unfinished.

## 2026-10-18 — Initial publication and tagged CI trigger check

- Published main commit dec9514 and annotated v0.1.0; both remote refs verified.
- GitHub registered the workflow as active; Actions permissions are enabled/all.
  No Actions run or Actions check suite appeared for the initial atomic push.
  GitHub service status reported no unresolved incident; cause is unconfirmed.
- Preserve v0.1.0. Prepare v0.1.1 metadata and push its main commit before creating
  the new tag, so workflow registration precedes that tag event. No branch/PR
  trigger or manual-dispatch workaround added. Remote artifacts remain unverified.
- v0.1.1 metadata, locked Cargo metadata and final native packaging/checksum pass.
  Published commit 6cccd52 to main, then pushed annotated v0.1.1 separately.
  [Tagged run 39129424692](https://github.com/gubnota/ghosh/actions/runs/39129424692)
  started successfully for that SHA. Architecture/version, dependency install and
  browser setup pass; macOS common checks are running, Linux waits on macOS.
  This confirms the trigger, not successful remote artifacts or Linux acceptance.
