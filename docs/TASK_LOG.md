# Task log

## 2026-10-19 — Approved operational supplement, lifecycle slice implemented

- User approved the operational supplement with “go”; continue sequentially.
- Added atomic bounded workspaces.json MRU/last-open state with legacy migration,
  explicit-open startup priority, native close/release and remove/clear history.
- Added searchable workspace switcher and palette actions; Cmd+[/] cycles tabs,
  Cmd+Shift+W closes workspace with draft/session guards; Cmd+W still closes a tab.
- Schema 1/2 local workspaces default approved, with manual pause retained on reload.
  Newer schemas remain inert/read-only. Disk conflicts and exact SSH grants remain checked.
- Verification: 67 Rust tests pass; two opt-in tests skipped. All 20 UI journeys pass
  (19 in the full run; the corrected dirty-tab locator passed a focused rerun).
  Strict TypeScript and production build pass. Managed SSH PTY/auth is next.

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

## 2026-10-19 — Operational diagnosis and proposed scope supplement

- Inspected focused node/SSH/terminal/command/workspace paths: the node editor
  defaults to agent auth while the service rejects it; managed remote PTY and
  scenario services are absent. Palette click exists, but placeholders are disabled.
  Restore-last/close/history cleanup and bracket tab shortcuts remain missing.
- Read server-first-run compact context and selected playbooks read-only. Cataloged
  seven Ansible entrypoints plus certbot-cloudflare.sh. The shell script forces
  renewal; certs_sync.yml copies the full Certbot tree with Nginx handler commented
  out. No production playbooks, credential files or remote mutations executed.
- Created IMPLEMENTATION_PLAN_OPERATIONAL_WORKFLOWS.md: concrete files/contracts,
  eight sequential usable slices, tests/migrations and default local approval
  proposal. Existing approved plan retained; material supplement approval pending.
- User clarified ComfyUI examples in Russian: MiniMax H3, Wan 2.2, LTX 2.3 and
  NSFW LoRAs. Confirmed primary upstream native H3/Wan and publisher LTX integration;
  plan uses family-specific locked recipes and a user-supplied LoRA asset importer.
  RunPod connection/path and concrete asset URLs remain recipe form inputs.
- Updated context/index, README planning link and proposed ADR-0018 without claiming
  changed runtime behavior. Source/configuration/dependencies and original 1.ghosh
  workspace remain unchanged. Telegram is design only; no messages sent.
- Document structure/links/whitespace and request coverage checked. Runtime tests
  were not repeated for this documentation-only increment.


## 2026-10-19 — Workspace lifecycle and usable direct SSH terminals

- Approved supplement; committed lifecycle/history/default local approval, command
  dispatch and keyboard navigation as 73000b3. Opening remains inert and external
  configuration changes still block effects until validated reload.
- Added core Pty port and shared bounded local/remote manager. Nodes/Terminal/
  Cmd+K now open real reviewed ssh2 PTYs using the existing xterm emulator.
- Direct file-key, SSH-agent and ephemeral schema-2 password authentication share
  the native single-use review. Encrypted file keys resolve only to native bytes
  and unlock per call. No passphrase/password storage, Keychain or agent forwarding.
- Opt-in native tests passed sequentially on debian.orb.local and node3.dabaga.cc:
  authentication, TTY input, 41×111 resize, acknowledged output replay, ANSI/UTF-8,
  exit status 7, single-use ticket rejection and session cleanup. OrbStack initially
  was stopped; starting it restored Debian DNS/reachability.
- Disposable ssh-agent and encrypted temporary copy of the already authorized key
  both passed the same node3 PTY checks. Original key/normal agent/server unchanged;
  no production file writes or software installation. Live password not tested.
- Checks: 67 Rust tests pass, 2 opt-in skipped in standard suite; Clippy, formatting,
  generated contract freshness and frontend build pass; all 22 single-worker UI
  checks pass, including node Terminal and clickable command dispatch.
- Persistent host enrollment, explicit agent identity choice, keyboard-interactive,
  jumps, exec/SFTP and route-loss recovery remain.
  Scenario/Ansible/certificate/timer/monitoring/Comfy/browser/scene services remain
  pending; direct terminal completion does not imply these are implemented.

