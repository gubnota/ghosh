use ghosh_core::{
    contracts::*,
    ssh_keys::{KeyGenerator, KeyInspector, PublicKeyMetadata},
};
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub struct OpenSshGenerator;
impl KeyGenerator for OpenSshGenerator {
    fn generate(&self, path: &Path) -> Result<()> {
        let unavailable = || AppError::new(ErrorCode::KeyUnavailable);
        let mut child = Command::new("/usr/bin/ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "", "-f"])
            .arg(path)
            .env("SSH_ASKPASS_REQUIRE", "never")
            .env_remove("DISPLAY")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| unavailable())?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    return if status.success() {
                        Ok(())
                    } else {
                        Err(unavailable())
                    };
                }
                Ok(None) if started.elapsed() < Duration::from_secs(5) => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(unavailable());
                }
            }
        }
    }
}

pub struct OpenSshInspector;
impl KeyInspector for OpenSshInspector {
    fn inspect(&self, path: &Path) -> Result<PublicKeyMetadata> {
        let unavailable = || AppError::new(ErrorCode::KeyUnavailable);
        let mut child = Command::new("/usr/bin/ssh-keygen")
            .args(["-l", "-E", "sha256", "-f"])
            .arg(path)
            .env("SSH_ASKPASS_REQUIRE", "never")
            .env_remove("DISPLAY")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| unavailable())?;
        let stdout = child.stdout.take().ok_or_else(unavailable)?;
        let reader = std::thread::spawn(move || {
            let mut bytes = Vec::new();
            stdout.take(4097).read_to_end(&mut bytes).map(|_| bytes)
        });
        let start = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) if start.elapsed() < Duration::from_secs(5) => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(unavailable());
                }
            }
        };
        let bytes = reader
            .join()
            .map_err(|_| unavailable())?
            .map_err(|_| unavailable())?;
        if !status?.success() || bytes.len() > 4096 {
            return Err(unavailable());
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| unavailable())?;
        if text.lines().count() != 1 {
            return Err(unavailable());
        }
        let fields: Vec<_> = text.split_whitespace().collect();
        let fingerprint = fields.get(1).ok_or_else(unavailable)?.to_string();
        let algorithm = fields
            .last()
            .ok_or_else(unavailable)?
            .trim_matches(['(', ')'])
            .to_string();
        if !["ED25519", "RSA", "ECDSA", "DSA", "ED25519-SK", "ECDSA-SK"]
            .contains(&algorithm.as_str())
            || !fingerprint.starts_with("SHA256:")
        {
            return Err(unavailable());
        }
        Ok(PublicKeyMetadata {
            algorithm,
            fingerprint,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghosh_core::{policy::Principal, ssh_keys::*};
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn native_generation_and_reference_relink_keep_identity_and_remove_staging() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let mut registry = FileKeyRegistry::open(&root.join("registry")).unwrap();
        let revision = registry.snapshot(Principal::LocalUi).unwrap().revision;
        let generated = registry
            .generate(
                Principal::LocalUi,
                KeyGenerateRequest {
                    revision,
                    label: "Generated fixture".into(),
                },
                &OpenSshGenerator,
                &OpenSshInspector,
            )
            .unwrap();
        let key = &generated.keys[0];
        assert_eq!(key.storage, KeyStorage::Managed);
        assert!(!key.encrypted);
        assert_eq!(key.status, KeyStatus::Available);
        assert_eq!(
            std::fs::metadata(&key.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            OpenSshInspector.inspect(Path::new(&key.path)).unwrap(),
            key.public
        );
        assert!(generated.default_key.is_none());
        assert!(
            !std::fs::read_dir(root.join("registry"))
                .unwrap()
                .any(|entry| entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".generate-"))
        );

        let source = root.join("reference");
        OpenSshGenerator.generate(&source).unwrap();
        let preview = registry
            .preview(
                Principal::LocalUi,
                KeyImportRequest {
                    path: source.to_string_lossy().into(),
                    label: "Movable fixture".into(),
                    storage: KeyStorage::Reference,
                },
                &OpenSshInspector,
            )
            .unwrap();
        let registered = registry
            .commit(
                Principal::LocalUi,
                KeyCommitRequest {
                    preview,
                    revision: generated.revision,
                },
                &OpenSshInspector,
            )
            .unwrap();
        let reference = registered
            .keys
            .iter()
            .find(|key| key.storage == KeyStorage::Reference)
            .unwrap();
        let id = reference.id.clone();
        let fingerprint = reference.public.clone();
        let registered = registry
            .set_default(
                Principal::LocalUi,
                KeyRegistryRequest {
                    id: Some(id.clone()),
                    revision: registered.revision,
                },
            )
            .unwrap();
        let moved = root.join("moved");
        std::fs::rename(&source, &moved).unwrap();
        assert_eq!(
            registry
                .snapshot(Principal::LocalUi)
                .unwrap()
                .keys
                .iter()
                .find(|key| key.id == id)
                .unwrap()
                .status,
            KeyStatus::Missing
        );
        let wrong = registry
            .preview(
                Principal::LocalUi,
                KeyImportRequest {
                    path: key.path.clone(),
                    label: "Wrong identity".into(),
                    storage: KeyStorage::Reference,
                },
                &OpenSshInspector,
            )
            .unwrap();
        assert!(
            registry
                .relink(
                    Principal::LocalUi,
                    KeyRelinkRequest {
                        revision: registered.revision.clone(),
                        id: id.clone(),
                        preview: wrong
                    },
                    &OpenSshInspector
                )
                .is_err()
        );
        let preview = registry
            .preview(
                Principal::LocalUi,
                KeyImportRequest {
                    path: moved.to_string_lossy().into(),
                    label: "Moved fixture".into(),
                    storage: KeyStorage::Reference,
                },
                &OpenSshInspector,
            )
            .unwrap();
        let relinked = registry
            .relink(
                Principal::LocalUi,
                KeyRelinkRequest {
                    revision: registered.revision.clone(),
                    id: id.clone(),
                    preview: preview.clone(),
                },
                &OpenSshInspector,
            )
            .unwrap();
        assert_eq!(relinked.default_key.as_deref(), Some(id.as_str()));
        let key = relinked.keys.iter().find(|key| key.id == id).unwrap();
        assert_eq!(key.public, fingerprint);
        assert_eq!(key.status, KeyStatus::Available);
        assert_eq!(Path::new(&key.path), moved);
        assert!(
            registry
                .relink(
                    Principal::LocalUi,
                    KeyRelinkRequest {
                        revision: registered.revision,
                        id,
                        preview
                    },
                    &OpenSshInspector
                )
                .is_err()
        );
    }

    #[test]
    fn native_openssh_checks_fresh_plain_and_encrypted_fixtures_without_keychain() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let mut registry = FileKeyRegistry::open(&root.join("registry")).unwrap();
        for (name, passphrase) in [("plain", ""), ("encrypted", "public-test-fixture-only")] {
            let path = root.join(name);
            // Public fixture text only, never a user passphrase or existing key.
            assert!(
                Command::new("/usr/bin/ssh-keygen")
                    .args(["-q", "-t", "ed25519", "-N", passphrase, "-f"])
                    .arg(&path)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .unwrap()
                    .success()
            );
            let preview = registry
                .preview(
                    Principal::LocalUi,
                    KeyImportRequest {
                        path: path.to_string_lossy().into(),
                        label: name.into(),
                        storage: KeyStorage::Reference,
                    },
                    &OpenSshInspector,
                )
                .unwrap();
            assert_eq!(preview.public.algorithm, "ED25519");
            assert!(preview.public.fingerprint.starts_with("SHA256:"));
            assert_eq!(preview.encrypted, !passphrase.is_empty());
            let revision = registry.snapshot(Principal::LocalUi).unwrap().revision;
            assert_eq!(
                registry
                    .commit(
                        Principal::LocalUi,
                        KeyCommitRequest { preview, revision },
                        &OpenSshInspector
                    )
                    .unwrap()
                    .keys
                    .len(),
                if name == "plain" { 1 } else { 2 }
            );
        }
        let invalid = root.join("invalid");
        std::fs::write(
            &invalid,
            "-----BEGIN OPENSSH PRIVATE KEY-----\nbm9wZQ==\n-----END OPENSSH PRIVATE KEY-----\n",
        )
        .unwrap();
        std::fs::set_permissions(&invalid, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(OpenSshInspector.inspect(&invalid).is_err());
    }
}
