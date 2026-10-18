# Live Base44 reference: UI and interaction inventory

Inspected sequentially on 2026-10-18 through the requested in-app Browser.
Reference: [ghosh-herd-os.base44.app](https://ghosh-herd-os.base44.app/).
Target implementation: [UI/functionality plan](../../IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md).

This is observation of the rendered application, not a source-code export or a
backend audit. All sections use the same `/` URL and in-app tabs. The prototype's
Homelab inventory and statuses are sample data, not the user's local workspace.
No private keys, workspace files or credentials were uploaded. No remote command,
certificate renewal, real terminal session or native approval was executed.

Legend: **Observed** = navigation/state transition exercised; **Visible** = control
or information inspected, effect not exercised; **Placeholder** = explicitly
simulated, static, or produced no visible transition in this inspection.

## Shell, navigation and workspace

| ID | Surface and controls | Observation / target requirement |
| --- | --- | --- |
| REF-01 | Welcome: create/open, first-use actions, recents and missing recent | Observed welcome. Recent removal visible, not exercised. Preserve native inert open, validation and real recents. |
| REF-02 | Shell: brand, workspace switcher, grouped navigation, jobs/approvals badges, closable tabs, bottom status | All 14 sidebar destinations opened. Tabs accumulate horizontally; labels clip at narrow width. Workspace switcher opens Overview rather than a chooser. Jobs badge opens Runs; no separate job detail was observed. Native badges/status must derive from services. |
| REF-03 | Overview: metrics, key→node→playbook→run, health links, activity, trust/details | Observed. Static Homelab counts/trust/revision; native implementation needs empty/loading/stale/error states and real session trust. |
| REF-04 | Create workspace: name, location, Browse, package preview, Cancel/Create | Opened/cancelled. No package created. Use existing native picker/create contract. |
| REF-05 | Open workspace: path, Browse, Cancel/Open | Opened/cancelled. No local package transmitted. Keep existing switch/dirty/conflict protections. |

## Commands and keyboard

| ID | Interaction | Result |
| --- | --- | --- |
| REF-06 | Header command trigger, Cmd+K, Cmd+Shift+P | All open command popup; input receives focus. |
| REF-07 | Type `terminal`, Enter | Filters to Open terminal; Enter opens terminal form with Local machine selected. No command executes. |
| REF-08 | Type `route`, Enter | Opens route chooser with Direct/Up, VLESS EU/Up, SOCKS Lab/Starting, HTTP Edge/Down. Cancelled. |
| REF-09 | Escape from popup/forms/help | Closes. Focus then reported on web area rather than original trigger: require reliable focus restoration in native UI. |
| REF-10 | Shortcut help | Opened. Lists Tab/Shift+Tab, Cmd+Shift+E, Cmd+W, Cmd+S, Cmd+F, Ctrl+Tab/Ctrl+Shift+Tab, zoom, Space, Shift+arrows, Menu/Shift+F10. These shortcuts were visible, not all executed. Remapping described as later release. |

Observed command catalog (24): Open workspace, Create workspace, Add node, Edit
node, Test connection, Use system SSH key, Import custom key, Generate SSH key,
New playbook (scenario), Import playbook, Edit playbook, Plan run, Apply run,
Open terminal, Open browser tab, Switch route, Renew certificate, Apply ComfyUI
recipe, MCP server settings, Background jobs, Approvals inbox, Settings,
Keyboard shortcuts, Move focus to navigation. Labels, descriptions, groups and
displayed shortcuts are retained in [commands.ax.txt](commands.ax.txt).

The popup advertises arrows/Enter/back behavior and terminal-like command syntax.
Full argument parsing, arrow navigation, command errors and keyboard-only coverage
of every row/menu/editor are **not established** by these observations.

## Nodes, keys and terminals

| ID | Surface and controls | Observation / target requirement |
| --- | --- | --- |
| REF-11 | Nodes: add/import, text filter, route filter, row/select-all checkboxes, details, terminal, run playbook, More | Filtering `edge` yields edge1/edge2; selecting edge1 adds “1 selected / Plan run.” More on edge1 produced no visible menu. Import and route-filter effects not exercised. Implement real import preview, context menu and selected-target plan flow. |
| REF-12 | Add node | Form opened/cancelled: name, labels, host, port22, user, key/agent/memory-only password, key source/inheritance, route, optional ProxyJump, workspace/system known_hosts, Test connection and Add. Saving must not connect. |
| REF-13 | Node details | Overview/Facts/Files/Runs/Terminal tabs visible; Overview, Facts, Files, Terminal opened. Facts are cached with timestamp/Refresh. Files is scoped-transfer placeholder. Node Runs tab visible, not exercised. |
| REF-14 | Node Terminal and Open terminal modal | Static simulated terminal. Form has local/node target, default/bash/zsh and session label. Open for edge1 produced “Terminal: edge1 (simulated).” No emulator opened. Real local and SSH PTY are required. |
| REF-15 | SSH Keys library | System `~/.ssh` keys and custom managed key; algorithm/fingerprint/default/locked status, public copy/use/remove-reference; node>workspace>global explanation. Assignment, deletion and unlock not exercised. |
| REF-16 | Use system / Import / Generate key forms | Each opened/cancelled. System form path/Browse. Import reference-in-place or managed copy. Generate algorithm Ed25519/RSA4096, name, destination, optional passphrase. No private bytes entered or keys created. |

## Playbooks and runs

| ID | Surface and controls | Observation / target requirement |
| --- | --- | --- |
| REF-17 | Playbook library | Three cards with name/path/readiness/targets/time, Import/New/Plan. install-nginx editor opened. Import not exercised. |
| REF-18 | New playbook | Opened/cancelled: name, path, Blank/Install package/Deploy config/Manage service/GPU setup templates, Create & edit. Must write a valid reviewed local file and open editor without execution. |
| REF-19 | Builder | Tasks package/file/symlink, duplicate/delete icons, Add task, handlers, variables, targets/diagnostics; some icons lack accessible names. Task-edit/add/reorder not exercised. Require source fidelity and keyboard actions. |
| REF-20 | Source and Check | Source is rendered YAML text, not an accessible editable input. Check produces “Check probes are read-only (simulated).” Sample `worker_processes` integer default `auto` is inconsistent; native validation must catch it. |
| REF-21 | Run plan | Plan opened: revision, identities/route, resolved nodes, task×node changes/diff, concurrency2, timeout10m, sudo. Apply produced “Check does not apply changes.” It did not open expected approval/apply flow. |
| REF-22 | Runs history | Search/status filter and four rows, mode/targets/counters/time, Retry failed nodes/Export log. Opening `r-1042` displays monitor `r-1043` Running: incorrect identity mapping. Retry/export effects not exercised. |
| REF-23 | Run monitor | Opened: task matrix, context/live log, Cancel. Static sample state; cancellation was not exercised. Require actual sequenced progress, correct counters/identity and durable terminal outcome. |

## Browser, routes and certificates

| ID | Surface and controls | Observation / target requirement |
| --- | --- | --- |
| REF-24 | Browser Profiles | Work/Research/Client profile tabs and library cards with UA/route/store/status/open/clear. Native webview area explicitly simulated; chrome icons not exposed as accessible buttons. No real profile isolation verified. |
| REF-25 | Open browser tab / Switch route | Both forms opened/cancelled. Profile/start URL; scoped route list with consumer-review text. Clear-data confirmation and profile create/edit were not exercised; no create/edit control observed on library surface. |
| REF-26 | Tunnels | Direct/VLESS/SOCKS/HTTP inventory, endpoint/state/consumers/restarts, readiness timeline/logs. Add tunnel produces simulated toast instead of editor. Row actions unlabeled; no start/stop/restart effect verified. |
| REF-27 | Certificates | Domain/issuer/provider/expiry/node/status table, add/renew, DNS failure banner, app-running scheduler, distribution old/new fingerprints/staged pair/Apply distribution. Renew produced simulated toast. Add/distribution effects not exercised. |

## Scripts, recordings, recipes, MCP and approvals

| ID | Surface and controls | Observation / target requirement |
| --- | --- | --- |
| REF-28 | Scripts | Library/new, static Python text, runtime, save icon, source hash/capabilities, unsandboxed warning. Review capabilities immediately marks session approved and exposes Run file; no review sheet observed. Script was not run. Need real editor and explicit scoped human grant. |
| REF-29 | Recordings | Library/new, Record/Pause/Stop/elapsed/step count/masking, Replay/Edit/Export, setup profile/vars/timing/CSV. Edit produced no visible change; New recording produces simulated armed toast. No capture/replay verified. |
| REF-30 | ComfyUI Recipes | Two recipes, commit/models/custom nodes, preflight/free disk/shared symlink, download queue, Verify/Apply/Export/New. Apply produces simulated toast. Declared6.4GB conflicts with 11.5GB+335MB queue; compute real totals. No installation/download. |
| REF-31 | MCP | Disabled, Start, read-only toggle, stdio+loopback settings, token-provider status, allowlist, small tool catalog, pending request Inspect/Deny. Opened only; server not started, permissions not changed. Prototype names (`list_nodes`, `run_apply`) differ from normative dotted B1 tools. |
| REF-32 | Approvals | Pending requester/tool/command/targets/route; Allow once/Allow session/Deny; session grants, history and export audit visible. No grant/deny clicked. Add full diff/revision/digest/expiry and revocation behavior in native service. |

## Settings (each category opened)

| Category | Observed contents | Native completion |
| --- | --- | --- |
| Appearance | Nightfly, monospace stack, zoom −/+/reset, ligatures off, reduced motion | Persistent local preferences, zoom bounds, real font preview. Zoom effects not exercised. |
| Workspace defaults | “Reachable settings surface. Full controls land in a later release.” | Key/route/targets/concurrency/timeouts with inheritance/validation. |
| SSH trust | Same placeholder | Known hosts, explicit enrollment/mismatch handling, no global bypass. |
| SSH Keys | Files/no Keychain/perms text and Use system key | Full shared library/defaults, reference removal and managed-file lifecycle. |
| Other secrets | Two metadata-only references, no values | Native secure-entry provider for non-SSH credentials; never repurpose for SSH keys. |
| Runtimes / Sidecars | Placeholders | Detected/pinned versions, paths, availability, licenses, supervised lifecycle. |
| Terminal | Font14/scrollback10000/canvas/experimental CSI-u/history off/bracketed paste | Editable validated settings, measured capability report; no untested renderer/protocol claim. |
| Browser capabilities | Placeholder | Native backend/OS/API/proxy limitations and smoke results. |
| Notifications / Storage & privacy | Placeholders | Job notifications, explicit retention/purge/export scopes; no automatic secret collection. |
| About / licenses | Hard-coded prototype/OS/runtime/license values, Copy diagnostics | Actual build/runtime/version/license information; previewable redacted export. |
| Keyboard shortcuts | Opens help modal | Search/remap/collision detection and complete action coverage. |

Detailed settings snapshots: [settings.ax.txt](settings.ax.txt). Some entries are
accessibility diffs, read in order rather than as independent complete trees.

## Evidence and limits

- [Screenshot gallery and manifest](README.md): 33 original JPEG captures; first 16
  at narrow browser size, remaining 17 at wider size. Preserve these differences;
  they show layout constraints rather than a controlled visual-regression suite.
- Screens include the reference's Base44 edit badge. Do not port it, fake window
  traffic lights, hard-coded identities, sample success statuses or simulated toasts.
- Browser interaction only: no Base44 project/source export, endpoint discovery,
  authentication secrets, private implementation inspection or ownership changes.
- Unexercised controls above are planned behavior, not claimed working reference
  features. Destructive clear/delete, actual grants, installation, real network
  effects and external exports were intentionally not executed during inspection.
- This inventory plus existing [full UI brief](../../BASE44_DESIGN_REQUEST.md)
  defines parity scope. The native release must replace placeholders with real
  service results or clearly state unavailable; never return simulated success.
