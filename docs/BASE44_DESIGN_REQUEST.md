# Base44 request: Ghosh, a keyboard-first remote operations console

Prepared 2026-10-18. Copy everything below the divider into Base44 and attach the
current-screen JPEGs listed in [the screenshot guide](base44/README.md).
These are requirements for a future UI, not a claim that SSH/playbooks exist today.

---

Design a polished, interactive desktop product prototype called **Ghosh**
(Bengali for cowherd). It herds remote servers, isolated browser profiles, network
routes, certificates, terminals, automation scripts and GPU/ComfyUI installations
from one window. Design the complete product, including every screen, sheet,
state and transition described here. The result is a desktop application
prototype for later implementation in Tauri 2 + React/TypeScript, not a marketing
website. Use realistic local mock data. No live infrastructure connections,
credentials, payments, login service or cloud database are needed. All simulated
results must be clearly labeled; never imply that a mock connection is live.

## Product priority and current gaps

The current native app implements workspace create/open/rename/reload/trust and
read-only node inventory. It has no Add node, playbook authoring, SSH key manager,
remote execution or working terminal. The attached ten screenshots document that
baseline, including error/confirmation states. Do not copy their visual styling.
The user opened `/Users/vm/Documents/1.ghosh` and could not find how to create a
scenario and run it on remote nodes. Solve that problem before polishing secondary
features. Use **Playbooks** as the main label; introduce “scenarios” as a synonym
once. Declarative YAML playbooks and advanced JS/Python scripts are distinct.

First-use journey: choose an SSH key → add a node → create a playbook → select
nodes → check/plan → approve → run → inspect per-node results. Keep Add node,
Create playbook and Import playbook visible above the fold and in the command
popup. A user should understand the next step within ten seconds. An empty
workspace must offer working setup actions rather than a passive inventory notice.

## Mandatory visual direction: Nightfly, terminal-like, readable

