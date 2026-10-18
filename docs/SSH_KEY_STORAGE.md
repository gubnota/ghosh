# SSH keys: accepted file storage requirements

Date: 2026-10-18. Status: file-key library/schema migration implemented;
reviewed direct SSH authentication implemented; encrypted unlock, managed PTY
and consumer discovery remain pending.
This amendment supersedes older Keychain-based SSH examples in the build prompt
and architecture. It does not change credential storage for unrelated services.

## Scope and storage

- “System keys” means current-user existing SSH key files (normally `~/.ssh`),
  reusable across Ghosh workspaces. No root privileges or cross-account sharing.
- Custom keys support reference-in-place or explicit copy to user-local managed
  storage, e.g. `~/Library/Application Support/app.ghosh.desktop/ssh/<uuid>.key`.
  Resolve the actual app data path through the platform provider, not a literal
  hard-coded home path. Registry metadata also lives outside workspace packages.
- Use ordinary filesystem storage. **No Keychain for SSH keys or passphrases.**
  Preserve passphrase encryption on encrypted private-key files. No requirement
  for an application vault/database or opaque OS credential service.
- Managed folders are 0700 and private files 0600, created restrictively without
  overwriting a source file. Reject unsafe managed paths/symlink substitution;
  existing user file links are rejected. Existing file permissions are checked, never silently changed.
- Explicit selected-file discovery only; no recursive credential ingestion or
  arbitrary `ssh_config` command execution. Preview algorithm/public fingerprint,
  source, ownership and usage without displaying private bytes.
- Passphrases/passwords are input per use, with an optional clearly labeled
  memory-only session cache. Forget on lock/quit, never persist in config, browser
  storage, command history, logs or Keychain. Do not embed them in command strings.
- SSH agent is an optional separate auth mode, never automatically enabled or used
  as a fallback for a missing explicit key. Agent forwarding stays off.

## References and precedence

- A local registry maps opaque key IDs to selected/managed paths and public metadata.
  Frontend DTOs contain metadata/IDs, never raw private bytes or passphrases.
- Effective key: explicit node override > workspace default > global default.
  An unresolved explicit override is an actionable error, not identity fallback.
- Workspace files store portable bindings/reference IDs, not private material.
  A copied/imported workspace can have unresolved bindings and needs explicit
  relinking; never resolve an untrusted imported absolute path automatically.
- Global reuse is within one macOS user account. Opening a package cannot read a
  key or attempt authentication. Unlocking/using keys follows explicit native
  service authorization and workspace trust rules.
- Referenced external files remain user-owned. Remove reference never deletes them.
  Managed-file deletion is a separate explicit confirmation showing affected
  workspaces/nodes. Removing a global default must explain inheritance changes.
- Export/backup/diagnostic bundles exclude private keys and passphrases. Public
  metadata/path disclosure should be previewable. Native services alone read keys.
- Server host identity is independent of client authentication: strict known_hosts,
  explicit first enrollment and no automatic acceptance of changed host keys.

## Implementation boundary and migration

The file-key library is implemented in `ghosh-core/src/ssh_keys.rs` with native
`ssh-keygen` adapters. It supports explicit existing/custom references, managed
copies, unencrypted Ed25519 generation, public metadata, defaults, same-fingerprint
relink and removing metadata while retaining private files. The local registry is
`ssh/registry.json` under the platform app-data directory. Encrypted imports retain
their encryption; there is no passphrase IPC or persisted passphrase cache.

Schema 2 stores opaque registry IDs. Workspace identity changes preview the schema
migration and retain a recovery manifest before committing. Schema 1 remains
readable without migration on open; legacy `secret://keychain/<id>` SSH references
are inert for transport. The separate non-SSH Keychain adapter still exists.

The approved implementation plan covers the key ports, generated DTOs and tested
missing/unsafe/encrypted files, precedence, import/relink and reference-only removal.
The native direct-authentication consumer resolves only reviewed, available,
unencrypted keys after rechecking registry revision and file safety. Private bytes
stay native and are held in Zeroizing storage; no key material crosses IPC.
Next integrate memory-only unlock and managed SSH PTY consumers. Retain non-SSH
SecretStore consumers independently; do not force file-key storage through the
current Keychain-only parser. Legacy SSH references require an explicit migration
workflow; never silently export existing Keychain items or rewrite user packages.

Design screens and keyboard flows: [BASE44_DESIGN_REQUEST.md](BASE44_DESIGN_REQUEST.md),
K1–K9, command popup and keyboard-only acceptance. The original design handoff
did not access keys; later implementation tests generated a disposable local identity
and migrated only the disposable native fixture. No private material is in screenshots.
