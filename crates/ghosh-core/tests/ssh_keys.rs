use base64::{
    Engine,
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD},
};
use ghosh_core::{contracts::*, policy::Principal, ssh_keys::*};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
};

struct Probe;
impl KeyInspector for Probe {
    fn inspect(&self, path: &Path) -> Result<PublicKeyMetadata> {
        let bytes = fs::read(path)?;
        assert_eq!(fs::metadata(path)?.permissions().mode() & 0o777, 0o600);
        Ok(PublicKeyMetadata {
            algorithm: "ED25519".into(),
            fingerprint: format!("SHA256:{}", STANDARD_NO_PAD.encode(Sha256::digest(&bytes))),
        })
    }
}
fn fixture(root: &Path, name: &str, encrypted: bool) -> PathBuf {
    let path = root.join(name);
    let cipher: &[u8] = if encrypted { b"aes256-ctr" } else { b"none" };
    let mut bytes = b"openssh-key-v1\0".to_vec();
    bytes.extend_from_slice(&(cipher.len() as u32).to_be_bytes());
    bytes.extend_from_slice(cipher);
    fs::write(
        &path,
        format!(
            "-----BEGIN OPENSSH PRIVATE KEY-----\n{}\n-----END OPENSSH PRIVATE KEY-----\n",
            STANDARD.encode(bytes)
        ),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    path
}
fn request(path: &Path, storage: KeyStorage) -> KeyImportRequest {
    KeyImportRequest {
        path: path.to_string_lossy().into(),
        label: "Fixture identity".into(),
        storage,
    }
}
fn commit(registry: &mut FileKeyRegistry, path: &Path, storage: KeyStorage) -> KeysSnapshot {
    let revision = registry.snapshot(Principal::LocalUi).unwrap().revision;
    let preview = registry
        .preview(Principal::LocalUi, request(path, storage), &Probe)
        .unwrap();
    registry
        .commit(
            Principal::LocalUi,
            KeyCommitRequest { preview, revision },
            &Probe,
        )
        .unwrap()
}
#[test]
fn reference_import_is_inert_until_commit_and_survives_restart_without_private_registry_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = fixture(&root, "id_fixture", false);
    let original = fs::read(&path).unwrap();
    let mut registry = FileKeyRegistry::open(&root.join("ssh")).unwrap();
    let preview = registry
        .preview(
            Principal::LocalUi,
            request(&path, KeyStorage::Reference),
            &Probe,
        )
        .unwrap();
    assert!(!preview.encrypted);
    assert!(
        registry
            .snapshot(Principal::LocalUi)
            .unwrap()
            .keys
            .is_empty()
    );
    assert!(!root.join("ssh/registry.json").exists());
    assert_eq!(fs::read_dir(root.join("ssh")).unwrap().count(), 1); // lease only; inspection stage removed
    let snapshot = commit(&mut registry, &path, KeyStorage::Reference);
    let id = snapshot.keys[0].id.clone();
    let snapshot = registry
        .set_default(
            Principal::LocalUi,
            KeyRegistryRequest {
                id: Some(id.clone()),
                revision: snapshot.revision,
            },
        )
        .unwrap();
    assert_eq!(snapshot.default_key, Some(id));
    drop(registry);
    let mut registry = FileKeyRegistry::open(&root.join("ssh")).unwrap();
    let snapshot = registry.snapshot(Principal::LocalUi).unwrap();
    assert_eq!(snapshot.keys[0].status, KeyStatus::Available);
    let serialized = fs::read_to_string(root.join("ssh/registry.json")).unwrap();
    assert!(!serialized.contains("PRIVATE KEY"));
    assert!(!serialized.contains("passphrase"));
    registry
        .forget(
            Principal::LocalUi,
            KeyRegistryRequest {
                revision: snapshot.revision,
                id: Some(snapshot.keys[0].id.clone()),
            },
        )
        .unwrap();
    assert_eq!(fs::read(path).unwrap(), original);
}
#[test]
fn managed_copy_is_owner_only_nonoverwriting_and_forget_retains_recoverable_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = fixture(&root, "encrypted_fixture", true);
    let original = fs::read(&path).unwrap();
    let mut registry = FileKeyRegistry::open(&root.join("ssh")).unwrap();
    let snapshot = commit(&mut registry, &path, KeyStorage::Managed);
    let key = &snapshot.keys[0];
    assert!(key.encrypted);
    assert_ne!(Path::new(&key.path), path);
    assert_eq!(fs::read(&key.path).unwrap(), original);
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(
        fs::metadata(&key.path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        fs::metadata(root.join("ssh")).unwrap().permissions().mode() & 0o777,
        0o700
    );
    registry
        .forget(
            Principal::LocalUi,
            KeyRegistryRequest {
                revision: snapshot.revision,
                id: Some(key.id.clone()),
            },
        )
        .unwrap();
    assert!(Path::new(&key.path).exists());
}
#[test]
fn unsafe_links_permissions_public_files_and_foreign_principals_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = fixture(&root, "key", false);
    let registry = FileKeyRegistry::open(&root.join("ssh")).unwrap();
    assert!(
        registry
            .preview(
                Principal::Mcp,
                request(&path, KeyStorage::Reference),
                &Probe
            )
            .is_err()
    );
    symlink(&path, root.join("link")).unwrap();
    assert!(
        registry
            .preview(
                Principal::LocalUi,
                request(&root.join("link"), KeyStorage::Reference),
                &Probe
            )
            .is_err()
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(
        registry
            .preview(
                Principal::LocalUi,
                request(&path, KeyStorage::Reference),
                &Probe
            )
            .is_err()
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o644
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, "ssh-ed25519 public-only").unwrap();
    assert!(
        registry
            .preview(
                Principal::LocalUi,
                request(&path, KeyStorage::Reference),
                &Probe
            )
            .is_err()
    );
    assert!(FileKeyRegistry::open(&root.join("ssh")).is_err());
}
#[test]
fn changed_source_preview_stale_revision_and_registry_tamper_never_commit() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = fixture(&root, "key", false);
    let mut registry = FileKeyRegistry::open(&root.join("ssh")).unwrap();
    let revision = registry.snapshot(Principal::LocalUi).unwrap().revision;
    let preview = registry
        .preview(
            Principal::LocalUi,
            request(&path, KeyStorage::Reference),
            &Probe,
        )
        .unwrap();
    fixture(&root, "key", true);
    assert!(
        registry
            .commit(
                Principal::LocalUi,
                KeyCommitRequest {
                    preview,
                    revision: revision.clone()
                },
                &Probe
            )
            .is_err()
    );
    assert!(
        registry
            .snapshot(Principal::LocalUi)
            .unwrap()
            .keys
            .is_empty()
    );
    let snapshot = commit(&mut registry, &path, KeyStorage::Reference);
    assert!(
        registry
            .set_default(
                Principal::LocalUi,
                KeyRegistryRequest {
                    revision,
                    id: Some(snapshot.keys[0].id.clone())
                }
            )
            .is_err()
    );
    fs::write(&path, "changed").unwrap();
    assert_eq!(
        registry.snapshot(Principal::LocalUi).unwrap().keys[0].status,
        KeyStatus::Changed
    );
    fs::remove_file(&path).unwrap();
    assert_eq!(
        registry.snapshot(Principal::LocalUi).unwrap().keys[0].status,
        KeyStatus::Missing
    );
    fs::write(root.join("ssh/registry.json"), "{}").unwrap();
    assert!(
        registry
            .forget(
                Principal::LocalUi,
                KeyRegistryRequest {
                    revision: snapshot.revision,
                    id: Some(snapshot.keys[0].id.clone())
                }
            )
            .is_err()
    );
}
#[test]
fn root_replacement_blocks_reads_and_writes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = fixture(&root, "key", false);
    let registry = FileKeyRegistry::open(&root.join("ssh")).unwrap();
    fs::rename(root.join("ssh"), root.join("old")).unwrap();
    fs::create_dir(root.join("outside")).unwrap();
    symlink(root.join("outside"), root.join("ssh")).unwrap();
    assert!(
        registry
            .preview(
                Principal::LocalUi,
                request(&path, KeyStorage::Managed),
                &Probe
            )
            .is_err()
    );
    assert_eq!(fs::read_dir(root.join("outside")).unwrap().count(), 0);
}

