use ghosh_core::{
    contracts::*,
    policy::Principal,
    workspace::{
        Workspace,
        inventory::*,
        model::{Auth, AuthMethod, Node},
    },
};
use std::fs;
fn fixture() -> (tempfile::TempDir, Workspace) {
    let tmp = tempfile::tempdir().unwrap();
    let ws = Workspace::create(
        tmp.path(),
        "Inventory",
        &tmp.path().join("locks"),
        &Cancellation::default(),
    )
    .unwrap();
    (tmp, ws)
}
fn node(id: &str) -> Node {
    Node {
        schema: 1,
        id: id.into(),
        host: "127.0.0.1".into(),
        port: 22,
        user: "tester".into(),
        labels: vec![],
        tunnel: "direct".into(),
        jump: None,
        auth: Auth {
            method: AuthMethod::Agent,
            key_ref: None,
            key_id: None,
            password_ref: None,
            agent_socket: None,
        },
    }
}
fn request(ws: &Workspace, change: NodeChange) -> NodeMutationRequest {
    let s = ws.snapshot();
    NodeMutationRequest {
        context: RevisionRequest {
            workspace_id: s.id,
            revision: s.revision,
        },
        change,
    }
}
fn commit(ws: &mut Workspace, change: NodeChange) -> WorkspaceSnapshot {
    let mutation = request(ws, change);
    let preview = ws.preview_node(Principal::LocalUi, &mutation).unwrap();
    ws.commit_node(
        Principal::LocalUi,
        &NodeCommitRequest {
            mutation,
            digest: preview.digest,
        },
        &Cancellation::default(),
    )
    .unwrap()
}
#[test]
fn preview_is_inert_and_create_update_reopen_preserve_unknown_fields_and_comments() {
    let (tmp, mut ws) = fixture();
    let source = "# my node\nschema=1\nid='edge1'\nhost='127.0.0.1' # host note\nuser='tester'\ncustom='keep'\nauth={method='agent', custom='also keep'}\n";
    let mutation = request(
        &ws,
        NodeChange::Import {
            source: source.into(),
            replace: false,
        },
    );
    let p = ws.preview_node(Principal::LocalUi, &mutation).unwrap();
    assert!(!ws.root().join("nodes/edge1.toml").exists());
    assert_eq!(p.after.as_ref().unwrap().id, "edge1");
    ws.commit_node(
        Principal::LocalUi,
        &NodeCommitRequest {
            mutation,
            digest: p.digest,
        },
        &Cancellation::default(),
    )
    .unwrap();
    let s = ws.snapshot();
    ws.set_trusted(Principal::LocalUi, &s.id, &s.revision, true)
        .unwrap();
    let mut n = node("edge1");
    n.host = "localhost".into();
    n.labels = vec!["web".into()];
    let s = commit(&mut ws, NodeChange::Update { node: n });
    assert!(!s.trusted);
    let saved = fs::read_to_string(ws.root().join("nodes/edge1.toml")).unwrap();
    for preserved in ["# my node", "# host note", "custom='keep'", "also keep"] {
        assert!(saved.contains(preserved), "{saved}");
    }
    let path = ws.root().to_owned();
    drop(ws);
    let reopened = Workspace::open(&path, &tmp.path().join("locks")).unwrap();
    assert_eq!(reopened.snapshot().nodes[0].host, "localhost");
}
#[test]
fn graph_checks_reject_missing_routes_and_jumps_cycles_and_referenced_removal() {
    let (_tmp, mut ws) = fixture();
    let mut n = node("bad");
    n.tunnel = "absent".into();
    assert_eq!(
        ws.preview_node(
            Principal::LocalUi,
            &request(&ws, NodeChange::Create { node: n })
        )
        .unwrap_err()
        .code,
        ErrorCode::MissingReference
    );
    commit(
        &mut ws,
        NodeChange::Create {
            node: node("edge1"),
        },
    );
    let mut n = node("edge2");
    n.jump = Some("edge1".into());
    commit(&mut ws, NodeChange::Create { node: n });
    assert_eq!(
        ws.preview_node(
            Principal::LocalUi,
            &request(&ws, NodeChange::Remove { id: "edge1".into() })
        )
        .unwrap_err()
        .code,
        ErrorCode::MissingReference
    );
    let mut n = node("edge1");
    n.jump = Some("edge2".into());
    assert_eq!(
        ws.preview_node(
            Principal::LocalUi,
            &request(&ws, NodeChange::Update { node: n })
        )
        .unwrap_err()
        .code,
        ErrorCode::MissingReference
    );
    commit(&mut ws, NodeChange::Remove { id: "edge2".into() });
    assert_eq!(ws.snapshot().nodes.len(), 1);
    assert!(fs::read_dir(ws.root().join("nodes")).unwrap().any(|e| {
        e.unwrap()
            .path()
            .extension()
            .is_some_and(|x| x == "removed")
    }));
}
#[test]
fn duplicate_import_requires_explicit_replace_and_changed_drafts_cannot_reuse_preview() {
    let (_tmp, mut ws) = fixture();
    commit(
        &mut ws,
        NodeChange::Create {
            node: node("edge1"),
        },
    );
    let source = fs::read_to_string(ws.root().join("nodes/edge1.toml")).unwrap();
    assert_eq!(
        ws.preview_node(
            Principal::LocalUi,
            &request(
                &ws,
                NodeChange::Import {
                    source: source.clone(),
                    replace: false
                }
            )
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
    let mut mutation = request(
        &ws,
        NodeChange::Import {
            source,
            replace: true,
        },
    );
    let p = ws.preview_node(Principal::LocalUi, &mutation).unwrap();
    if let NodeChange::Import { source, .. } = &mut mutation.change {
        source.push_str("\nextra='changed'\n");
    }
    assert_eq!(
        ws.commit_node(
            Principal::LocalUi,
            &NodeCommitRequest {
                mutation,
                digest: p.digest
            },
            &Cancellation::default()
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
}
#[test]
fn stale_disk_cancel_foreign_callers_and_unsafe_sources_have_zero_writes() {
    let (_tmp, mut ws) = fixture();
    let mutation = request(
        &ws,
        NodeChange::Create {
            node: node("edge1"),
        },
    );
    for principal in [Principal::Script, Principal::Mcp] {
        assert_eq!(
            ws.preview_node(principal, &mutation).unwrap_err().code,
            ErrorCode::Denied
        );
    }
    let p = ws.preview_node(Principal::LocalUi, &mutation).unwrap();
    let commit = NodeCommitRequest {
        mutation,
        digest: p.digest,
    };
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        ws.commit_node(Principal::LocalUi, &commit, &cancel)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert!(!ws.root().join("nodes/edge1.toml").exists());
    let manifest = ws.root().join("ghosh.toml");
    let mut text = fs::read_to_string(&manifest).unwrap();
    text.push_str("\n# external\n");
    fs::write(manifest, text).unwrap();
    assert_eq!(
        ws.commit_node(Principal::LocalUi, &commit, &Cancellation::default())
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
    assert!(!ws.root().join("nodes/edge1.toml").exists());
    let s = ws.snapshot();
    ws.reload(Principal::LocalUi, &s.id, &s.revision).unwrap();
    for source in [
        "schema=1\nid='../bad'\nhost='localhost'\nuser='me'\n[auth]\nmethod='agent'",
        "schema=1\nid='bad'\nhost='localhost'\nuser='me'\n[auth]\nmethod='agent'\npassword='do not store'",
    ] {
        assert!(
            ws.preview_node(
                Principal::LocalUi,
                &request(
                    &ws,
                    NodeChange::Import {
                        source: source.into(),
                        replace: false
                    }
                )
            )
            .is_err()
        );
    }
}
#[cfg(unix)]
#[test]
fn symlink_substitution_between_preview_and_commit_cannot_write_outside_package() {
    use std::os::unix::fs::symlink;
    let (tmp, mut ws) = fixture();
    let mutation = request(
        &ws,
        NodeChange::Create {
            node: node("edge1"),
        },
    );
    let p = ws.preview_node(Principal::LocalUi, &mutation).unwrap();
    let outside = tmp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::remove_dir(ws.root().join("nodes")).unwrap();
    symlink(&outside, ws.root().join("nodes")).unwrap();
    assert!(
        ws.commit_node(
            Principal::LocalUi,
            &NodeCommitRequest {
                mutation,
                digest: p.digest
            },
            &Cancellation::default()
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}
