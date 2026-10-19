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
- [20](20-ssh-review-debian.jpg): native managed SSH review of Debian endpoint,
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

Manual system-SSH checks used existing verified host keys with StrictHostKeyChecking=yes,
BatchMode=yes and bounded connect attempts. Those checks changed no remote files.
Captures 16–17 are manual system-SSH sessions inside a real local PTY.
Captures 20–22 use the separate ghosh-ssh transport and native review service; both
authenticate as vm with an explicitly selected existing file key and close without
executing remote commands. Both match existing known-host records. Managed remote
PTY acceptance is shown in 24–25. Persistent host enrollment, SFTP and facts remain pending.
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
