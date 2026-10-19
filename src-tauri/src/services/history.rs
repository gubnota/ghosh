use ghosh_core::contracts::{AppError, ErrorCode, RecentWorkspace, Result};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct WorkspaceHistory {
    pub recent: Vec<RecentWorkspace>,
    pub last: Option<String>,
}
impl WorkspaceHistory {
    pub fn load(dir: &Path) -> Self {
        let path = dir.join("workspaces.json");
        // Once migrated, a closed/cleared or damaged state must never resurrect
        // entries from the old file.
        let mut state = if fs::symlink_metadata(&path).is_ok() {
            read_bounded(&path)
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or_default()
        } else {
            let recent: Vec<RecentWorkspace> = read_bounded(&dir.join("recent.json"))
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or_default();
            Self {
                last: recent.first().map(|entry| entry.path.clone()),
                recent,
            }
        };
        state.recent.retain(|entry| {
            !entry.path.is_empty() && entry.path.len() <= 4096 && entry.name.len() <= 480
        });
        let mut seen = std::collections::HashSet::new();
        state.recent.retain(|entry| seen.insert(entry.path.clone()));
        state.recent.truncate(10);
        if !state
            .recent
            .iter()
            .any(|entry| Some(&entry.path) == state.last.as_ref())
        {
            state.last = None;
        }
        state
    }
    pub fn remember(&mut self, name: &str, path: &str) {
        self.recent.retain(|entry| entry.path != path);
        self.recent.insert(
            0,
            RecentWorkspace {
                name: name.into(),
                path: path.into(),
            },
        );
        self.recent.truncate(10);
        self.last = Some(path.into());
    }
    pub fn forget(&mut self, path: Option<&str>) {
        self.recent
            .retain(|entry| path.is_some_and(|path| entry.path != path));
        if path.is_none() || self.last.as_deref() == path {
            self.last = None;
        }
    }
    pub fn save(&self, dir: &Path) -> Result<()> {
        let path = dir.join(format!(".workspaces-{}.tmp", uuid::Uuid::new_v4()));
        let result = (|| {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&path)?;
            file.write_all(
                &serde_json::to_vec(self).map_err(|_| AppError::new(ErrorCode::Internal))?,
            )?;
            file.sync_all()?;
            fs::rename(&path, dir.join("workspaces.json"))?;
            #[cfg(unix)]
            fs::File::open(dir)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(path);
        }
        result
    }
}
fn read_bounded(path: &Path) -> Option<Vec<u8>> {
    use std::io::Read;
    if !fs::symlink_metadata(path).ok()?.file_type().is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .ok()?
        .take(64 * 1024 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() <= 64 * 1024).then_some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrate_restore_close_and_forget_without_touching_workspaces() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("Example.ghosh");
        fs::create_dir(&workspace).unwrap();
        let path = workspace.to_str().unwrap();
        fs::write(
            dir.path().join("recent.json"),
            serde_json::to_vec(&vec![RecentWorkspace {
                name: "Example".into(),
                path: path.into(),
            }])
            .unwrap(),
        )
        .unwrap();
        let mut history = WorkspaceHistory::load(dir.path());
        assert_eq!(history.last.as_deref(), Some(path));
        history.last = None;
        history.save(dir.path()).unwrap();
        let mut history = WorkspaceHistory::load(dir.path());
        assert!(history.last.is_none());
        assert_eq!(history.recent.len(), 1);
        history.remember("Renamed", path);
        history.save(dir.path()).unwrap();
        let mut history = WorkspaceHistory::load(dir.path());
        assert_eq!(history.last.as_deref(), Some(path));
        assert_eq!(history.recent[0].name, "Renamed");
        history.forget(Some(path));
        history.save(dir.path()).unwrap();
        assert!(WorkspaceHistory::load(dir.path()).recent.is_empty());
        assert!(workspace.exists());
    }
    #[test]
    fn bounded_mru_clear_and_failed_save_preserve_previous_state() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = WorkspaceHistory::default();
        for n in 0..12 {
            history.remember(&format!("W{n}"), &format!("/W{n}.ghosh"));
        }
        assert_eq!(history.recent.len(), 10);
        history.remember("Again", "/W5.ghosh");
        assert_eq!(history.recent.len(), 10);
        assert_eq!(history.recent[0].name, "Again");
        history.save(dir.path()).unwrap();
        assert!(history.save(&dir.path().join("missing")).is_err());
        assert_eq!(WorkspaceHistory::load(dir.path()).recent.len(), 10);
        history.forget(None);
        history.save(dir.path()).unwrap();
        let reloaded = WorkspaceHistory::load(dir.path());
        assert!(reloaded.last.is_none());
        assert!(reloaded.recent.is_empty());
    }
}