#[test]
fn native_resolution_rechecks_revision_encryption_and_file_identity() {
    use ghosh_core::policy::SecretAccess;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let mut registry = FileKeyRegistry::open(&root.join("ssh")).unwrap();
    let access = SecretAccess::authorize(
        Principal::NativeService,
        &ulid::Ulid::new().to_string(),
        true,
    )
    .unwrap();
    let path = fixture(&root, "key", false);
    let snapshot = commit(&mut registry, &path, KeyStorage::Reference);
    let id = &snapshot.keys[0].id;
    let private = registry.resolve(&access, id, &snapshot.revision).unwrap();
    assert_eq!(private.as_slice(), fs::read(&path).unwrap().as_slice());
    drop(private);
    assert!(registry.resolve(&access, id, "stale").is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(registry.resolve(&access, id, &snapshot.revision).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fixture(&root, "key", true);
    assert!(registry.resolve(&access, id, &snapshot.revision).is_err());
    let encrypted = fixture(&root, "encrypted", true);
    let snapshot = commit(&mut registry, &encrypted, KeyStorage::Reference);
    let id = &snapshot.keys.iter().find(|key| key.encrypted).unwrap().id;
    let ciphertext = registry.resolve(&access, id, &snapshot.revision).unwrap();
    assert_eq!(
        ciphertext.as_slice(),
        fs::read(&encrypted).unwrap().as_slice()
    );
}
