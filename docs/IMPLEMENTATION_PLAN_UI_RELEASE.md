# Implementation Plan: UI polish and tagged desktop builds

Date: 2026-10-18
Status: Implemented; local checks pass. Tagged CI/Linux artifacts and native continuous-resize acceptance remain pending.
Related: `IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md` remains approved and in progress.
Execution: sequential work, tests, builds and GitHub jobs; no parallel agents.

## Goal

Match the requested Base44 icon and control details while preserving Nightfly,
monospace typography, keyboard operation and working native services. Fix native
resize presentation and produce tested macOS arm64 and Linux x86_64 desktop
artifacts only for version tags. Commit the complete reviewed repository state
to main, create an annotated version tag and push, as already authorized.

## Behavior before this increment

- React/TypeScript frontend; Tauri 2/Rust desktop backend; version 0.1.0.
- Navigation uses Unicode symbols. Header Jobs and Approvals have working
  destinations/counts but text labels instead of the reference's SVG controls.
- Nodes use a separate checkbox column. The header has no mixed selection state;
  node rows currently have no status dot. Filtering preserves hidden selections.
- Native selects lack explicit arrow spacing. Shift is displayed as a font glyph.
- The shell uses viewport height and JS width thresholds that account for font
  size/zoom. Native window configuration has no explicit background color.
- macOS is the only enabled desktop backend. A non-macOS compile error blocks
  Linux; the current CI runs on every push and pull request and uploads nothing.
- File SSH keys are already separate from macOS Keychain. Linux should retain
  those file workflows; legacy Keychain references cannot resolve there.
- Local PTYs and direct managed SSH authentication work. Managed remote PTYs,
  remote execution and playbooks remain later phases of the approved plan.
- Almost all project files are untracked; main currently contains only the
  initial commit. Publication must include the existing implementation, not
  merely this increment. Preserve the user's existing .gitignore edits.

## Proposed approach

### Reference fidelity and keyboard behavior

Use pinned `lucide-react` static imports, matching the SVG family observed in the
reference browser. Add a small typed icon registry rather than Unicode stand-ins.
Include the upstream ISC notice. Icons supplement accessible names; focus targets,
keyboard dispatch, button names and approval logic stay attached to real actions.

Verified main-screen icon inventory from Base44:

| Surface | Icons |
| --- | --- |
| Shell/navigation | Activity, Inbox, LayoutDashboard, ChevronRight, Server, BookText, History, KeyRound, Globe, Route, ShieldCheck, FileCode, MonitorPlay, Cpu, Plug, Settings, X |
| Overview/welcome | FolderOpen, Plus, ArrowUpRight, Clock, TriangleAlert, Trash2, FileWarning |
| Nodes | Plus, Search, Filter, Terminal, Play, Ellipsis |
| Playbooks | Upload, Plus, FileCode2, Clock, Play |
| Runs | Search, RotateCcw, Download |
| SSH Keys | Upload, Plus, Copy, Star, Trash2, FileWarning, ShieldAlert |
| Browser Profiles | Plus, ArrowLeft, ArrowRight, RotateCw, Lock, ShieldAlert |
| Tunnels | Plus, Square, RotateCw |
| Certificates | Plus, RefreshCw, TriangleAlert |
| Scripts | ShieldAlert, Save |
| Recordings | Plus, EyeOff, Play, Pencil, Download |
| Recipes | Plus, HardDrive, Download, CircleCheck, Play |
| MCP | Lock, Play, ShieldAlert |
| Settings | HardDrive, Terminal, Bell, Info, Keyboard |

Recheck relevant reference dialogs while implementing. Use icons on their
corresponding implemented controls and existing feature placeholders; retain
honest unavailable states where service implementation is pending. Activity opens
actual background jobs; Inbox opens actual approvals with live counts.

Render Shift using a consistent outlined SVG in a shared shortcut component, with
an accessible textual shortcut. All text selectors use one themed, accessible combobox and top-layer listbox,
with taller rows, padded SVG arrows, keyboard selection/typeahead, disabled
options, Escape/Tab behavior and viewport-bounded menus, as requested. Add
Cmd+1–9 for displayed tabs; retain guarded Cmd+W and ordinary Cmd+Q behavior.
Prevent text overflow at small sizes and 200% interface zoom.

Combine each node checkbox, status dot and name into one aligned Node cell; put
select-all next to NODE in the header. Use native checkbox mixed state. Select-all
acts on visible rows and retains hidden selections. Dots indicate configured /
unverified state unless actual connection evidence exists; do not imply live
monitoring or turn successful authentication into a permanent online status.

### Resize presentation

