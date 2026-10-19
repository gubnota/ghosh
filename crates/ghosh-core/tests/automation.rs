use ghosh_core::{
    automation::*,
    contracts::*,
    policy::Principal,
    workspace::{Workspace, automation::*},
};
use std::fs;
const SOURCE: &str = "# keep comments 世界\nschema: 1\nname: Inspect\ntasks:\n  - name: Facts\n    check: {command: 'uname -a'}\n    shell: {command: 'true'}\n";
fn fixture() -> (tempfile::TempDir, Workspace) {
    let tmp = tempfile::tempdir().unwrap();
    let ws = Workspace::create(
        tmp.path(),
        "Automation",
        &tmp.path().join("locks"),
        &Cancellation::default(),
    )
    .unwrap();
    (tmp, ws)
}
fn context(ws: &Workspace) -> RevisionRequest {
    let snapshot = ws.snapshot();
    RevisionRequest {
        workspace_id: snapshot.id,
        revision: snapshot.revision,
    }
}
fn mutation(ws: &Workspace) -> AutomationMutation {
    AutomationMutation {
        context: context(ws),
        document: AutomationDocument {
            id: "facts".into(),
            kind: AutomationKind::Playbook,
            source: SOURCE.into(),
        },
        replace: false,
        remove: false,
    }
}
fn reviewed(ws: &Workspace, mutation: AutomationMutation) -> AutomationCommit {
    let preview = ws
        .automation_preview(Principal::LocalUi, &mutation)
        .unwrap();
    AutomationCommit {
        mutation,
        digest: preview.digest,
    }
}
#[test]
fn script_id_error_is_specific_and_correction_preserves_source() {
    let (_tmp, mut ws) = fixture();
    let mut change = mutation(&ws);
    change.document = AutomationDocument {
        id: "create a file".into(),
        kind: AutomationKind::Shell,
        source: "#!/bin/sh\ncd ~\necho '123' > out.txt\n".into(),
    };
    assert_eq!(validate(&change.document).issues, [AUTOMATION_ID_HELP]);
    let error = ws
        .automation_preview(Principal::LocalUi, &change)
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidDocument);
    assert_eq!(error.message, AUTOMATION_ID_HELP);
    assert_eq!(fs::read_dir(ws.root().join("scripts")).unwrap().count(), 0);
    change.document.id = "create-a-file".into();
    let exact = change.document.source.clone();
    let request = reviewed(&ws, change);
    ws.automation_commit(Principal::LocalUi, &request, &Cancellation::default())
        .unwrap();
    assert_eq!(
        fs::read_to_string(ws.root().join("scripts/create-a-file.sh")).unwrap(),
        exact
    );
}
#[test]
fn source_roundtrip_revision_binding_and_recoverable_removal() {
    let (tmp, mut ws) = fixture();
    let request = reviewed(&ws, mutation(&ws));
    assert!(!ws.root().join("playbooks/facts.yml").exists());
    ws.automation_commit(Principal::LocalUi, &request, &Cancellation::default())
        .unwrap();
    assert_eq!(
        fs::read_to_string(ws.root().join("playbooks/facts.yml")).unwrap(),
        SOURCE
    );
    let root = ws.root().to_owned();
    drop(ws);
    let mut ws = Workspace::open(&root, &tmp.path().join("locks")).unwrap();
    let doc = ws
        .automation_read(
            Principal::LocalUi,
            &AutomationReadRequest {
                context: context(&ws),
                id: "facts".into(),
                kind: AutomationKind::Playbook,
            },
        )
        .unwrap();
    assert_eq!(doc.source, SOURCE);
    assert_eq!(
        ws.automations(Principal::LocalUi, &context(&ws)).unwrap()[0]
            .validation
            .task_count,
        1
    );
    let mut remove = mutation(&ws);
    remove.replace = true;
    remove.remove = true;
    let request = reviewed(&ws, remove);
    ws.automation_commit(Principal::LocalUi, &request, &Cancellation::default())
        .unwrap();
    assert!(
        ws.automations(Principal::LocalUi, &context(&ws))
            .unwrap()
            .is_empty()
    );
    assert!(fs::read_dir(root.join("playbooks")).unwrap().any(|entry| {
        entry
            .unwrap()
            .path()
            .extension()
            .is_some_and(|ext| ext == "removed")
    }));
}
#[test]
fn changed_source_disk_cancel_and_foreign_callers_cannot_write() {
    let (_tmp, mut ws) = fixture();
    for principal in [Principal::Script, Principal::Mcp] {
        assert_eq!(
            ws.automation_preview(principal, &mutation(&ws))
                .unwrap_err()
                .code,
            ErrorCode::Denied
        );
    }
    let mut request = reviewed(&ws, mutation(&ws));
    request.mutation.document.source.push_str("# changed\n");
    assert_eq!(
        ws.automation_commit(Principal::LocalUi, &request, &Cancellation::default())
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
    let request = reviewed(&ws, mutation(&ws));
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        ws.automation_commit(Principal::LocalUi, &request, &cancel)
            .unwrap_err()
            .code,
        ErrorCode::Cancelled
    );
    assert!(!ws.root().join("playbooks/facts.yml").exists());
    fs::write(ws.root().join("scripts/external.sh"), "echo external\n").unwrap();
    assert_eq!(
        ws.automation_commit(Principal::LocalUi, &request, &Cancellation::default())
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
}
#[test]
fn invalid_sources_open_inert_but_cannot_be_committed_and_duplicates_are_rejected() {
    let (tmp, ws) = fixture();
    let root = ws.root().to_owned();
    drop(ws);
    fs::write(
        root.join("playbooks/facts.yaml"),
        "{{ malformed : secret-value",
    )
    .unwrap();
    let mut ws = Workspace::open(&root, &tmp.path().join("locks")).unwrap();
    let list = ws.automations(Principal::LocalUi, &context(&ws)).unwrap();
    assert!(!list[0].validation.valid);
    assert!(!list[0].validation.issues.join(" ").contains("secret-value"));
    let mut change = mutation(&ws);
    change.replace = true;
    change.document.source = "invalid".into();
    let request = reviewed(&ws, change);
    assert_eq!(
        ws.automation_commit(Principal::LocalUi, &request, &Cancellation::default())
            .unwrap_err()
            .code,
        ErrorCode::InvalidDocument
    );
    drop(ws);
    fs::write(root.join("playbooks/facts.yml"), SOURCE).unwrap();
    let ws = Workspace::open(&root, &tmp.path().join("locks")).unwrap();
    assert_eq!(
        ws.automations(Principal::LocalUi, &context(&ws))
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
}
#[test]
fn task_contract_and_ansible_link_validation_are_bounded_and_inert() {
    assert!(parse_playbook(SOURCE).is_ok());
    for source in [
        SOURCE.replace("schema: 1", "schema: 2"),
        SOURCE.replace("    check: {command: 'uname -a'}\n", ""),
        SOURCE.replace("    shell:", "    timeout_s: 0\n    shell:"),
        SOURCE.replace("    shell:", "    notify: [missing]\n    shell:"),
        SOURCE.replace("    shell:", "    unknown: true\n    shell:"),
    ] {
        assert!(parse_playbook(&source).is_err());
    }
    let link = "schema: 1\nname: Existing\nproject: /nonexistent/project\nplaybook: setup.yml\ninventory: hosts.ini\n";
    assert!(parse_ansible(link).is_ok());
    assert!(parse_ansible(&link.replace("setup.yml", "../outside.yml")).is_err());
    assert!(parse_ansible(&link.replace("/nonexistent/project", "relative/project")).is_err());
    let mut doc = AutomationDocument {
        id: "../outside".into(),
        kind: AutomationKind::Shell,
        source: "echo okay".into(),
    };
    assert!(!validate(&doc).valid);
    doc.id = "safe".into();
    doc.source = "x".repeat(MAX_SOURCE + 1);
    assert!(!validate(&doc).valid);
}
#[cfg(unix)]
#[test]
fn symlink_sources_cannot_escape_workspace() {
    let (tmp, ws) = fixture();
    let root = ws.root().to_owned();
    drop(ws);
    let outside = tmp.path().join("outside.sh");
    fs::write(&outside, "echo outside").unwrap();
    std::os::unix::fs::symlink(&outside, root.join("scripts/escape.sh")).unwrap();
    assert!(Workspace::open(&root, &tmp.path().join("locks")).is_err());
    assert_eq!(fs::read_to_string(outside).unwrap(), "echo outside");
}

#[test]
fn standard_ansible_is_preserved_without_a_ghosh_task_schema() {
    let source = "# upstream syntax\n- name: Original\n  hosts: web\n  roles: [existing_role]\n  vars: { message: '{{ inventory_hostname }}' }\n  tasks:\n    - ansible.builtin.include_tasks: tasks.yml\n    - block:\n        - ansible.builtin.debug: {msg: '{{ message }}'}\n      rescue:\n        - ansible.builtin.fail: {msg: failed}\n  handlers:\n    - name: Reload\n      ansible.builtin.service: {name: nginx, state: reloaded}\n";
    let (_tmp, mut workspace) = fixture();
    let mut change = mutation(&workspace);
    change.document.source = source.into();
    assert!(uses_ansible(&change.document));
    assert!(validate(&change.document).valid);
    let commit = reviewed(&workspace, change);
    workspace
        .automation_commit(Principal::LocalUi, &commit, &Cancellation::default())
        .unwrap();
    assert_eq!(
        fs::read_to_string(workspace.root().join("playbooks/facts.yml")).unwrap(),
        source
    );
    assert!(!uses_ansible(&AutomationDocument {
        id: "legacy".into(),
        kind: AutomationKind::Playbook,
        source: SOURCE.into()
    }));
}
