use ghosh_core::{
    contracts::*,
    policy::Principal,
    ssh_keys::*,
    workspace::{
        Workspace,
        identity::*,
        inventory::*,
        model::{Auth, AuthMethod, Node},
    },
};
use std::fs;
const GLOBAL: &str = "dfe02cbc-75d1-40ef-aa3d-7b3e2fd5266f";
const WORKSPACE: &str = "44c5d5eb-6443-487f-b4cf-3840363ad6f3";
const NODE: &str = "d6f3ea7e-a875-4e66-82d7-c6384c93f710";
fn context(ws: &Workspace) -> RevisionRequest {
    let s = ws.snapshot();
    RevisionRequest {
        workspace_id: s.id,
        revision: s.revision,
    }
}
fn node() -> Node {
    Node {
        schema: 2,
        id: "edge".into(),
        host: "127.0.0.1".into(),
        port: 22,
        user: "ops".into(),
        auth: Auth {
            method: AuthMethod::Key,
            key_ref: None,
            key_id: None,
            password_ref: None,
            agent_socket: None,
        },
        labels: vec![],
        tunnel: "direct".into(),
        jump: None,
    }
}
fn keys() -> KeysSnapshot {
    KeysSnapshot {
        revision: "keys-revision".into(),
        default_key: Some(GLOBAL.into()),
        keys: [GLOBAL, WORKSPACE, NODE]
            .into_iter()
            .map(|id| KeySummary {
                id: id.into(),
                label: id.into(),
                path: "/unused".into(),
                storage: KeyStorage::Reference,
                public: PublicKeyMetadata {
                    algorithm: "ED25519".into(),
                    fingerprint: format!("SHA256:{id}"),
                },
                encrypted: false,
                status: KeyStatus::Available,
            })
            .collect(),
    }
}
#[test]
fn precedence_is_explicit_and_missing_or_changed_identity_never_falls_back() {
    let mut node = node();
    let mut keys = keys();
    assert_eq!(
        select_identity(&node, None, &keys).key_id.as_deref(),
        Some(GLOBAL)
    );
    assert_eq!(
        select_identity(&node, Some(WORKSPACE), &keys).source,
        "workspace"
    );
    node.auth.key_id = Some(NODE.into());
    let selected = select_identity(&node, Some(WORKSPACE), &keys);
    assert_eq!(selected.key_id.as_deref(), Some(NODE));
    assert_eq!(selected.source, "node");
    keys.keys.retain(|key| key.id != NODE);
    assert_eq!(
        select_identity(&node, Some(WORKSPACE), &keys).status,
        Some(KeyStatus::Missing)
    );
    assert_eq!(
        select_identity(&node, Some(WORKSPACE), &keys)
            .key_id
            .as_deref(),
        Some(NODE)
    );
    node.auth.key_id = None;
    keys.keys[1].status = KeyStatus::Changed;
    assert_eq!(
        select_identity(&node, Some(WORKSPACE), &keys).status,
        Some(KeyStatus::Changed)
    );
    node.schema = 1;
    node.auth.key_ref = Some("secret://keychain/old".into());
    assert_eq!(
        select_identity(&node, None, &keys).source,
        "legacy_keychain"
    );
    assert!(select_identity(&node, None, &keys).key_id.is_none());
    node.auth.method = AuthMethod::Agent;
    assert_eq!(select_identity(&node, None, &keys).source, "agent");
}
#[test]
fn migration_is_reviewed_single_file_backed_up_and_preserves_legacy_nodes() {
    let temp = tempfile::tempdir().unwrap();
    let locks = temp.path().join("locks");
    let mut ws =
        Workspace::create(temp.path(), "Migration", &locks, &Cancellation::default()).unwrap();
    let root = ws.root().to_owned();
    let legacy = "# leave me intact\nschema=1\nid='legacy'\nhost='127.0.0.1'\nuser='ops'\n[auth]\nmethod='key'\nkey_ref='secret://keychain/old'\n";
    fs::write(root.join("nodes/legacy.toml"), legacy).unwrap();
    let s = ws.snapshot();
    ws.reload(Principal::LocalUi, &s.id, &s.revision).unwrap();
    let original = fs::read(root.join("ghosh.toml")).unwrap();
    let request = WorkspaceKeyRequest {
        context: context(&ws),
        default_key: Some(WORKSPACE.into()),
    };
    let preview = ws
        .preview_workspace_key(Principal::LocalUi, &request)
        .unwrap();
    assert_eq!(preview.before_schema, 1);
    assert_eq!(preview.legacy_nodes, ["legacy"]);
    assert_eq!(fs::read(root.join("ghosh.toml")).unwrap(), original);
    assert!(!root.join(&preview.backup).exists());
    let bad = WorkspaceKeyCommit {
        request: request.clone(),
        digest: "forged".into(),
    };
    assert!(
        ws.commit_workspace_key(Principal::LocalUi, &bad, &Cancellation::default())
            .is_err()
    );
    let saved = ws
        .commit_workspace_key(
            Principal::LocalUi,
            &WorkspaceKeyCommit {
                request,
                digest: preview.digest,
            },
            &Cancellation::default(),
        )
        .unwrap();
    assert_eq!(saved.schema, 2);
    assert!(!saved.read_only);
    assert!(!saved.trusted);
    assert_eq!(fs::read(root.join(&preview.backup)).unwrap(), original);
    assert_eq!(
        fs::read_to_string(root.join("nodes/legacy.toml")).unwrap(),
        legacy
    );
    assert_eq!(
        ws.workspace_key(Principal::LocalUi, &context(&ws))
            .unwrap()
            .as_deref(),
        Some(WORKSPACE)
    );
    let mutation = NodeMutationRequest {
        context: context(&ws),
        change: NodeChange::Create { node: node() },
    };
    let preview = ws.preview_node(Principal::LocalUi, &mutation).unwrap();
    ws.commit_node(
        Principal::LocalUi,
        &NodeCommitRequest {
            mutation,
            digest: preview.digest,
        },
        &Cancellation::default(),
    )
    .unwrap();
    drop(ws);
    let ws = Workspace::open(&root, &locks).unwrap();
    assert_eq!(ws.snapshot().nodes.len(), 2);
    assert!(!ws.snapshot().read_only);
    assert_eq!(
        ws.workspace_key(Principal::LocalUi, &context(&ws))
            .unwrap()
            .as_deref(),
        Some(WORKSPACE)
    );
}
#[test]
fn schema_boundaries_and_stale_or_foreign_migrations_fail_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(
        temp.path(),
        "Safety",
        &temp.path().join("locks"),
        &Cancellation::default(),
    )
    .unwrap();
    let root = ws.root().to_owned();
    let mutation = NodeMutationRequest {
        context: context(&ws),
        change: NodeChange::Create { node: node() },
    };
    assert!(ws.preview_node(Principal::LocalUi, &mutation).is_err());
    let request = WorkspaceKeyRequest {
        context: context(&ws),
        default_key: None,
    };
    assert!(
        ws.preview_workspace_key(Principal::NativeService, &request)
            .is_err()
    );
    let preview = ws
        .preview_workspace_key(Principal::LocalUi, &request)
        .unwrap();
    let source = fs::read_to_string(root.join("ghosh.toml")).unwrap();
    fs::write(root.join("ghosh.toml"), format!("{source}\n# external\n")).unwrap();
    assert!(
        ws.commit_workspace_key(
            Principal::LocalUi,
            &WorkspaceKeyCommit {
                request,
                digest: preview.digest
            },
            &Cancellation::default()
        )
        .is_err()
    );
    assert!(!root.join(preview.backup).exists());
    let mut node = node();
    node.schema = 1;
    assert!(node.validate("edge").is_err());
    node.schema = 2;
    node.auth.key_id = Some("/private/key".into());
    assert!(node.validate("edge").is_err());
    node.auth.key_id = None;
    node.auth.key_ref = Some("secret://keychain/old".into());
    assert!(node.validate("edge").is_err());
}
