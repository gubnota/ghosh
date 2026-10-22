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
    node.auth.key_ref = None;
    assert_eq!(select_identity(&node, None, &keys).source, "system");
    node.auth.agent_socket = Some("/tmp/custom-agent".into());
    assert_eq!(select_identity(&node, None, &keys).source, "agent");
    node.auth.agent_socket = None;
    node.schema = 2;
    assert_eq!(select_identity(&node, None, &keys).source, "agent");
    node.auth.method = AuthMethod::System;
    assert_eq!(
        select_identity(&node, Some(WORKSPACE), &keys).source,
        "system"
    );
    assert!(select_identity(&node, None, &keys).key_id.is_none());
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
    assert!(saved.trusted);
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

fn save_node(ws: &mut Workspace, change: NodeChange) {
    let mutation = NodeMutationRequest {
        context: context(ws),
        change,
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
}

#[test]
fn uppercase_node_ids_round_trip_inventory_and_jump_references_in_both_schemas() {
    for schema in [1, 2] {
        let temp = tempfile::tempdir().unwrap();
        let locks = temp.path().join("locks");
        let mut ws =
            Workspace::create(temp.path(), "Names", &locks, &Cancellation::default()).unwrap();
        let root = ws.root().to_owned();
        if schema == 2 {
            let request = WorkspaceKeyRequest {
                context: context(&ws),
                default_key: None,
            };
            let preview = ws
                .preview_workspace_key(Principal::LocalUi, &request)
                .unwrap();
            ws.commit_workspace_key(
                Principal::LocalUi,
                &WorkspaceKeyCommit {
                    request,
                    digest: preview.digest,
                },
                &Cancellation::default(),
            )
            .unwrap();
        }
        let mut gpu = node();
        gpu.id = "RTX3070".into();
        gpu.schema = schema;
        gpu.auth.method = AuthMethod::System;
        save_node(&mut ws, NodeChange::Create { node: gpu.clone() });
        let mut worker = gpu.clone();
        worker.id = "Worker_1-GPU".into();
        worker.jump = Some(gpu.id.clone());
        save_node(
            &mut ws,
            NodeChange::Create {
                node: worker.clone(),
            },
        );
        gpu.host = "192.0.2.30".into();
        save_node(&mut ws, NodeChange::Update { node: gpu.clone() });
        drop(ws);
        let mut ws = Workspace::open(&root, &locks).unwrap();
        let nodes = ws.node_details(Principal::LocalUi, &context(&ws)).unwrap();
        assert_eq!(
            nodes.iter().find(|n| n.id == gpu.id).unwrap().host,
            gpu.host
        );
        assert_eq!(
            nodes
                .iter()
                .find(|n| n.id == worker.id)
                .unwrap()
                .jump
                .as_deref(),
            Some("RTX3070")
        );
        assert!(root.join("nodes/RTX3070.toml").is_file());
        let mutation = NodeMutationRequest {
            context: context(&ws),
            change: NodeChange::Remove { id: gpu.id.clone() },
        };
        assert_eq!(
            ws.preview_node(Principal::LocalUi, &mutation)
                .unwrap_err()
                .code,
            ErrorCode::MissingReference
        );
        worker.jump = Some("rtx3070".into());
        let mutation = NodeMutationRequest {
            context: context(&ws),
            change: NodeChange::Update {
                node: worker.clone(),
            },
        };
        assert_eq!(
            ws.preview_node(Principal::LocalUi, &mutation)
                .unwrap_err()
                .code,
            ErrorCode::MissingReference
        );
        save_node(&mut ws, NodeChange::Remove { id: worker.id });
        save_node(&mut ws, NodeChange::Remove { id: gpu.id.clone() });
        let source = format!(
            "# preserve import\nschema={schema}\nid='RTX3070'\nhost='192.0.2.31'\nuser='ops'\ncustom='intact'\n[auth]\nmethod='system'\n"
        );
        save_node(
            &mut ws,
            NodeChange::Import {
                source: source.clone(),
                replace: false,
            },
        );
        gpu.host = "192.0.2.32".into();
        save_node(&mut ws, NodeChange::Update { node: gpu.clone() });
        let patched = fs::read_to_string(root.join("nodes/RTX3070.toml")).unwrap();
        assert!(patched.contains("# preserve import"));
        assert!(patched.contains("custom='intact'"));
        save_node(
            &mut ws,
            NodeChange::Import {
                source: source.clone(),
                replace: true,
            },
        );
        assert_eq!(
            fs::read_to_string(root.join("nodes/RTX3070.toml")).unwrap(),
            source
        );
        drop(ws);
        let mut ws = Workspace::open(&root, &locks).unwrap();
        assert_eq!(ws.snapshot().nodes[0].id, "RTX3070");
        save_node(&mut ws, NodeChange::Remove { id: gpu.id });
        drop(ws);
        assert!(
            Workspace::open(&root, &locks)
                .unwrap()
                .snapshot()
                .nodes
                .is_empty()
        );
    }
}

#[test]
fn case_only_node_collisions_fail_without_changing_saved_definition() {
    let temp = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(
        temp.path(),
        "Names",
        &temp.path().join("locks"),
        &Cancellation::default(),
    )
    .unwrap();
    let mut gpu = node();
    gpu.schema = 1;
    gpu.auth.method = AuthMethod::System;
    gpu.id = "RTX3070".into();
    save_node(&mut ws, NodeChange::Create { node: gpu.clone() });
    let original = fs::read(ws.root().join("nodes/RTX3070.toml")).unwrap();
    let revision = ws.snapshot().revision;
    gpu.id = "rtx3070".into();
    let source = toml_edit::ser::to_string_pretty(&gpu).unwrap();
    for change in [
        NodeChange::Create { node: gpu.clone() },
        NodeChange::Update { node: gpu.clone() },
        NodeChange::Import {
            source: source.clone(),
            replace: false,
        },
        NodeChange::Import {
            source,
            replace: true,
        },
        NodeChange::Remove { id: gpu.id },
    ] {
        let mutation = NodeMutationRequest {
            context: context(&ws),
            change,
        };
        let error = ws.preview_node(Principal::LocalUi, &mutation).unwrap_err();
        assert_eq!(error.code, ErrorCode::Conflict);
        assert!(error.message.contains("different capitalization"));
        let error = ws
            .commit_node(
                Principal::LocalUi,
                &NodeCommitRequest {
                    mutation,
                    digest: "unreviewed".into(),
                },
                &Cancellation::default(),
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Conflict);
        assert_eq!(ws.snapshot().revision, revision);
        assert_eq!(
            fs::read(ws.root().join("nodes/RTX3070.toml")).unwrap(),
            original
        );
    }
}

#[test]
fn node_id_rules_are_specific_and_do_not_relax_other_entity_ids() {
    use ghosh_core::workspace::model::{entity_id, validate_node_id};
    for id in ["RTX3070", "9_GPU-Node", &"A".repeat(64)] {
        assert!(validate_node_id(id).is_ok());
    }
    for id in [
        "",
        "_GPU",
        "-GPU",
        "GPU name",
        "../GPU",
        "GПU",
        &"A".repeat(65),
    ] {
        let error = validate_node_id(id).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidDocument);
        assert!(
            error
                .message
                .starts_with("Node IDs must use 1–64 ASCII letters")
        );
        let mut invalid = node();
        invalid.id = id.into();
        assert_eq!(invalid.validate(id).unwrap_err().message, error.message);
        invalid.id = "valid".into();
        invalid.jump = Some(id.into());
        if !id.is_empty() {
            assert_eq!(
                invalid.validate("valid").unwrap_err().message,
                error.message
            );
        }
    }
    assert!(!entity_id("RTX3070"));
    let mut invalid = node();
    invalid.labels.push("GPU".into());
    assert!(invalid.validate("edge").is_err());
    invalid.labels.clear();
    invalid.tunnel = "GPU".into();
    assert!(invalid.validate("edge").is_err());
}