- Native release managed PTYs: Debian TTY/ANSI/Unicode paste, Ctrl+C, Vim edit/
  save/exit and two-node split resizing to 21×48 PASS. Only a disposable Debian
  `/tmp/ghosh-managed-pty-smoke-20261019` file was written. Evidence 24–25
  and README now show actual managed sessions.
- Fixed a focus race after opening the second SSH session; all 22 UI checks pass
  with immediate input targeting the new session. UI version comes from package.json.
- Rebuilt arm64 release PASS (11.85 MiB). Native focus regression PASS: opened
  Debian then node3 from Terminal; immediate keyboard input ran in node3 without
  refocusing. Footer reports the package version (0.1.1).

## 2026-10-19 — Immediate SSH and scenario/script authoring

- User requested no SSH confirmation. Node Test/Terminal now authenticate directly;
  only required password/passphrase input remains. Native single-use tickets still
  pin the handshake key and reject changed known keys. New keys are accepted per
  connection without enrollment. StrictMode/busy guards prevent duplicate opens.
- Playbooks/Scripts now have catalogs, source editors, paste import, six examples,
  native validation, revision-bound save, exact-source reopen and recoverable remove.
  Source limits are 256 KiB. YAML task/handler structure and Ansible link paths are
  validated; script validation checks bounds, not interpreter syntax. Opening an
  Ansible link never reads/copies the external project or its credentials.
- Run/Stop, targets, interpreter checks, Ansible Runner, durable runs, certificates,
  timers, monitoring, ComfyUI, browser sessions/recordings and scene automation are
  still pending. FEATURE_STATUS.md distinguishes working features from missing ones.
- Checks: 72 Rust tests pass, 2 opt-in skipped; Clippy/format, generated-contract
  freshness, strict frontend build and all 24 single-worker UI checks pass.
  Native arm64 release 0.1.2 builds successfully (12.13 MiB).
- Native create/validate/save/reopen passed in the disposable workspace, preserving
  source. Immediate Test passed on Debian (109 ms) and node3 (1,389 ms); Debian
  and node3 Terminals opened directly with input focused. No remote command was sent by this
  smoke check. Original 1.ghosh was not edited. README captures 26–28 show this build;
  older SSH approval screenshots are explicitly historical.
- Reviewed previous tag CI: v0.1.1 macOS checks/build/artifact passed; Linux failed
  during common checks before packaging (exit 101). Public diagnostics do not give
  the Rust error; detailed logs require working GitHub authorization (current token
  returned HTTP 401). Linux artifact/desktop acceptance is not yet claimed.
