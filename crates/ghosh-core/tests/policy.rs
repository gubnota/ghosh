use ghosh_core::{
    contracts::{AppError, ErrorCode},
    policy::{Principal, REQUIRED_APPROVALS, SecretAccess, needs_approval, validate_approvals},
    secrets::{SecretRef, SecretValue},
};
#[test]
fn supplied_default_approval_set_covers_b1_and_browser_actions() {
    let all: Vec<String> = REQUIRED_APPROVALS.iter().map(|s| (*s).into()).collect();
    validate_approvals(&all).unwrap();
    for tool in [
        "profile.open_tab",
        "page.eval",
        "page.click",
        "page.type",
        "page.run",
    ] {
        assert!(needs_approval(tool, false));
        assert!(needs_approval(tool, true));
    }
    assert!(!needs_approval("node.run", true));
    assert!(needs_approval("node.run", false));
    assert!(needs_approval("node.exec", true));
    for index in 0..all.len() {
        let mut incomplete = all.clone();
        incomplete.remove(index);
        assert!(validate_approvals(&incomplete).is_err());
    }
}
#[test]
fn secret_access_requires_trusted_native_service_not_ui_or_agent() {
    let id = "01J9Z3K8X0Q5T7R2M4N6P8V1WA";
    for principal in [Principal::LocalUi, Principal::Script, Principal::Mcp] {
        assert!(SecretAccess::authorize(principal, id, true).is_err());
    }
    assert!(SecretAccess::authorize(Principal::NativeService, id, false).is_err());
    assert!(SecretAccess::authorize(Principal::NativeService, id, true).is_ok());
}
#[test]
fn reference_and_error_envelopes_never_include_invalid_input() {
    for bad in [
        "SECRET_CANARY_9",
        "secret://keychain/../../escape",
        "secret://keychain/",
        "secret://keychain/a?b",
    ] {
        assert!(SecretRef::parse(bad).is_err());
    }
    assert_eq!(
        SecretRef::parse("secret://keychain/ssh-gpu1")
            .unwrap()
            .account(),
        "ssh-gpu1"
    );
    assert!(
        !serde_json::to_string(&AppError::new(ErrorCode::SecretUnavailable))
            .unwrap()
            .contains("SECRET_CANARY_9")
    );
    let value = SecretValue::new(b"SECRET_CANARY_9".to_vec());
    assert_eq!(value.expose(), b"SECRET_CANARY_9");
}