Use the user's [Nightfly Default reference](https://terminalcolors.com/themes/nightfly/default/).
The palette below was checked against that page's
[Alacritty download](https://terminalcolors.com/downloads/alacritty/nightfly-default.toml)
on 2026-10-18; a local transcription is included with the handoff.

| Role | Nightfly value |
| --- | --- |
| Canvas | `#011627` |
| Primary text | `#bdc1c6` |
| Bright text | `#d6deeb` |
| Separators / subdued surface | `#1d3b53` |
| Muted text candidate | `#7c8f8f` |
| Primary action / focus | `#82aaff` |
| Success | `#a1cd5e` |
| Warning | `#e7d37a` |
| Error | `#fc514e` |
| Secondary accent | `#c792ea` |
| Active target accent | `#7fdbca` |
| Selection | background `#b2ceee`, text `#080808` |
| Terminal cursor | `#9ca1aa`, cursor text `#080808` |

Semantic usage is a Ghosh design choice. Check actual foreground/background
contrast in every state; do not use the separator color for ordinary text or
assume ANSI colors guarantee accessibility. Dark Nightfly is the default and
required review theme. A later light variant must preserve the same interaction
model; it is secondary to this request.

Make the app feel like an excellent terminal workspace: prompt rows, tight tables,
plain panels, thin rules, a restrained status line and clear task output. Prefer
`ghosh >` for app commands and `$` only inside actual shell sessions. Use minimal
corners (2–6px), little shadow and no marketing hero, giant dashboard cards,
illustrations, gradients, fake scanlines, glow or CRT effects. Keep color purposeful.

Typography is a priority: the current font treatment was explicitly rejected.
Use a clean, readable monospace UI throughout, including navigation and forms:
`"SFMono-Regular", Menlo, Monaco, Consolas, monospace`. A properly licensed bundled
JetBrains Mono is an acceptable alternative if it is included offline; choose one
family consistently. Font size 14px for UI/body, 13px minimum for secondary text,
14px editor/terminal, 20–24px page titles. Body weight 400, selected labels 500,
headings at most 600; line height 1.45–1.6. Normal letter spacing, sentence case,
no artificial condensed text, tiny all-caps labels, negative tracking or thin gray
text. Disable programming ligatures by default. Provide font size and UI zoom
controls; test 100%, 125%, 150% and 200% without losing actions or information.

Desktop first: design at 1440×960, verify 1280×800 and 1920×1200. Smaller widths
collapse navigation into a keyboard-accessible switcher. Respect macOS traffic
lights. Use comfortable row heights and visible keyboard focus; density must not
come from unreadably small text. Respect reduced motion and accessible contrast.
Status meaning must use text/icon as well as color. No network font dependency.

## Keyboard-only operation and command popup — mandatory

Every feature, form, table, tab, inspector, file picker, editor, modal, error
recovery and confirmation must work without a mouse. This includes task reordering,
node multiselect, route choice, log inspection, pane resizing, screenshots/data
export and approval decisions. Never make drag, hover, right-click or an icon with
no accessible name the only route to an action. Pointer support is welcome.

The main entry point is a **modal command popup**, opened with **⌘K**. Show a
persistent `ghosh >` / `⌘K` trigger. Also support ⌘⇧P as a discoverable alternate.
Typing `>` at a non-editable app navigation surface may open command mode, but
must never intercept typing inside fields, editors, terminals or browser pages.

The popup has a prompt, fuzzy suggestions, category, short explanation, target
context and shortcut hints. Match natural phrases such as “new scenario” to
`playbook new`. Arrow keys select, Enter opens the selected action, Esc goes back
one level then closes; ⌘K toggles. Tab accepts a completion only while focus is in
the command input with a suggested token; elsewhere Tab has normal focus behavior.
An empty query shows frequent actions and recent non-sensitive destinations.
No secret arguments or history. Missing prerequisites offer an action to fix them.

| Typed app command | Result |
| --- | --- |
| `workspace open` / `workspace new` | Workspace picker / creation modal |
| `node add` | Focused add-node modal |
| `node edit edge1` / `node test edge1` | Edit form / reviewed connection test |
| `key use-system` | Choose an existing local SSH key file |
| `key import` / `key generate` | Custom key workflow / local generation form |
| `playbook new` / `playbook import` | Creation modal / import review |
| `playbook edit install-nginx` | Open editor tab |
| `run plan install-nginx --target edge1` | Resolve target and show plan |
| `run apply install-nginx --target edge1` | Fresh plan and approval, then run |
| `terminal open edge1` / `terminal open local` | Real node / local terminal tab after required review |
| `route switch` / `profile open` | Scoped selection modal |
| `jobs` / `approvals` / `settings` / `help keys` | Corresponding panel |

This is an application action language, not a shell: no pipes, substitution,
redirection, arbitrary executables or automatic evaluation. Optional arguments
prefill ordinary fields. Unknown commands explain syntax and suggest actions.
Sensitive or mutating commands never execute just by matching a suggestion.
`run apply` always opens a plan review; Enter cannot bypass trust or approval.
Provide suggestions rather than requiring users to memorize flags.

Action popup flow: command → small focused form → validation → review when
needed → explicit submit → result/destination. Prefill workspace/node context,
show inherited values, focus the first editable field and preserve partially
entered forms while drilling into a picker. Submitting a form must not silently
run a playbook. Distinguish Create, Save, Check, Plan and Apply.

| Shortcut / key | Behavior and scope |
| --- | --- |
| ⌘K, ⌘⇧P | Command popup; configurable app-reserved chord |
| Tab / ⇧Tab | Logical forward/back focus; focus trapped within a modal |
| ↑ / ↓, Home / End | List/table/tree navigation with visible active item |
| Space | Toggle checkbox or selected table row; never run a dangerous action |
| ⇧↑ / ⇧↓ | Extend table row selection; selected count announced |
| Enter | Activate focused item/default safe form action; not approval by accident |
| Esc | Back/close popup or leave local mode; never cancel a remote run silently |
| ⌘S | Explicit Save in an editable document; show conflict/read-only reason |
| ⌘F | Search current table/editor/log; terminal shortcut policy shown |
| ⌃Tab / ⌃⇧Tab | Next/previous app tab through the shell focus scope |
| ⌘⇧E | Move focus to navigation; return to content with focus command |
| ⌘W | Close current app tab with unsaved/process guard, not whole workspace |
| ⌘+ / ⌘− / ⌘0 | UI zoom in/out/reset outside embedded-content ownership |
| Menu key / ⇧F10 | Open row context actions; same actions exist in command popup |

Offer commands for focus main/editor/output/terminal, next pane, resize pane,
move tab left/right, move task up/down, show row actions, select targets and cancel
run. Thus every operation has an accessible route even if its direct shortcut is
unassigned. Supply a searchable, remappable shortcuts view with collision checks.
Use macOS symbols and labels; show Ctrl equivalents for a later cross-platform UI.

Focus is scoped: shell, modal, editor, terminal or browser. Reserve the global
command chord at the native shell level; pass ordinary typing, Ctrl-C and editor
shortcuts to the focused content. ⌘W/zoom/tab keys need an explicit ownership
policy for embedded content and a visible escape-to-shell route. Do not swallow
terminal input or browser form keys. A web prototype may simulate this policy;
document native dispatch requirements rather than claiming web handlers solve it.

On modal open, announce its title and put focus in the appropriate input. Background
content is inert. Close returns focus to the invoking control, or its logical
replacement if removed. Pickers return focus and retained values to their parent
form. Approval sheets focus Cancel/Deny first, not Apply; require explicit focus
and activation of the affirmative control. Busy state, validation summaries and
async results are announced without stealing focus. Use actual dialog semantics,
labels, roving list focus and screen-reader row names. Keep active selections
visible on scroll and restore them on return. No keyboard trap outside a modal.

Show a shortcut hints strip such as `↑↓ choose · ↵ open · esc back` inside the
popup. It is instruction text, not decorative pretend terminal output.

## One window, one shared tab strip

Persistent app shell:

- Top title bar: traffic lights, workspace name/switcher, unsaved/conflict status,
  persistent `ghosh >` / ⌘K command trigger, running jobs count and approvals inbox.
- Left sidebar groups: Overview; Nodes; Playbooks; Runs; SSH Keys (global); Browser Profiles;
  Tunnels; Certificates; Scripts; Recordings; ComfyUI Recipes; MCP; Settings.
- One shared tab strip above the main content, containing dashboard/detail,
  browser, terminal and editor tabs together. Tabs reorder, close and pin.
- Browser tabs show favicon, title and a small colored profile badge plus route
  state. Mix Work, Research and Client profiles side by side. Switching profile
  means selecting a tab, never switching windows or opening separate tab bars.
- Main content with optional right inspector; collapsible bottom panel with
  Output / Tasks / Problems / Activity and clear per-run context.
- Status bar: active target/profile/route, workspace save state and background jobs.

A real browser tab will be a native webview positioned beneath the app chrome.
In this prototype, render a convincing local placeholder page with clear sample
content. Do not embed third-party sites or claim that simulated native services are live.
Real SSH/PTY functionality is mandatory in the implemented desktop app; a Base44
web prototype must label its native placeholders. Do not claim real proxy isolation. Native content must leave the tab bar and controls
visible. Design overlay states so implementation can hide native content while
an app modal is shown.

## Sample workspace and scenarios

Workspace `Homelab.ghosh`; five nodes:
`gpu1` (ubuntu, GPU/ComfyUI/prod, VLESS EU), `gpu2` (GPU/staging, direct),
`edge1` (staging/nginx, direct), `edge2` (staging/nginx, direct),
`build-mac` (macOS/brew, direct). Production is represented by gpu1.
Profiles: Work blue/direct, Research violet/VLESS EU, Client amber/SOCKS Lab.
Routes: Direct; VLESS EU up; SOCKS Lab starting; HTTP Edge down.
Certificates: example.com wildcard due in 12 days, api.example.net healthy for
71 days, staging.example.org verification failed. Recipes: Flux Basic installed
on gpu1, SDXL Research planned for gpu2. Use example domains and documentation IPs.
Never display real or realistic credential values: show local SSH key names/public fingerprints or non-SSH secret references and
masked fields. Fixture certificate private-key material must never be present.

Build all screen families below. Sheets/subviews may share a component but must
be individually reachable from the prototype. Each numbered item has distinct
content and working navigation, not a duplicate placeholder.

## 01 — First launch and workspace lifecycle

1. Welcome / recent workspaces: Create, Open package, recent list with path,
   last opened, missing-file state, removable recent entry, no account requirement.
2. Create workspace sheet: name/location, `.ghosh` package preview, defaults,
   create success and permission/path validation failure.
3. Open/import review: schema version, workspace identity, missing local SSH key
   bindings, trust status, unsafe script declarations; open read-only or trust
   explicitly. Opening a package never auto-runs scripts.
4. Workspace overview: node health, active runs, blocked routes, upcoming renewals,
   model downloads, recent activity and useful first-run setup actions.
5. Workspace settings: paths/defaults, recent-workspace management, reveal package,
   local privacy/storage, schema details, close with running jobs choices.
6. Migration review: version change, backup location, preview, progress, failure
   recovery; unknown newer schema opens read-only. No destructive downgrade CTA.
7. External edit conflict: file/field diagnostics, side-by-side local/external
   diff, reload/keep/resolve actions, save error and unsaved indicators.
8. Clone/import profile identity conflict: explain local browser sessions are
   outside package; choose fresh isolated stores or intentional continued identity.

## 02 — Nodes and SSH

9. Node inventory: searchable/filterable table with name, host, labels, OS/GPU,
   route, connection status, last run and row actions. Bulk selection and targeting.
10. Add/edit node sheet: host/port/user, labels, auth mode key/agent/password,
    local SSH key picker (inherited default or explicit override), optional ProxyJump node, route selector, timeouts,
    known_hosts source. Show cycle/reference validation and Test connection.
11. SSH host trust sheet: full hostname/port and fingerprint, source, explicit
    first enrollment; separate alarming changed-key state with no auto-accept.
12. Node detail: Overview / Facts / Files / Runs / Terminal; CPU/GPU/disk facts
    with cached timestamp/refresh, selected route, reachability, reload-cert hook.
13. Connection diagnostic view: connecting/jump/auth/proxy/host verification stages,
    actionable known-host mismatch, agent unavailable and route-down states.
14. Scoped file transfer view: local/remote path, permission/owner, progress,
    checksum and atomic replacement status; cancelled/partial transfer recovery.

## 03 — Playbooks and run history

15. Playbook library: declarative YAML playbooks, target selectors, last result, New/Import/Open.
    Link advanced JS/Python automation to Scripts rather than conflating its safety model.
16. Playbook editor: code + target picker + variables + diagnostics, Plan changes,
    Check, Run, save; check probes must be read-only and trusted.
17. Run plan/diff review: resolved nodes, routes, checks, per-task expected changes,
    unified diff, exact commands, unknown/unavailable diff state, concurrency,
    timeout and clear Check mode label. Apply follows an explicit action.
18. Run monitor: per-node task matrix with ok/changed/skipped/failed, counters,
    live log stream, progress, handlers, queue/lock state, Cancel and elapsed time.
19. Run details/history: filter by node/playbook/status, timeline, per-task diff,
    safe log export, retry failed nodes through a fresh plan.
20. Cancel and partial outcome states: stopping vs stopped, completed changes,
    remote outcome unknown, reconnect/re-check; never claim automatic rollback.

## 04 — Terminal

21. New terminal picker: **Local machine** or any configured node, cwd, shell
    and session label. Every node detail/inventory has Open terminal and the
    `terminal open <node>` command; `terminal open local` is always discoverable.
22. Real terminal tab: interactive terminal emulator backed by a local PTY or
    SSH PTY, input focus, truecolor, scrollback, search, copy, safe OSC-8 links,
    resize and split panes. A log viewer or command/output textbox is insufficient.
23. Terminal session menu: multiple independent sessions per node, reconnect,
    rename, keyboard split/resize, close; active process confirmation, process
    exit and connection-lost view. Reconnect starts a new session unless an
    explicitly configured remote multiplexer supports resumption; do not promise
    automatic process recovery. Terminal lifetime is separate from run logs.
24. Terminal settings: offline font family/size, ligatures capability, scrollback,
    renderer status and keyboard protocol marked experimental when applicable.
    Terminal input/history persistence is off by default.

### Real terminal implementation and local-machine test gate

The shipped native app must use a real terminal emulator (for example an xterm.js
renderer) connected to native services: local shell through a PTY and remote shell
through an authenticated SSH channel with PTY allocation. This is a functional
requirement, not a request for terminal-looking text. Reuse node authentication,
strict host verification, jumps and routes; never fall back to a local shell or a
direct route when remote connection fails. Show `Local` versus `user@host` and
route prominently in the tab/status line, especially before paste or commands.

Support interactive input/output, UTF-8 and wide characters, ANSI/truecolor,
cursor movement, alternate screen, line editing/history, Ctrl-C/Ctrl-D, terminal
resize propagation (rows/columns), bracketed paste, bounded scrollback and output
backpressure. Full-screen programs must work. Handle shell exit, EOF, process
lifecycle, disconnect, reconnect and cancellation without freezing the shell UI.
Never record typed passwords or persist terminal input/history by default; show
that the chosen shell's own history is governed by the user's shell settings.
Display a multiline paste review without silently rewriting commands. Escape
sequences must not gain privileged app IPC or silently write to the clipboard.
Keyboard commands expose copy, paste, search, split, resize and return-to-shell;
terminal Ctrl-C remains an interrupt when terminal input has focus.

Base44 deliverable: reachable local and per-node terminal views and realistic
clearly marked simulation states. Native implementation deliverable: the real
backend and **executed local-machine terminal smoke test**, not a screenshot or
successful standalone shell command. The current M1.1 app cannot pass this yet.

Required native test on the developer's local Mac, inside Ghosh:

1. Open `terminal open local` using only the keyboard; verify a real interactive
   user shell and visibly label the local host/cwd. Record app/OS/shell versions.
2. Type `pwd`, `printf 'GHOSH_LOCAL_PTY_OK\n'` and `stty size`; capture output.
   Resize through both window and keyboard pane controls and verify rows/columns
   change. Test arrows/history, Unicode, paste, scrollback search and copy.
3. Run `sleep 30`, press Ctrl-C and obtain the prompt again. Run a local full-screen
   editor (e.g. `vi`), enter/leave its alternate screen and return without corrupt
   display. Test terminal input and app command-popup focus return independently.
4. Open a second local session/split; verify independent cwd/process state, output
   responsiveness and close-active-process confirmation. Exit one cleanly; retain
   the other. Verify empty/failed startup and font zoom states.
5. For each SSH backend path, open a real permitted test node terminal, confirm
   identity/host pin, repeat resize, interactive editor and interrupt checks,
   simulate disconnection and reconnect. Missing key/route must show a useful
   error. A localhost SSH target is optional only when already configured; do not
   enable Remote Login or modify local SSH settings as a hidden test dependency.

Record expected/actual results and screenshots/logs with secrets redacted. Local
PTY and SSH PTY results are separate; a local pass does not prove remote routing.
Treat this gate as pending until the native terminal exists and tests are run.

## 05 — Browser profiles and mixed tabs

25. Profile library: label/color, UA preset, store identity summary, route, tab
    count, last used, New profile/Open tab/Edit/Clear data.
26. Profile settings: identity/color, UA preset/custom, platform/languages/locale/
    timezone, zoom/viewport, downloads/popups, init scripts and route/kill-switch.
    Store identity is read-only; invalid imported identity shows a validation error.
    Each setting shows native support, best effort or unsupported when relevant.
27. Browser tab: back/forward/reload, address field, profile badge, route status,
    page loading/error, recorder affordance, download indicator, zoom.
28. New tab/profile picker: existing profile or fresh isolated profile, start URL;
    show different-profile tabs together and same-profile tabs sharing sessions.
29. Route change review: list all affected tabs, preserve session storage, explain
    reload/suspend, apply route or cancel. Never silently fall back to direct.
30. Route-down blocked tab: failed route, last health check, Retry tunnel, Change
    route with review; show protection limits in plain language.
31. Clear browsing data sheet: selected profile only, cookies/storage/cache,
    affected tabs, sign-out consequence and destructive action state. Clearing
    data keeps profile identity; show progress and failure/retry before reopening.
32. Downloads and popup request center: progress, save location, allowed/blocked
    status and policy choice; permission decisions scoped to profile/origin.

Required copy: “User-Agent changes are not full fingerprint protection.”
Use “Limited proxy protection” until leak tests have validated the backend; never
claim anti-detect guarantees or universal traffic isolation.

## 06 — Tunnels and routing

33. Tunnel inventory: direct/http/socks5/vless, engine, health, latency, local
    endpoint without credentials, consumers, restart count, start/stop controls.
34. Add/edit tunnel: kind, credential-free endpoint or masked VLESS import to
    Keychain, engine, loopback listener, health URL/interval, restart policy,
    kill-switch explanation. Validate without echoing the imported URI.
35. Tunnel detail: readiness/health timeline, redacted logs, associated tabs/nodes,
    route generation, restart/backoff, crash-loop and auth error states.
36. Stop/restart impact sheet: affected tabs, SSH sessions and runs; blocked
    consumers, restart action, intentional direct-route change as separate review.

## 07 — Certificates

37. Certificate inventory: domains, expiry countdown, provider/method, renewal
    status, per-node deployment summary and failures, Renew due action.
38. Certificate setup/edit: domains, email, Cloudflare token reference, certbot
    versus native ACME capability, renew threshold, target selectors, destination,
    owner/mode, reload hook, TLS host/port/SNI. Validate secret references.
39. Issue/renew workflow: DNS challenge, propagation, order/finalize, local verify,
    failure/cleanup; staging mode visibly different from production.
40. Distribution plan: old/new public fingerprints, per-node staged file pair,
    mode/owner and reload command, expected changes and Apply.
41. Certificate detail: public chain info, fingerprint, notAfter; Sync / Verify;
    per-node transfer, activation, reload and TLS comparison with expected vs served.
    No private-key viewer or copy-private-key action.
42. Renewal scheduler: app-running-only explanation, next check, due jobs,
    missed-on-close state, enable/disable, last attempt and retry.

## 08 — Script editor and runtime permissions

43. Script library/file tree: scripts, playbooks, templates and declarations;
    JS/TS/Python/Node/YAML/TOML/shell/Dockerfile file types, recent runs.
44. Script editor tab: offline highlighting, ghosh API completion, diagnostics,
    split editor/output, runtime picker, node/profile/route targets, Run selection,
    Run file, Stop, save state and unsaved changes guard.
45. Script capability review: requested nodes/profiles/tunnels/paths/secrets/MCP
    servers, source hash/change indicator, allow once/session/saved scope or deny.
    For Python/Node clearly state “Trusted local code: this runtime can access
    your computer through its own libraries.” Do not label it sandboxed.
46. Runtime manager: installed/unavailable JS/Python/Node, version/path/source,
    signed managed runtime status, startup failure and permission diagnostics.
47. Script run detail: structured output, errors with source locations, duration,
    scoped targets, cancellation, output truncation and safe artifact links.

## 09 — Recorder, replay and data

48. Recorder controls in a browser tab: record/pause/stop, elapsed time, step count,
    active profile/tab, password/sensitive-input masking, unsupported frame notice.
49. Recording library: profile, steps, last replay, status, edit/export/replay.
50. Recording editor: numbered editable steps for navigation/click/type/key/scroll,
    selector fallbacks, wait/assert/extract/save, insert/reorder/delete, screenshot
    preview, variable/secret reference editor and diagnostic errors.
51. Replay setup: choose profile/tab, variable input, secret refs, timing/timeout,
    exact changes review and approval. No stored secret plaintext.
52. Replay monitor/report: current step highlight, pause/cancel, step timing,
    assertion diff, selector ambiguity, inaccessible frame/manual step, retry.
53. Extraction/data preview: rows/columns, selector/field mapping, pagination,
    JSON/CSV/SQLite output and workspace-relative path, overwrite review.
54. Export to script: JS editor preview, ghosh page API calls, variables and
    secret references, save/open in editor; no fake CDP/Puppeteer connection.
55. Recording schedules: frequency/timezone, scoped grant, next run, last result,
    app-running-only state, missed runs and expired permissions.

## 10 — ComfyUI recipes

56. Recipe library: install state by node, pinned ComfyUI revision, models/custom
    nodes counts, total size, plan/apply/verify/export actions.
57. Recipe editor: target install/venv paths, Python version, ComfyUI commit, custom
    nodes with pinned commits/requirements, models URL/SHA-256/destination/token
    ref, preflight disk and shared-model symlink policy, post-install hooks.
58. Recipe plan: installed vs expected commits, missing/drift/hash mismatch,
    required/free disk, total download size, changes and unpinned revision errors.
59. Recipe apply/download queue: per-model progress/speed/bytes, parallelism,
    resume/restart state, checksum verification, install steps, Cancel, disk-full,
    auth/rate-limit/hash mismatch failures and partially completed recovery.
60. Recipe verification: model hashes, custom-node revisions, venv requirements,
    missing symlinks, repair plan and second-apply no-change state.
61. Recipe export wizard: select existing node/install, discovery preview, exact
    observed revisions/models, unresolved hashes that block apply, save recipe.

## 11 — MCP server and client

62. MCP server settings: disabled default, read-only toggle, stdio registration
    command, loopback HTTP address, token handled by the configured native credential provider, token rotate,
    Origin allowlist, connection status, clients, limits, errors. Never expose
    actual token text or allow public binding by casual toggle. Missing mandatory
    approvals block startup with the missing tools listed and a repair action.
63. Tool catalog: actual available tools with inputs/results, read-only/mutating
    badge, approval requirement, sample safe call. In read-only mode hide mutators;
    show an explanatory count of capabilities that require write mode.
64. Resources/prompts explorer: scoped manifest/entity/API/run-log resources and
    write_playbook/fix_failed_run/record_to_script prompts, redacted previews and
    untrusted-data labels; read actions do not execute prompts.
65. Agent connection/activity: client identity, negotiated protocol/capabilities,
    tool timeline, progress/cancel, errors/rate limits, subscription state and
    output truncation. Remote/page content carries an “Untrusted source” label.
66. Native-style approval sheet: requester, exact command/tool, targets, route,
    public arguments and diff, secret placeholders; Allow once / Allow for this
    session / Deny. Changed plans invalidate a prior approval. No agent self-approve.
67. Approvals inbox/grants: pending requests, expiry, allowed/denied history,
    scoped session grants and revoke. Denied operation shows no execution effects.
68. MCP client server manager: predeclared command/args or HTTP endpoint, cwd/env,
    headers secret reference, capabilities, connect/disconnect/test, first-use
    approval, changed-config reapproval and supervised process failure states.

## 12 — Settings, security and shared supporting views

69. Settings: Appearance; Workspace defaults; SSH trust; SSH Keys; Other secrets; Runtimes;
    Sidecars; Terminal; Browser capabilities; Notifications; Storage & privacy;
    About/licenses; Keyboard shortcuts.
70. Non-SSH secret reference manager (separate from file-based SSH Keys): reference ID/provider, intended use and missing/
    locked/available state, add/replace/delete through native-style secure entry;
    no secret-value list, reveal button or stored plaintext fixture.
71. Audit viewer: filter by requester/tool/decision/target, public plan digest,
    redacted diff and timestamp, local export with review. Explain local audit
    history without claiming tamper-proof storage.
72. Background jobs center: cert renewals, downloads, runs, tunnel restarts;
    per-job progress, queue/lock/cancel, app-close implications.
73. Modal command interface specified above: full keyboard action coverage,
    typed app commands, fuzzy search, argument completion, focused action forms,
    errors, nested pickers, focus return and the same plan/approval flow.
74. Storage/retention: workspace state/log/artifact sizes, retention controls,
    clear selected data, preview affected files and safe cleanup failure state.
75. Notification center/toasts: completed run, failed route, expiry warning,
    lost connection, approval needed, external-edit conflict, update diagnostic.
    Keep important events accessible after toast dismissal.
76. About and capability diagnostics: app/OS/architecture, backend capabilities,
    local runtime versions, licensing, copy safe diagnostic bundle with secrets
    excluded and a preview of included files.

## Detailed SSH key views — required additions K1–K9

SSH keys use ordinary files, **never macOS Keychain**. “System keys” means existing
keys under the current user's `~/.ssh` (or another user-selected location), reusable
across all their Ghosh workspaces. It does not mean root-owned files or sharing
private keys across macOS accounts. Custom keys may be referenced in place or
copied into Ghosh's user-local managed storage. No cloud credential database.
See [the storage requirements](SSH_KEY_STORAGE.md) for the handoff contract.

- **K1 — SSH Keys library:** System keys / Custom keys, source path, friendly name,
  algorithm, public fingerprint, availability, encrypted-key status and usage.
  Actions: Use existing, Import, Generate, Set global default, View usage, Remove.
  No private key text or passphrase column. Show “Available to all workspaces”.
- **K2 — Use system key:** native local file chooser, optional path entry,
  validation of selected key and public metadata; do not upload it to Base44.
  Explicitly list selected files; do not recursively ingest all of `~/.ssh` or
  run arbitrary `ssh_config` directives. Missing/unsupported/encrypted states.
- **K3 — Import custom key:** choose Reference existing file or Copy into managed
  storage, destination/ownership explanation, collision and permissions states.
  Preserve the original. No multiline private-key paste into a web prototype.
- **K4 — Generate:** local algorithm/default Ed25519, name, destination, optional
  passphrase entry/confirmation, public-key copy/export and manual installation
  guidance. Generation does not imply deployment to a server or working access.
- **K5 — Key detail:** public metadata, source/managed location, referenced by
  workspaces/nodes, last validation, relink, public-key copy and default status.
- **K6 — Assignment:** global default → workspace override → node override;
  show effective key and provenance. Explicit node reference overrides inherited
  defaults; missing explicit key is an error, not fallback to a different identity.
- **K7 — Unlock:** encrypted-file passphrase prompt with optional memory-only
  session retention. Forget on lock/quit; never persist a passphrase or use
  Keychain. Cancel leaves authentication unattempted. Wrong-passphrase recovery.
- **K8 — Remove/relink:** distinguish removing a reference from deleting a
  Ghosh-managed file; show affected users/nodes, require explicit destructive
  confirmation, never delete an externally referenced file. Repair missing paths.
- **K9 — Diagnostics:** unavailable file, unsafe permissions, unsupported format,
  agent missing, wrong passphrase and rejected server auth. Repair is explicit;
  never silently chmod a user's existing files. Key availability is not reachability.

Use restrictive file permissions for managed private files (0600) and managed
folders (0700). Private bytes stay in the native SSH service; the UI receives only
public metadata and opaque references. Keys stay outside `.ghosh` packages,
exports, logs and diagnostic bundles. Agent auth is optional and clearly separate;
agent forwarding is off. Password auth, if offered, is memory-only for this flow.
Host-key verification is a separate server identity review, never a client-key picker.

## Detailed Playbook views — required additions P1–P12

- **P1 — Create:** ⌘K `playbook new`, then name, workspace-relative file, Blank /
  Install package / Deploy config / Manage service / GPU setup template. Explain
  supported target platforms. Create opens the editor and does not run anything.
- **P2 — Targets:** node checkboxes, label selectors with explicit AND/OR,
  exclusions, resolved count/list, OS, route, effective SSH key and connection
  state. Show zero matches, deleted node, unavailable route and untrusted workspace.
  Freeze the resolved list for a plan; target changes invalidate the plan.
- **P3 — Editor:** task list/builder alongside YAML with optional full-width
  source view. Add/reorder/duplicate/disable/delete using keyboard commands.
  Task forms cover package, file/template, service, command, condition, loop,
  notification/handler, privilege escalation and timeout. Display supported
  adapter/platform combinations. Preserve unknown YAML when switching views;
  unsupported constructs remain editable as source and execution is blocked with
  diagnostics, never silently dropped or treated as successful.
- **P4 — Variables:** typed inputs, defaults, required markers and source. Define
  precedence as workspace → playbook → explicit run override. Show secret
  placeholders/references, not plaintext; no secrets in command history or URL.
  Show interpolation errors and a review of effective non-sensitive values.
- **P5 — Validate:** YAML/schema, task adapter availability, references, targets,
  privilege requirements, key/route availability and trust. Separate static
  validation from reviewed remote probes. Invalid docs can be saved as drafts
  but cannot Check/Apply. A failed connection is not a syntax error.
- **P6 — Check/Plan:** readonly supported probes, current vs desired state, diff,
  unknowns, unsupported checks and estimate per node/task. Arbitrary commands
  cannot be proven read-only: never execute a mutating command in Check. An
  unsupported probe is “Unknown / check unsupported”, not “No changes”.
- **P7 — Apply review:** exact playbook revision, targets, routes, identities,
  expected mutations, redacted commands/diffs, privilege, concurrency, timeout,
  production badges and unknown results. Explicit Apply approval, Cancel first.
  Changed playbook/targets/variables/route/identity invalidate stale approval.
- **P8 — Monitor:** per-node/task matrix, queued/checking/awaiting approval/
  running/ok/changed/skipped/failed/cancelling, timing, handlers and redacted
  stdout/stderr. Keyboard opens task details, searches output and changes panes.
- **P9 — Failure/cancel:** partial changes remain visible, unreachable nodes and
  remote outcome unknown; Cancel requests stopping rather than claiming rollback.
  Preserve completed results, offer reconnect/recheck and a fresh repair plan.
- **P10 — History:** saved run configuration/revision, target snapshot, status,
  diffs and safe export; retry failed nodes through a fresh plan/approval. Show a
  second apply with “No changes” only where adapters actually support idempotence.
- **P11 — Import:** local YAML picker, schema preview, target mapping and unknown
  constructs. Describe Ghosh's supported declarative subset. Do not claim complete
  Ansible compatibility, install Ansible implicitly or convert scripts silently.
  Existing JS/Python imports go to advanced Scripts with their own trust review.
- **P12 — Persistence/conflicts:** dirty marker, explicit Save, parser diagnostics,
  external-change comparison, resolve/reload/discard, close guards and readonly
  future schema. Never lose a task when a picker or popup is dismissed.

Use a complete sample `install-nginx` on Linux nodes `edge1` and `edge2` with
fictional documentation IPs: ensure nginx present, deploy a template, notify a
restart handler only on change. Show staging selected and production explicitly
excluded, changed state on first apply and no changes on a second supported
apply. Keep the plan/monitor counts internally consistent; show per-node expected
changes rather than an ambiguous global total. Browser/native placeholders must
say “Simulated” where they show connection or execution results.

## Reusable components and state coverage

Create a coherent token/component system: sidebar item, tabs/profile badges, node
selector, route picker, local SSH key picker, non-SSH secret reference input, status chip, tree/table, unified
diff, log viewer, progress row, operation timeline, diagnostics panel, modal/sheet,
empty state, skeleton, inline validation, tooltip, toast and destructive review.
Separate healthy, starting, degraded, down and unknown statuses. Keep meaning
consistent across browser, SSH, certificates, recipes and MCP.

For each main screen demonstrate: populated, empty/first use, loading, error,
permission denied, disconnected and read-only states as relevant. For long jobs
include queued, awaiting approval, running, cancelling, completed, failed, partial
and outcome-unknown. Explain why an action is disabled and provide a concrete
recovery path. No clickable buttons without a local prototype behavior; simulations
must be visibly distinguishable from real infrastructure results.

## Required interactive walkthroughs

A. With keyboard only: create Homelab → ⌘K key use-system or key import → node add
   edge1 → review host fingerprint → test connection → playbook new → add tasks →
   select targets → check/plan → inspect diff → approve → run → open per-task log.
B. Open Work and Research browser tabs together → switch profiles via tab badges →
   simulate VLESS failure → Research blocks while Work remains usable → retry.
C. Create wildcard certificate → simulated DNS issuance → review node sync →
   simulate one failed reload → inspect expected/served fingerprint → repair plan.
D. Edit a script → choose Python → read trusted-local-code capability sheet →
   deny → verify no run → allow once → simulated output/cancel.
E. Record an extraction → mask sensitive input → edit selector/wait/assert →
   replay in another profile → export JS and CSV preview.
F. Plan Flux Basic on gpu1 → show disk preflight → download interruption/resume →
   SHA-256 mismatch → retry verified artifact → second apply shows no changes.
G. Enable MCP read-only → inspect available tools → switch to write mode → receive
   agent request → inspect native-style approval → deny and inspect audit.
H. External file edit while locally dirty → comparison → resolve → explicit Save;
   close workspace with running jobs and review consequences.

## Output and acceptance

Deliver a navigable prototype with all 76 original screen/state entries plus K1–K9 and P1–P12 represented through
routes, tabs, sheets or subviews, a screen inventory mapped to navigation, design
tokens and reusable components. Provide a screen/state gallery so reviewers can
inspect failure and permission states without waiting for a simulation. Annotate
native integration points (webview/terminal/secure sheet) in handoff notes, keeping
engineering details out of ordinary product copy.

Make Overview, mixed browser tabs, Node detail, Run monitor, Script editor, Recipe
apply and Approval sheet especially polished. Use coherent mock data across all
views; changing a route or run updates its related badges, details and activity.
Preserve a professional desktop density and one-window workflow throughout.

## Extra acceptance: commands, typography, keys and playbooks

P0: deliver a polished Nightfly shell and the complete keyboard-only key → node →
playbook → targets → plan → approval → run → history journey first. P1: retain all
remaining product families as reachable, coherent routes/subviews and complete
these after the core journey. Mark simulated and unavailable features honestly.

Provide a keyboard coverage inventory with screen, action, focus entry, keys,
focus return and tested result. Demonstrate these paths with the pointer untouched:

1. From welcome, open/create workspace, choose a referenced system key, add a node,
   complete host review, create a two-task playbook, select nodes, check, approve,
   inspect per-node output, cancel a run and revisit history.
2. Repeat with custom key import, workspace default and node override; simulate a
   missing key and encrypted key, cancel unlock, retry and relink without data loss.
3. Open `⌘K`, type a natural phrase, select completion, open nested key picker,
   return to a retained form; test no matches, invalid argument and stale context.
4. In an editor, terminal and browser field, ordinary text and Ctrl-C stay with
   the content. Global command chord opens the shell popup and returns focus.
5. Reorder tasks/tabs, resize panes, select multiple nodes, inspect context actions,
   copy a public key, use file chooser/path entry, review/deny approval and recover
   an external edit without drag, hover or mouse clicks.
6. Include Local machine plus per-node terminal views; for native implementation,
   execute and record the local PTY and SSH PTY tests defined above.
7. Validate Tab/Shift-Tab, screen-reader names, zoom, focus contrast, async notices,
   destructive default focus, Esc behavior, shortcut collisions and no focus loss.

Deliver design tokens, documented commands/shortcuts, component states, clickable
prototype, screen/state gallery and native-integration handoff notes. A browser
prototype cannot read local SSH keys or perform native SSH safely: simulate those
boundaries with explicit placeholders. Do not build cloud storage for credentials
or transmit keys, passphrases or screenshot contents to services from the prototype.
Keep key parsing, native file handling and SSH execution for the later Rust app.

Current-screen references are evidence of existing behavior and gaps. They show a
light UI and legacy Keychain wording; those are not the requested Nightfly theme
or SSH storage model. No existing screenshot depicts the requested command popup,
key manager or playbook editor because those features have not been implemented.
