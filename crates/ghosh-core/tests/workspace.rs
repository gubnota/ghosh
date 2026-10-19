use ghosh_core::{
    contracts::{Cancellation, ErrorCode},
    policy::Principal,
    workspace::{Workspace, document::Document, watch::WorkspaceWatch},
};
use std::{fs, path::PathBuf, sync::mpsc, time::Duration};
use tempfile::TempDir;

struct Fixture {
    tmp: TempDir,
    path: PathBuf,
    locks: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let locks = tmp.path().join("locks");
        let ws =
            Workspace::create(tmp.path(), "Homelab", &locks, &Cancellation::default()).unwrap();
        let path = ws.root().to_owned();
        drop(ws);
        Self { tmp, path, locks }
    }
    fn open(&self) -> Workspace {
        Workspace::open(&self.path, &self.locks).unwrap()
    }
    fn write(&self, path: &str, text: &str) {
        fs::write(self.path.join(path), text).unwrap();
    }
    fn manifest(&self) -> String {
        fs::read_to_string(self.path.join("ghosh.toml")).unwrap()
    }
    fn replace(&self, from: &str, to: &str) {
        self.write("ghosh.toml", &self.manifest().replace(from, to));
    }
    fn node(&self, id: &str, extra: &str) {
        self.write(&format!("nodes/{id}.toml"),&format!("schema=1\nid=\"{id}\"\nhost=\"example.com\"\nuser=\"root\"\n{extra}\n[auth]\nmethod=\"agent\"\n"));
    }
    fn fails(&self, code: ErrorCode) {
        assert_eq!(
            Workspace::open(&self.path, &self.locks).err().unwrap().code,
            code
        );
    }
}
#[test]
fn create_reopen_and_round_trip_unknown_fields_and_comments() {
    let f = Fixture::new();
    f.write(
        "ghosh.toml",
        &format!(
            "# keep this header\n{}\n[future]\ncustom = 'preserved' # keep this too\n",
            f.manifest()
        ),
    );
    let mut ws = f.open();
    let first = ws.snapshot();
    assert!(first.trusted);
    assert_eq!(first.nodes.len(), 0);
    let next = ws
        .rename(
            Principal::LocalUi,
            &first.id,
            &first.revision,
            "Lab two",
            &Cancellation::default(),
        )
        .unwrap();
    assert_ne!(next.revision, first.revision);
    drop(ws);
    assert_eq!(f.open().snapshot().name, "Lab two");
    let text = f.manifest();
    assert!(text.contains("# keep this header"));
    assert!(text.contains("custom = 'preserved' # keep this too"));
    assert!(f.path.join("trust/known_hosts").exists());
}
#[test]
fn duplicate_writer_and_copied_identity_are_rejected() {
    let f = Fixture::new();
    let _ws = f.open();
    f.fails(ErrorCode::Busy);
    let other = f.tmp.path().join("copy.ghosh");
    fs::create_dir(&other).unwrap();
    for name in [
        "nodes",
        "profiles",
        "tunnels",
        "certs",
        "playbooks",
        "recordings",
        "recipes",
        "state",
        "scripts",
        "templates",
        "data",
    ] {
        fs::create_dir(other.join(name)).unwrap();
    }
    fs::copy(f.path.join("ghosh.toml"), other.join("ghosh.toml")).unwrap();
    assert_eq!(
        Workspace::open(&other, &f.locks).err().unwrap().code,
        ErrorCode::Busy
    );
}
#[test]
fn external_edit_conflicts_without_overwrite_then_reload() {
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    f.replace("Homelab", "External");
    assert!(ws.snapshot().external_changed);
    assert_eq!(
        ws.rename(
            Principal::LocalUi,
            &s.id,
            &s.revision,
            "Local",
            &Cancellation::default()
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
    assert!(f.manifest().contains("External"));
    let fresh = ws.reload(Principal::LocalUi, &s.id, &s.revision).unwrap();
    assert_eq!(fresh.name, "External");
    assert!(fresh.trusted);
}
#[test]
fn invalid_external_edit_keeps_last_valid_snapshot_and_recovers() {
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    let valid = f.manifest();
    f.write("ghosh.toml", "broken = [");
    assert_eq!(
        ws.reload(Principal::LocalUi, &s.id, &s.revision)
            .unwrap_err()
            .code,
        ErrorCode::InvalidDocument
    );
    assert_eq!(ws.snapshot().name, "Homelab");
    assert!(ws.snapshot().external_changed);
    f.write("ghosh.toml", &valid);
    assert!(
        !ws.reload(Principal::LocalUi, &s.id, &s.revision)
            .unwrap()
            .external_changed
    );
}
#[test]
fn reload_rejects_replaced_package_without_adopting_old_directory() {
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    let moved = f.tmp.path().join("moved.ghosh");
    fs::rename(&f.path, &moved).unwrap();
    fs::create_dir(&f.path).unwrap();
    assert_eq!(
        ws.reload(Principal::LocalUi, &s.id, &s.revision)
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
    assert!(
        fs::read_to_string(moved.join("ghosh.toml"))
            .unwrap()
            .contains("Homelab")
    );
}
#[test]
fn nested_inline_secret_containers_are_rejected_without_echoing_values() {
    for text in [
        "schema=1\n[[extension]]\ntoken='CANARY'",
        "schema=1\nextension=[{ nested={ password='CANARY' } }]",
    ] {
        let error = Document::parse(text).err().unwrap();
        assert_eq!(error.code, ErrorCode::InvalidDocument);
        assert!(!format!("{error:?}").contains("CANARY"));
    }
}
#[test]
fn stale_revision_wrong_identity_and_non_ui_callers_cannot_write() {
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    for (principal, id, revision, code) in [
        (
            Principal::Mcp,
            s.id.as_str(),
            s.revision.as_str(),
            ErrorCode::Denied,
        ),
        (
            Principal::LocalUi,
            "wrong",
            s.revision.as_str(),
            ErrorCode::Denied,
        ),
        (
            Principal::LocalUi,
            s.id.as_str(),
            "stale",
            ErrorCode::Conflict,
        ),
    ] {
        assert_eq!(
            ws.rename(principal, id, revision, "No", &Cancellation::default())
                .unwrap_err()
                .code,
            code
        );
    }
    assert_eq!(ws.snapshot().name, "Homelab");
}
#[test]
fn newer_schema_is_inert_and_read_only() {
    let f = Fixture::new();
    f.replace("schema = 1", "schema = 3");
    let mut ws = f.open();
    let s = ws.snapshot();
    assert!(s.read_only);
    assert!(!s.trusted);
    assert_eq!(
        ws.rename(
            Principal::LocalUi,
            &s.id,
            &s.revision,
            "No",
            &Cancellation::default()
        )
        .unwrap_err()
        .code,
        ErrorCode::UnsupportedSchema
    );
}
#[test]
fn manifest_rejects_invalid_identity_time_and_approval_list() {
    let f = Fixture::new();
    let initial = f.manifest();
    for text in [
        initial.replace("schema = 1", "schema = 0"),
        initial.replace("name = \"Homelab\"", "name = \"\""),
        initial.replace("created = ", "created = 'invalid' # "),
        initial.replace("\"page.click\",", ""),
    ] {
        f.write("ghosh.toml", &text);
        f.fails(ErrorCode::InvalidDocument);
    }
}
#[test]
fn missing_default_profile_and_missing_or_cyclic_node_references_fail() {
    let f = Fixture::new();
    let initial = f.manifest();
    f.replace(
        "default_tunnel = \"direct\"",
        "default_profile = \"missing\"",
    );
    f.fails(ErrorCode::MissingReference);
    f.write("ghosh.toml", &initial);
    f.node("one", "jump=\"missing\"");
    f.fails(ErrorCode::MissingReference);
    f.node("one", "jump=\"two\"");
    f.node("two", "jump=\"one\"");
    f.fails(ErrorCode::MissingReference);
    f.node("two", "");
    assert_eq!(f.open().snapshot().nodes.len(), 2);
}
#[test]
fn invalid_node_filename_auth_and_inline_secret_are_rejected_without_leaks() {
    let f = Fixture::new();
    f.node("one", "");
    let original = fs::read_to_string(f.path.join("nodes/one.toml")).unwrap();
    for text in [
        original.replace("id=\"one\"", "id=\"other\""),
        original.replace("method=\"agent\"", "method=\"password\""),
        format!("{original}\npassword=\"SECRET_CANARY_9\"\n"),
    ] {
        f.write("nodes/one.toml", &text);
        let e = Workspace::open(&f.path, &f.locks).err().unwrap();
        assert_eq!(e.code, ErrorCode::InvalidDocument);
        assert!(!format!("{e:?}").contains("SECRET_CANARY_9"));
    }
}
#[test]
fn node_changes_participate_in_revision() {
    let f = Fixture::new();
    f.node("one", "");
    let mut ws = f.open();
    let s = ws.snapshot();
    f.node("two", "");
    assert_eq!(
        ws.rename(
            Principal::LocalUi,
            &s.id,
            &s.revision,
            "No",
            &Cancellation::default()
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
}
#[test]
fn profile_store_requires_uuid_and_is_immutable_while_loaded() {
    let f = Fixture::new();
    f.write(
        "profiles/work.toml",
        "schema=1\nid=\"work\"\ndata_store=\"01J9Z3K8X0Q5T7R2M4N6P8V1WA\"\n",
    );
    f.fails(ErrorCode::InvalidDocument);
    let initial = "schema=1\nid=\"work\"\ndata_store=\"3f2b8c1e-5a47-4d9e-9b6c-1e0a7d4c2f88\"\n";
    f.write("profiles/work.toml", initial);
    let mut ws = f.open();
    let s = ws.snapshot();
    f.write(
        "profiles/work.toml",
        &initial.replace("3f2b8c1e", "4f2b8c1e"),
    );
    assert_eq!(
        ws.reload(Principal::LocalUi, &s.id, &s.revision)
            .unwrap_err()
            .code,
        ErrorCode::InvalidDocument
    );
}
#[test]
fn duplicate_profile_stores_rejected() {
    let f = Fixture::new();
    for id in ["work", "other"] {
        f.write(
            &format!("profiles/{id}.toml"),
            &format!(
                "schema=1\nid=\"{id}\"\ndata_store=\"3f2b8c1e-5a47-4d9e-9b6c-1e0a7d4c2f88\"\n"
            ),
        );
    }
    f.fails(ErrorCode::MissingReference);
}
#[test]
fn paths_reject_traversal_absolute_and_overlapping_roots() {
    let f = Fixture::new();
    let original = f.manifest();
    for path in ["../outside", "/tmp/outside", "state/nodes", "state"] {
        f.write(
            "ghosh.toml",
            &original.replace("nodes = \"nodes\"", &format!("nodes = \"{path}\"")),
        );
        f.fails(ErrorCode::InvalidPath);
    }
}
#[cfg(unix)]
#[test]
fn symlink_escape_and_swap_after_open_never_write_outside() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    let outside = f.tmp.path().join("outside.toml");
    fs::write(&outside, "outside").unwrap();
    fs::remove_file(f.path.join("ghosh.toml")).unwrap();
    symlink(&outside, f.path.join("ghosh.toml")).unwrap();
    assert_eq!(
        ws.rename(
            Principal::LocalUi,
            &s.id,
            &s.revision,
            "No",
            &Cancellation::default()
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
    assert_eq!(fs::read_to_string(&outside).unwrap(), "outside");
    drop(ws);
    f.fails(ErrorCode::InvalidPath);
}
#[test]
fn cancellation_before_commit_leaves_manifest_and_target_untouched() {
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    let original = f.manifest();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        ws.rename(Principal::LocalUi, &s.id, &s.revision, "No", &cancel)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert_eq!(f.manifest(), original);
    assert_eq!(
        Workspace::create(f.tmp.path(), "Cancelled", &f.locks, &cancel)
            .err()
            .unwrap()
            .code,
        ErrorCode::Cancelled
    );
    assert!(!f.tmp.path().join("Cancelled.ghosh").exists());
}
#[test]
fn existing_target_and_partial_staging_are_not_overwritten_or_opened() {
    let f = Fixture::new();
    let original = f.manifest();
    assert_eq!(
        Workspace::create(f.tmp.path(), "Homelab", &f.locks, &Cancellation::default())
            .err()
            .unwrap()
            .code,
        ErrorCode::Conflict
    );
    assert_eq!(f.manifest(), original);
    let partial = f.tmp.path().join(".ghosh-create-interrupted");
    fs::create_dir(&partial).unwrap();
    fs::write(partial.join("ghosh.toml"), "broken").unwrap();
    assert_eq!(
        Workspace::open(&partial, &f.locks).err().unwrap().code,
        ErrorCode::InvalidPath
    );
    assert!(
        Workspace::create(
            f.tmp.path(),
            "Recovered",
            &f.locks,
            &Cancellation::default()
        )
        .is_ok()
    );
}
#[test]
fn local_approval_defaults_on_and_pause_survives_reload() {
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    assert!(s.trusted);
    ws.set_trusted(Principal::LocalUi, &s.id, &s.revision, false)
        .unwrap();
    f.replace("Homelab", "External");
    assert!(ws.snapshot().external_changed);
    assert!(!ws.snapshot().trusted);
    let fresh = ws.reload(Principal::LocalUi, &s.id, &s.revision).unwrap();
    assert!(!fresh.trusted);
    assert!(!fresh.external_changed);
    drop(ws);
    assert!(f.open().snapshot().trusted);
}
#[test]
fn disk_changes_keep_local_approval_but_still_block_writes() {
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    f.replace("Homelab", "External");
    assert!(ws.snapshot().trusted);
    assert!(ws.snapshot().external_changed);
    assert!(
        ws.rename(
            Principal::LocalUi,
            &s.id,
            &s.revision,
            "Unsafe",
            &Cancellation::default()
        )
        .is_err()
    );
}

#[test]
fn bounded_watcher_observes_external_change_and_stops_on_drop() {
    let f = Fixture::new();
    let (tx, rx) = mpsc::sync_channel(4);
    let watch = WorkspaceWatch::start(&f.path, move || {
        let _ = tx.try_send(());
    })
    .unwrap();
    f.replace("Homelab", "Changed");
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(watch);
    while rx.try_recv().is_ok() {}
    f.replace("Changed", "Later");
    assert!(rx.recv_timeout(Duration::from_millis(400)).is_err());
}
#[cfg(unix)]
#[test]
fn filesystem_failure_does_not_change_last_valid_snapshot_or_leave_temporary_file() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let mut ws = f.open();
    let s = ws.snapshot();
    let original = f.manifest();
    fs::set_permissions(&f.path, fs::Permissions::from_mode(0o500)).unwrap();
    let result = ws.rename(
        Principal::LocalUi,
        &s.id,
        &s.revision,
        "No",
        &Cancellation::default(),
    );
    fs::set_permissions(&f.path, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(result.unwrap_err().code, ErrorCode::Io);
    assert_eq!(ws.snapshot().name, "Homelab");
    assert_eq!(f.manifest(), original);
    assert!(
        !fs::read_dir(&f.path).unwrap().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp"))
    );
}
#[test]
fn comments_on_edited_name_survive() {
    let mut doc = Document::parse("schema=1\n[workspace]\nname='old' # name comment\n").unwrap();
    doc.rename("new").unwrap();
    assert!(doc.text().contains("# name comment"));
}
#[test]
fn create_rejects_path_names() {
    let f = Fixture::new();
    for name in ["../escape", "/tmp/escape", "..", ".hidden", "a/b"] {
        assert!(Workspace::create(f.tmp.path(), name, &f.locks, &Cancellation::default()).is_err());
    }
    assert!(!f.tmp.path().parent().unwrap().join("escape.ghosh").exists());
}
#[test]
fn document_size_bound_is_enforced() {
    let f = Fixture::new();
    f.write("ghosh.toml", &"#".repeat(1_048_577));
    f.fails(ErrorCode::InvalidDocument);
}