Set the native window/webview background to Nightfly using the pinned Tauri
version's supported configuration. Make HTML, body, root and shell fill the
available area with shrinkable grid children. Reproduce continuous resize with
a terminal open and at 100%/200% zoom; inspect layout/terminal fit work before
changing the existing scaled breakpoints. Remove avoidable resize work only where
measurement identifies it. Browser screenshots alone cannot validate native
white flashes; capture native resize evidence as well.

### Deno and remote agents

Recommend optional Deno for future JS/TS jobs over authenticated SSH. Keep it
separate from the approved local embedded deno_core scripting API. No persistent
Ghosh server agent is needed for the current SSH design. An optional short-lived
compiled helper can be considered later for structured operations that justify it.

Future Deno execution must use a pinned runtime, reviewed script/dependencies,
noninteractive permission flags, explicit file/network/environment access,
timeouts, cancellation and bounded output. Do not grant --allow-all implicitly.
Subprocess/FFI access escapes Deno's restrictions and needs explicit treatment;
initial static imports also need dependency controls. Do not silently install a
runtime on either test node. This increment documents the decision; it does not
pretend the unfinished remote executor is available.

### Linux foundation and artifacts

Bring forward only the Linux desktop build foundation from M8. Replace the
non-macOS compile block with a Linux adapter that reports SecretUnavailable for
legacy Keychain operations. No plaintext secret fallback; file SSH keys retain
their existing behavior. Preserve the macOS implementation. Test the Linux
adapter and build actual Linux code in a native x86_64 runner.

Use pinned Rust and lockfiles. Select native targets in the release script rather
than requiring Darwin components in the global toolchain configuration. Build
an unsigned macOS arm64 .app archive and a Linux x86_64 .deb plus binary archive.
Linux runtime WebKitGTK/GTK dependencies must be documented; its binary archive
is not a promise of a statically linked standalone executable.

Replace CI triggers with push tags matching v*. Reject tags whose semantic
version does not match package.json, workspace Cargo.toml and tauri.conf.json.
No branch, PR, schedule or manual workflow trigger. Run macOS then Linux with a
job dependency, one Rust build job/test thread and one browser test worker.
Use macos-14 arm64 and ubuntu-24.04 x86_64, asserting actual runner architecture.
Install the documented Tauri Linux build prerequisites and use platform-specific
bundle arguments. Upload platform/version/SHA-named artifacts and SHA-256 sums.
Keep workflow token permissions minimal; no store publication or signing secrets.

## File changes

| File | Change |
| --- | --- |
| `package.json`, `package-lock.json` | Modify: pinned Lucide dependency and reproducible lockfile. |
| `THIRD_PARTY_NOTICES.md` | Create: icon package license attribution. |
| `src/components/Icon.tsx` | Create: typed static imports and shared sizing/accessibility defaults. |
| `src/components/Shortcut.tsx` | Create: SVG Shift and readable shortcut labels. |
| `src/app/commands.ts` | Modify: typed reference icon names instead of glyphs. |
| `src/app/AppShell.tsx` | Modify: navigation/tab/workspace icons and Activity/Inbox header controls. |
| `src/app/CommandPopup.tsx` | Modify: render shortcut SVGs consistently. |
| `src/app/App.tsx` | Modify: existing welcome/placeholder action icons and measured resize handling. |
| `src/features/settings/SettingsView.tsx` | Modify: shortcut rendering and reference category icons. |
| `src/features/nodes/NodesView.tsx` | Modify: combined aligned Node cell, neutral status dot, mixed select-all and matching action SVGs. |
| `src/features/keys/KeysView.tsx` | Modify: reference SVGs on existing key actions. |
| `src/features/workspace/WorkspaceView.tsx` | Modify: matching icons on existing workspace actions. |
| `src/features/terminal/TerminalView.tsx` | Modify: terminal toolbar icons; verify resizing without excessive fitting work. |
| `src/styles.css` | Modify: icon/keycap sizing, checkbox alignment, select arrow inset and full-viewport dark layout. |
| `src-tauri/tauri.conf.json` | Modify: supported native Nightfly background configuration. |
| `src-tauri/src/platform/mod.rs` | Modify: select a Linux adapter without changing macOS behavior. |
| `src-tauri/src/platform/unavailable.rs` | Create: Linux legacy-secret adapter and fail-closed unit tests. |
| `rust-toolchain.toml` | Modify: platform-neutral pinned toolchain components. |
| `scripts/verify-release.mjs and scripts/check.sh` | Create: version/tag checks and sequential common verification, including generated-contract drift. |
| `scripts/build-release.sh` | Create: platform checks, native target/bundle build, archives and checksums. |
| `.github/workflows/ci.yml` | Modify: tag-only trigger, sequential native build jobs and artifact upload. |
| `tests/ui/shell.spec.ts` | Modify: meaningful header-action, mixed/filtered selection, keyboard and resize regressions. |
| `README.md` | Modify: current screenshots, build/download/run instructions, Linux requirements and feature limitations. |
| `docs/evidence/ui-shell/` | Add reviewed native screenshot evidence with no private key material. |
| `docs/PROJECT_CONTEXT.md`, `docs/ARCHITECTURE_INDEX.md`, `docs/MODULE_OWNERSHIP.md`, `docs/API_CONTRACTS.md`, `docs/DECISIONS.md`, `docs/TASK_LOG.md` | Modify after implementation: concise actual state, Linux boundary, release commands, verification results. |
| `docs/IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md` | Modify after approval: record this bounded Linux build exception; preserve remaining feature phases. |

