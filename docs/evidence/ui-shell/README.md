# Native UI evidence

Captured from the local arm64 release app on macOS 27, deployment target 14.0.
These are native screenshots, separate from the Base44 prototype reference.
Minimum macOS 14 execution and managed SSH route-loss acceptance remain pending.

- 01–04: Nightfly welcome, conflict/draft preservation, settings and command popup.
- 05–07: native jobs, approvals and 200% zoom.
- 08–10: keyboard node editor, review and saved inventory.
- 11–14: explicit Ed25519 generation review, file-key library, schema 2 identity
  migration review and inherited node fingerprint review.
- [15](15-local-pty.jpg): real local shell with TTY, ANSI and UTF-8 output.
- [16](16-debian-pty.jpg): system SSH to debian.orb.local inside Ghosh; Linux ARM64.
- [17](17-node3-pty.jpg): system SSH to node3.dabaga.cc inside Ghosh; Debian x86-64.
- [18](18-split-pty.jpg): independent local sessions, both resized to 21×48 columns.
- [20](20-ssh-review-debian.jpg): historical managed SSH review of Debian endpoint,
  inherited file identity and known ED25519 server fingerprint.
- [21](21-ssh-auth-debian.jpg): native managed Debian authentication PASS, 106ms.
- [22](22-ssh-auth-node3.jpg): native managed node3 authentication PASS, 2298ms.
- [23](23-themed-dropdown.jpg): current native Nightfly custom selector, taller rows, SVG arrow/check and header/navigation icons.
  Captured 2026-10-18 in the user-opened workspace without saving any changes.
  Original JPEG, 3384×2168; SHA-256 `cb514c07e5e93d83a2aa9420789bdd4c8fe33c5bc259867a3bc4036e0b5b5c57`.

- [24](24-managed-debian-pty.jpg): managed Debian SSH PTY, TTY/ANSI/Unicode,
  Ctrl+C and Vim edit/save/exit; 2026-10-19, original JPEG 2320×1560.
- [25](25-managed-ssh-split.jpg): managed node3 and Debian sessions, both resized
  to 21×48; 2026-10-19, original JPEG 2320×1560.
- [26](26-playbook-examples.jpg): native Playbook examples catalog, release 0.1.2.
- [27](27-playbook-editor.jpg): native source editor after successful validation;
  create, save and reopen preserved the exact source in the disposable workspace.
- [28](28-immediate-ssh-test.jpg): immediate Debian SSH authentication success
  without an approval step; 109 ms, a single observation rather than a benchmark.
  Node3 Test also passed (1,389 ms). Both Terminals opened directly with input focused.
  Captures 26–28: 2026-10-19, original JPEG 2320×1560.
- [29](29-run-targets.png): release 0.1.3 Run dialog with Debian, local machine
  and node3 selected; the unrelated example node remains unselected.
- [30](30-run-results.png): persisted successful `linux-facts` Check report,
  including Debian stdout and phase status.
- [31](31-run-local-node3.png): the same report scrolled to real Darwin arm64
  local output and Linux x86-64 node3 output; all three targets exited 0.
  Captures 29–31: 2026-10-19, original PNG 2320×1560. This run only reads facts.
  One earlier UI connection attempt failed before execution; the later successful
  report is the captured evidence. Independent native acceptance also passed on
  each remote host. The original `1.ghosh` was reopened afterwards without edits.

Manual system-SSH checks used existing verified host keys with StrictHostKeyChecking=yes,
BatchMode=yes and bounded connect attempts. Those checks changed no remote files.
Captures 16–17 are manual system-SSH sessions inside a real local PTY.
Captures 20–22 show the earlier approval UI, superseded in 0.1.2. They use the
separate ghosh-ssh transport and native review service; both
authenticate as vm with an explicitly selected existing file key and close without
executing remote commands. Both match existing known-host records. Managed remote
PTY acceptance is shown in 24–25. Persistent host enrollment and SFTP remain pending;
read-only facts scenarios and their durable results are shown in 29–31.
Managed Debian Vim wrote only `/tmp/ghosh-managed-pty-smoke-20261019`;
no production configuration or software changed. Native local vi editing/save/exit
and shell-screen restoration also passed.
Native screenshots were inspected; they are not frontend fixture renderings.

The generated identity is disposable test data. Later managed authentication uses
a reviewed reference to the existing ~/.ssh/id_ed25519; captures show only public
fingerprints. No private bytes were shown, copied or added to the workspace.
The fixture `/private/tmp/Ghosh-UI-1018.ghosh` was migrated by explicit keyboard
review; its schema 1 manifest recovery copy remains beside the schema 2 manifest.
The original user workspace was not migrated. The global key default stayed unset.
Managed test key: 0600; containing directory: 0700; generation stages cleaned.

Native keyboard selectors need the popup to open before arrow selection. Review,
save and focus return were checked in the real app. Browser tests use explicit IPC
fixtures and do not substitute for native filesystem/SSH/PTY acceptance.


## 2026-10-19 — Browser fixtures for new authoring UI

These are explicit IPC-fixture presentation captures, **not native execution or GPU evidence**.
- [32](32-ansible-results-fixture.png): stored report and JSON export control.
- [33](33-comfy-recipe-fixture.png): narrow recipe layout at 460×760; contained model/plugin inputs.
- [34](34-comfy-wizard-fixture.png): full-width ComfyUI wizard with editable installation paths and pin/hash fields.

Separate genuine Ansible controller and authorized Debian/node3 tests are recorded
in TASK_LOG.md. No GPU installation or model download was performed.

- [35](35-code-editor-fixture.png): bundled Monaco syntax-highlighted source at 460×760; offline completion, maximize/restore and RU physical Cmd+S are covered by the UI journey.
- [36](36-live-run-fixture.png): current task/output and overall 4/6 progress at 460×760; footer returns from another section. Explicit IPC fixture, not native execution evidence.

## 2026-10-19 — Native 0.1.5 editor and streaming acceptance

Actual macOS arm64 release app, disposable `/private/tmp/Ghosh live editor smoke 20261019.ghosh`:
- [37](37-native-expanded-editor.png): expanded offline Monaco with shell syntax highlighting and native syntax-only validation.
- [38](38-native-live-output.png): real local stdout/stderr visible before exit, after navigating to Nodes and returning through the running footer.
- [39](39-native-typescript-diagnostics.png): bundled worker reports a genuine TypeScript type mismatch; Deno property completion was also observed.

Run `0495d9e7-2408-4ed6-b945-3e7cdc48667f` finished with exit 0, 1/1 steps,
100% progress and saved LIVE_STDOUT/LIVE_STDERR/FINISHED output. Cmd+S saved
the shell script. The unsaved diagnostic draft was discarded. Original
`/Users/vm/Documents/1.ghosh` was reopened without changing its files.

Real native local tests separately verify stdout/stderr arrival before process exit,
check/apply/verify counting, persistent final reports and old report compatibility.
