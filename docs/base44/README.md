# Ghosh → Base44 handoff

Prepared 2026-10-18, sequentially. Nothing uploaded or sent to Base44.

## Live reference → native implementation plan

The resulting [Base44 reference](https://ghosh-herd-os.base44.app/) was inspected
through the requested Browser. See the [implementation plan](../IMPLEMENTATION_PLAN_UI_FUNCTIONALITY.md),
[interaction inventory](reference-app/INTERACTION_INVENTORY.md) and
[33 reference screenshots](reference-app/README.md). Those captures show the
prototype; the ten native screenshots below show current implemented Ghosh.
The earlier handoff ZIP has not been rebuilt to include this new reference evidence.

## Start here

1. Paste [the complete design request](../BASE44_DESIGN_REQUEST.md) into Base44.
2. Attach the ten JPEGs in [screenshots/](screenshots/) as **current implementation
   references**. If attachment count is limited, start with 01, 02, 03, 07, 09, 10.
3. Include [SSH file storage requirements](../SSH_KEY_STORAGE.md) for implementation
   handoff. The brief already covers the user-facing key views.
4. Review the [offline gallery](index.html) and [capture manifest](screenshots.json).
   [Nightfly palette transcription](reference/nightfly-default.toml) is included.

The ZIP contains these docs and all ten screenshots in a portable folder.
[SHA-256 checksums](SHA256SUMS.txt) identify the original capture bytes. It does
not contain workspace packages, source code, SSH keys, tokens or private manifests.

## Requested design

- Nightfly Default dark palette, readable monospace UI, terminal-like tables/panels.
- Every operation keyboard-accessible. Cmd+K opens app commands and focused modals.
- First-class node setup, system/custom file keys, YAML playbooks, target selection,
  fresh plan/approval and per-node run results. No Keychain for SSH identities.
- Real local terminal and real SSH PTY terminal per node in the native app; local
  interactive terminal smoke and separate remote PTY acceptance tests are pending.
- Full original product scope retained: 76 screen/state entries + K1–K9/P1–P12.

## Capture provenance and limits

These are ten unedited native Ghosh window captures on Apple Silicon macOS 27,
light theme, approximately 1160×780 logical pixels (2320×1560 JPEG pixels). They
show implemented M1.1 screens/states, not proposed designs. The redesigned palette
on this gallery is a review aid, not an app screenshot. OS overlays visible in a
capture are incidental. No dark-theme application smoke was performed.

02–04 include the actual `/Users/vm/Documents/1.ghosh` workspace. It was opened
and inspected without changing files or trust. Other state captures use the
current welcome/open UI or a disposable `/private/tmp/Ghosh-Base44-1018.ghosh`
fixture, including name drafts, conflict and fictional node inventory. The fixture
was never connected to a node and trust confirmation was cancelled. The original
workspace was reopened after capture; creating the fixture added a recent entry.

Screenshots expose local paths, display names, workspace identity/revision and
recent-history entries. They contain no private key or token values. Review this
local metadata before sharing externally; no upload was performed by this task.
Legacy Keychain wording belongs to the current app, not the new SSH requirement.

Current app has no key manager, playbook editor, SSH runtime or terminal emulator.
Those requested views cannot be captured as existing functionality. Native local
PTY/SSH tests are specified, not executed; standalone shell commands would not
validate an in-app terminal. No runtime source was changed for this handoff.

## Verification

All ten files were checked as JPEG captures at 2320×1560; original bytes preserved.
The complete brief retains all 76 original entries, plus nine key and twelve
playbook views. Document links/fences, palette TOML, manifest, gallery image links
and image descriptions passed static checks. The gallery was not browser-rendered:
the browser tool blocked local file URLs, so no alternate browser route was used.
Application runtime/build tests were not repeated for this documentation task.