No files need deletion. No workspace schema or IPC DTO change is expected.

## Implementation steps

1. Add pinned Lucide imports, attribution and shared icon/shortcut components;
   update commands, shell and existing controls using the audited inventory.
   Verify accessible labels and keyboard command routes before moving on.
2. Fix the Nodes layout and native mixed-state checkbox. Add filtered selection
   tests covering Space, partial selection, select-all and deselect-all.
3. Apply shared select styling and SVG Shift. Check every existing select at
   100% and 200% zoom, including dialogs, using keyboard-only interaction.
4. Fix native backgrounds and viewport layout; profile actual resize behavior.
   Verify continuous native resize with local terminal activity and retain proof.
5. Add the Linux fail-closed legacy-secret adapter and its tests. Keep SSH key
   files independent of it and preserve the working macOS secret boundary.
6. Add release scripts and replace workflow triggers/jobs. Validate tag/version
   guards locally; run common verification and the macOS bundle build serially.
7. Exercise non-mutating connection/terminal smoke checks on debian.orb.local
   and node3.dabaga.cc using existing credentials/host review. Update screenshots,
   README and compact context documents with observed results and limitations.
8. Review the complete pending tree for ignored data, secrets, generated output
   and license notices. Preserve docs/ghosh-prompt.md exclusion and the user's
   /Users/vm/Documents/1.ghosh workspace. Confirm remote main/tag state; commit
   all intended non-ignored changes on main, then create annotated v0.1.0 if
   unused and push main plus tag without force. No repeated publication approval.
9. Observe the tag-triggered macOS job then Linux job, verify downloadable
   artifacts/checksums and record their links. If CI fails, fix with a new commit
   and version tag; do not silently move an already published tag.

## Verification

Run sequentially, with locked dependencies:

- npm ci; npm run typecheck; npm run test:ui; npm run build.
- cargo fmt --all -- --check.
- cargo clippy --locked --workspace --all-targets -- -D warnings.
- cargo test --locked --workspace; explain environment-dependent ignored tests.
- cargo run --locked -p ghosh-core --example export_contracts -- --check.
- Native macOS build via the release script; check executable arm64 architecture,
  minimum OS metadata, archive contents and SHA-256 checksums.
- GitHub Linux runner: same applicable Rust/frontend checks, x86_64 executable
  verification, .deb dependency/content checks and a bounded Xvfb startup smoke
  test with an isolated home. Report native desktop interaction separately from
  headless startup; do not label an ARM OrbStack build x86_64 evidence.
- Native keyboard walkthrough: command popup, Shift display, header jobs/inbox,
  node selection, selectors, continuous resize, local PTY input/resize and both
  previously authorized SSH hosts. Do not apply remote system changes.
- Inspect all README image links and screenshots. Confirm ordinary main pushes
  do not launch automation and the version tag launches the two jobs serially.

## Risks / edge cases

- Native white flashes can come from WebKit painting; dark CSS alone may be
  insufficient. Do not claim resolution without observing native continuous resize.
- Bringing Linux forward changes the previous platform scope. Its first artifact
  is a desktop foundation, not full M8 parity or completion of remote playbooks.
- Linux legacy Keychain references must fail explicitly, never fall back to disk.
- macOS archives are unsigned; describe launch requirements accurately. Signing
  and notarization need separate credentials and are not silently assumed.
- A release includes the existing untracked implementation; review it before
  staging and exclude private workspaces, keys, build output and machine paths.
- A published tag is immutable in this workflow; corrective releases use new tags.

## Out of scope

Persistent remote agents; remote Deno executor implementation; Windows; automatic
runtime deployment; completing deferred playbook/browser/tunnel/MCP services;
Apple signing/notarization; automatic updates; background schedulers; app stores.

## Sources

- Reference inspected interactively: https://ghosh-herd-os.base44.app/
- Deno permission model: https://docs.deno.com/runtime/fundamentals/security/
- Lucide React: https://lucide.dev/guide/react/
- Tauri CI/Linux prerequisites: https://v2.tauri.app/distribute/pipelines/github/
- Tauri window configuration: https://v2.tauri.app/reference/config/
- GitHub runners: https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- Workflow tag filters: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