- Release script repackaged 0.1.2; local macOS archive SHA-256 verification passed.
  Published main 7b8a81e and annotated v0.1.2; tag-triggered
  [CI run 39260833453](https://github.com/gubnota/ghosh/actions/runs/39260833453)
  is in progress. Remote build/artifact success is not yet claimed.
- Closed the two test-only SSH sessions and restored `/Users/vm/Documents/1.ghosh`
  in the app. Its files were not modified; restore-last again points to that workspace.

## 2026-10-19 — Sequential local/SSH scenario execution

- Implemented revision-bound Run/Stop and private durable Runs reports for native
  Check/Apply scenarios and Shell/Python/Deno scripts. One run per workspace,
  up to eight targets, authenticated before sequential execution. SSH starts
  immediately; only required per-call credentials prompt. Original 1.ghosh untouched.
- Check uses exit 0/10 for ok/drift; Apply checks, changes and verifies. Explicit
  bounded retries, skip rules and deduplicated handlers are covered. Local process
  groups are stopped; remote disconnect/timeouts become unknown and never replay.
- Result output is bounded (32 KiB/phase, 256 KiB/run), control-filtered and literal
  JSON input strings are masked. Reports exclude source/credentials/inputs, use
  private atomic files and reconcile stale running records to unknown on restart.
- Checks: formatting, Clippy, generated contract freshness, strict frontend build,
  80 Rust tests and all 25 single-worker UI tests pass. Two opt-in tests remain
  skipped in the standard suite. The local runtime test covers real Check/Apply,
  verify, handler and repeat-run idempotence, plus bounded output and cancellation.
- Opt-in native acceptance passed sequentially on debian.orb.local and node3:
  file authentication, real PTY, resize/replay/ANSI/UTF-8, exit 7, both exec streams,
  draining beyond output caps and saved Linux-facts scenario to persistent report.
  Only read-only commands ran remotely; no installs or production file writes.
- Built release 0.1.3 macOS arm64 app and archive; SHA-256 verification passed.
  Native UI report `4f66d68b-d765-46d6-b877-27c2bc4cad27` succeeded on Debian,
  local macOS and node3 with exit 0 each. An earlier UI connection attempt failed;
  the later successful report and independent per-host acceptance are recorded.
  Actual screenshots 29–31 are linked in README and the native evidence index.
  Reopened the original `/Users/vm/Documents/1.ghosh` without changing its files.
- v0.1.2 CI: macOS passed and uploaded its artifact; Linux failed at
  scripts/check.sh before packaging (job 118097912909). User then requested
  skipping Linux. Commented out its whole job, retained tag-only macOS CI and
  documented the temporary suspension. Linux root-cause diagnosis is deferred.
- Implementation committed as 30356ac; release 0.1.3 macOS archive contains the
  current README and passed SHA-256 verification. CI configuration parses with
  only macos-arm64 enabled; branch pushes remain excluded from its trigger.
- Ansible Runner, certificate renewal/distribution, durable timers, monitoring,
  ComfyUI installs/downloads, browser sessions/recordings and scene automation
  remain pending. Result bindings and report-history cleanup also remain pending.


## 2026-10-19 — RU/EN, upstream Ansible, script results and ComfyUI recipes

- Clarified Keyboard layout with the user: physical hotkeys in RU/EN, no remapping.
  Physical letters/digits/brackets preserve composition/Alt guards and terminal Ctrl input.
- Added standard Ansible play-list execution through pinned ansible-core 2.20.10 /
  ansible-runner 2.4.3 and explicit local controller Python. Linked project stays in place.
  Original inventory, group_vars, roles, includes and handlers work. No Ansible dialect introduced.
- User clarified simple custom scenarios are also useful: retain schema: 1 check/script/verify
  and add a visible “Simple script with success check” example. No controller needed.
  Standalone Shell/Python/Deno success remains exit-code based; report JSON can be exported.
- Runner reuses fresh native authentication and exact server pins, private temporary files,
  bounded filtered task events and no_log censoring. One invocation per selected node;
  inventory alias must match node ID. External project source is not revision-frozen.
  Full multi-host batch semantics and agent/password-specific Runner live acceptance remain.
- ComfyUI wizard generates ordinary Ansible for existing SSH/GPU nodes: editable paths,
  pinned plugin Git commits, requirements installation, checksum/atomic model downloads,
  GPU/disk/path checks and optional running node-class checks. Family names are labels only.
  No verified model catalog, paid provisioning, automatic restart or GPU inference acceptance.
- Verification: full check.sh passes (81 Rust tests, 3 opt-in skipped, 28 sequential UI journeys;
  formatting, warning-denied Clippy, generated contract freshness and strict frontend build).
  Genuine local opt-in controller test passes Check/Apply/idempotence, includes/no_log/fail/Stop,
  external roles/group_vars/inventory/handlers and private Runner cleanup (9.99 s).
- Genuine read-only Ansible and existing native auth/PTY/exec/scenario acceptance passed
  sequentially on debian.orb.local and node3.dabaga.cc (33.63 s total). No production writes.
  Original /Users/vm/Documents/1.ghosh and private server-first-run inputs were untouched.
- Generated ComfyUI YAML passes upstream Ansible syntax-check. Screenshots 32–34 are
  explicitly browser IPC fixtures, separate from actual native evidence 29–31.
- Timers, certificate orchestration, monitoring/Telegram, browser sessions/recordings and
  scenes remain pending. Linux CI stays commented out.
- Release 0.1.4: native macOS arm64 bundle built; version manifests aligned and
  controller requirements included in the archive. Archive SHA-256 verification
  passed. Main/tag publication is authorized; GitHub CI status is tracked separately.

## 2026-10-19 — Offline script editor and live multi-step runs

- Fixed invalid script ID feedback: explicit rules and an optional corrected ID;
  source survives correction. SSH agent, missing key, changed key and unsafe file
  failures now give distinct recovery instructions without weakening file checks.
- Added bundled offline Monaco, expandable editor, syntax highlighting/completion,
  find, RU/EN physical Cmd+S, plain-text fallback and syntax-only diagnostics.
  Shell uses sh -n, Python uses isolated ast.parse, YAML uses native validation;
  TypeScript uses bundled workers. Diagnostics never execute the edited source.
- Reused schema: 1 tasks/check/shell for multi-step scripts, added a three-step
  file example, and retained standard upstream Ansible. Runs show current target,
  step/phase, bounded live output and primary-step progress. Footer returns to logs
  while navigating. Dynamic Ansible shows completed task counts without a guessed
  total. App exit does not provide durable background execution.
- Native adapters drain stdout/stderr during execution; input masking handles read
  boundaries. Live previews stay in memory; completed phases/final reports retain
  private atomic persistence and backward-compatible optional progress metadata.
- Full check.sh passed: 88 Rust tests, three opt-in skipped, 32 single-worker UI
  journeys, formatting, warning-denied Clippy, contract freshness and frontend build.
  Genuine local Ansible opt-in passed using .cache/ansible-controller/bin/python
  (10.35 s); the initially tried default controller path did not exist locally.
  After final Ansible task-name masking, focused genuine controller acceptance
  passed again (10.73 s); formatting and diff whitespace checks also passed.
- Screenshots 35–36 in README are explicitly browser IPC fixtures; native tests
  separately prove both streams arrive before process exit and counts persist.
- Retesting Debian/node3 was not executed: automatic approval review hit its
  service usage limit. This was not an unsafe-action determination. Prior v0.1.4
  remote acceptance remains historical evidence, not a v0.1.5 retest.
- Version manifests aligned to 0.1.5; native macOS arm64 release built (14.75 MiB app).
  Actual native captures 37–39 document expanded editing, sh -n diagnostics,
  physical Cmd+S saving, Deno completion and bundled TypeScript type diagnostics.
  Local run 0495d9e7-2408-4ed6-b945-3e7cdc48667f streamed stdout/stderr before
  exit, survived navigation to Nodes and finished exit 0, 1/1 steps, 100%.
  Final saved output includes FINISHED. Discarded the temporary Deno draft and
  reopened original 1.ghosh without changing its files. Archive includes current
  README/controller requirements; archive SHA-256 and embedded README verification
  passed. Main/tag publication is authorized; CI status is tracked separately.
  Linux CI remains commented out; only version tags trigger the macOS workflow.
# 2026-10-19 — Default new nodes to inherited file keys

- Fixed schema-2 Add node drafts incorrectly starting with SSH agent authentication.
  Drafts now use file authentication with no node key override, inheriting the
  configured workspace/global system or custom file key through the shared resolver.
- Existing/imported agent nodes remain explicit; schema-1 drafts retain agent mode
  until reviewed file-identity migration. Existing node `1` must be changed to File key
  to inherit a configured default; no user workspace was edited automatically.
- Verified strict frontend production build, all 33 sequential browser journeys
  and three native identity tests (precedence, missing/changed keys and migration).
  Browser coverage checks default inheritance, saved schema/auth/key binding and
  explicit agent switching. No live remote SSH retest or desktop bundle rebuild.

## 2026-10-19 — Notebook scripts, reviewed system identities and usable background runs

- Approved plan: IMPLEMENTATION_PLAN_NOTEBOOK_RUNS.md. Check reports explicitly say
  whether changes are needed; history offers Apply against current saved source.
  Temporary-file native acceptance proves Check leaves the file absent and Apply
  writes/verifies it. The user's ms-script previously ran Check, explaining no file.
- Added saved .steps.yml notebooks: stable ordered cells, Shell/Python/Deno, Run cell
  or all, timeouts, optional Shell success checks and inline bounded results. Each
  cell starts a new process; files persist, kernel state does not. Keyboard save
  uses the same validated revision-bound commit as other script editors.
- Added explicit reviewed file-key setup for existing agent nodes, separate OrbStack
  and system-key bindings, schema migration/backup and no silent agent fallback.
  Existing user workspace, key registry and global default were not changed.
- Added workspace-scoped Remove/Clear finished history, including JSONL; malformed,
  foreign and active IDs are protected, including worker finalization races.
- Added modeless output/Stop/minimize/full-run monitor. Automation no longer blocks
  independent SSH shells; one automation and workspace mutation guards remain.
- Full checks passed: 91 Rust tests, three opt-ins skipped, 37 sequential browser
  journeys, formatting, warning-denied Clippy, contract freshness and production build.
- Genuine native read-only SSH/PTY/exec/scenario and saved Shell/Python notebook
  with success-check/concurrent-shell acceptance passed separately on OrbStack
  (11.81 s) and node3 (28.37 s), with existing per-node file keys and host trust.
  No remote software installation or persistent test files.
- Browser captures 40–41 are fixture evidence; remote tests above use real native
  transport. Native release rebuild and rebuilt-app acceptance recorded below.
- Rebuilt v0.1.5 macOS arm64 app: genuine desktop notebook completed two saved
  Shell/Python cells plus success-check, created/read the temporary expected file,
  and showed live progress/output while an independent local PTY returned its
  sentinel. Native evidence 42–43; original 1.ghosh restored and test shell closed.
- Corrected completed monitor accessibility label after native inspection; two
  focused progress/monitor browser journeys passed again. Release includes this fix.
- Release archive: artifacts/v0.1.5-macos-arm64/ghosh-v0.1.5-macos-arm64.tar.gz.
  Final native build succeeded; SHA256SUMS verified. Final app relaunched into 1.ghosh.
  User workspace/key registry/global defaults unchanged; existing agent nodes need
  reviewed per-node system-key setup. Keep the app open for background execution.
- Graphify source-upload step was rejected by automatic approval review; analysis
  continued locally. No complete graph generation or external upload is claimed.

## 2026-10-19 — Multistep Playbook defaults and transfer

- Approved plan: IMPLEMENTATION_PLAN_PLAYBOOK_DEFAULTS.md and user follow-ups.
  New Playbooks default to ordered Shell/Python/Deno steps; the full editor and
  each complete step can occupy the window. Drag reorder preserves saved step IDs;
  Move up/down provide a keyboard alternative. Offline completion includes keywords
  and function/control-flow constructions alongside syntax diagnostics.
- New documents save in playbooks/<id>.steps.yml. Legacy scripts/<id>.steps.yml
  remain readable/editable in place; duplicate IDs across those folders are rejected.
- System authentication uses the current user's existing SSH file, with OrbStack's
  host-specific key for *.orb.local. Legacy schema-1 Agent/no-socket nodes use this
  policy without registry import, migration or a setup button. Explicit agents,
  custom sockets, registered keys and passwords remain explicit. File identity is
  pinned at review and rechecked at authentication; unsafe/changed keys fail.
- Step 1 displays running activity before any step finishes. Modeless bounded output
  and independent terminals remain usable; finished Runs can be removed/cleared.
- Import and drag/drop open inert drafts; export preserves all saved source. Native
  import bounds file size and rejects directories/symlinks; normal revision-bound
  validation/save handles duplicates. Added successful, failing and timeout examples
  in examples/playbooks, offered directly in the Playbooks catalog.
- User-defined timeout -1 means unlimited for both step code and success check.
  Finite timeouts remain 1..3600; 0 and other negative values are invalid documents.
  Stop still cancels unlimited execution. Timeout/interruption retains Outcome unknown
  because remote/local changes may already have occurred.
- 96 Rust tests pass (three opt-in tests skipped in the ordinary suite). Genuine local
  examples create/verify the expected file, fail with exit 7, time out at one second,
  finish a five-second unlimited step, and cancel a longer unlimited step.
- Agent-disabled native system-key acceptance passed both authorized hosts (27.09 s),
  including OrbStack's saved Shell/Python Playbook and independent PTY. A separate
  node3 run proves its saved Playbook, unlimited Shell step/check and independent
  SSH PTY (31.67 s). No remote installation or persistent remote test files.
- Browser drag initially tested endpoints that scrolled out of view before pointer
  press. Keeping both handles visible verifies a real drag; the complete authoring
  journey now passes, including stable order, -1 persistence and per-step execution.
- Final formatting, Clippy with warnings denied, 96 Rust tests, generated-contract
  freshness and production frontend build pass. All 38 browser workflows pass after
  replacing stale percentage assertions with active-step and indeterminate-progress checks.
- The running desktop app contains an open Playbook draft. Preserve that session;
  rebuilt-app GUI acceptance is pending saving the draft and reopening the new app.
- macOS arm64 v0.1.5 release rebuilt successfully; packaged Ghosh.app is 14.93 MiB.
  Release archive SHA256SUMS verification passes. Current app was not restarted.

## 2026-10-20 — Playbooks, Monitor and keyboard workflows

- Approved plan: IMPLEMENTATION_PLAN_MONITOR_PLAYBOOKS.md and user follow-ups.
  Playbooks exposes only multistep documents; Scripts navigation becomes Monitor.
  Legacy files/reports remain intact. Runs contains executions/history, output,
  Stop, export and cleanup; starting Playbooks stays in Playbooks.
- Fixed the reported Save error: timeout 0 now means unlimited for code and success
  checks; legacy -1 remains accepted. Other negative values receive a safe, specific
  structural error. Syntax diagnostics are advisory. Save/Cmd+S performs atomic,
  revision-bound persistence directly without a separate Validate action or execution.
  Runtime and Timeout controls align; examples use 0.
- Restore example Playbooks creates only missing stable IDs; existing edits and
  collisions remain untouched. Examples remain ordinary editable documents.
- Monitor owns an independent local/SSH connection, bounded log output and actual
  CPU/RAM/disk measurements. Journal, file-tail and macOS log presets remain editable;
  commands require explicit Run. Probes pause while hidden. Stop/disconnect cleans
  up local processes/channels; remote termination remains server-dependent.
- Terminal sessions have X buttons, guarded Cmd+W close, and Cmd+Shift+[ / ] cycling
  with focus restoration. Closing the last session keeps Terminal ready for another.
  Hold Cmd+Option for shortcut help; release, Escape or blur dismisses it. Persistent
  searchable help shares its shortcut definitions.
- Formatting, warnings-denied Clippy, all 99 Rust tests (3 opt-in tests skipped),
  generated contract freshness, production frontend build and all 41 browser
  workflows pass. Native atomic-save coverage proves syntax errors and timeout 0
  persist without executing source. Local Monitor metrics/cancellation pass.
- Agent-disabled system-key authentication and independent Monitor log/CPU/RAM/disk
  acceptance pass on both authorized OrbStack Debian and node3 targets (31.90 s). No key registry
  or user workspace changes; no remote installation.
- Preserve the running desktop app and its open Playbook draft; rebuilt-app GUI
  acceptance requires saving the draft and reopening the updated application.
- macOS arm64 v0.1.5 release rebuilt successfully (Ghosh.app 15.07 MiB).
  Packaged archive SHA256SUMS verification passes. The running app was not restarted.

## 2026-10-22 — Uppercase node IDs

- User approved the scoped node-ID implementation plan. Root cause: the form
  allowed RTX3070, while shared lowercase-only entity validation rejected it.
- Node IDs and filename stems now preserve ASCII case in schemas 1/2. Jump
  references accept the same format and resolve exactly. Other entity IDs remain
  lowercase. No migration or automatic rename; older apps may reject uppercase IDs.
- Core load/preview/commit rejects case-only duplicates with a static, specific
  conflict message, including imports/replacements and wrong-case edit/removal.
  Malformed IDs receive a specific static format error. Form constraints/help
  mirror the core rule, including the 64-character limit and first character.
- Formatting, warnings-denied workspace Clippy, all 103 Rust tests (3 opt-in tests
  skipped), generated-contract freshness and production frontend build pass.
  Regressions cover create/update/import/replace/remove/reopen in both schemas,
  exact-case jumps, invalid IDs and collision attempts leaving saved bytes intact.
- Both targeted browser node workflows pass using installed Chrome via a temporary
  test configuration. Default Playwright browser was absent; its stalled download
  was stopped. Browser fixtures verify form/review behavior, not native persistence.
- macOS arm64 v0.1.5 release rebuilt (Ghosh.app 15.07 MiB). Applied a complete local
  ad-hoc bundle signature after the linker-only signature failed strict verification;
  strict signature verification now passes. Repackaged archive/checksum verification
  passes. The running desktop app and open draft were not restarted or changed.
