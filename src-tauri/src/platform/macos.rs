use ghosh_core::{
    contracts::{AppError, ErrorCode, Result},
    policy::SecretAccess,
    secrets::{SecretRef, SecretStore, SecretValue},
};
use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

pub struct KeychainStore;
fn service(access: &SecretAccess) -> String {
    format!("app.ghosh.workspace.{}", access.workspace_id())
}
impl SecretStore for KeychainStore {
    fn resolve(&self, reference: &SecretRef, access: &SecretAccess) -> Result<SecretValue> {
        get_generic_password(&service(access), reference.account())
            .map(SecretValue::new)
            .map_err(|_| AppError::new(ErrorCode::SecretUnavailable))
    }
    fn put(&self, reference: &SecretRef, value: &SecretValue, access: &SecretAccess) -> Result<()> {
        set_generic_password(&service(access), reference.account(), value.expose())
            .map_err(|_| AppError::new(ErrorCode::SecretUnavailable))
    }
    fn delete(&self, reference: &SecretRef, access: &SecretAccess) -> Result<()> {
        delete_generic_password(&service(access), reference.account())
            .map_err(|_| AppError::new(ErrorCode::SecretUnavailable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghosh_core::policy::Principal;
    #[test]
    #[ignore = "Requires an unlocked user Keychain; uses a temporary service and removes it."]
    fn native_keychain_round_trip_and_cleanup() {
        // A fresh service cannot overwrite a user's existing credential.
        let id = ghosh_core::workspace::storage::hash(
            format!("{:?}", std::time::SystemTime::now()).as_bytes(),
        );
        let account = SecretRef::parse(&format!("secret://keychain/test-{}", &id[..20])).unwrap();
        let access =
            SecretAccess::authorize(Principal::NativeService, "01J9Z3K8X0Q5T7R2M4N6P8V1WA", true)
                .unwrap();
        let store = KeychainStore;
        store
            .put(
                &account,
                &SecretValue::new(b"temporary-ghosh-test".to_vec()),
                &access,
            )
            .unwrap();
        let result = store.resolve(&account, &access);
        let cleanup = store.delete(&account, &access);
        assert_eq!(result.unwrap().expose(), b"temporary-ghosh-test");
        cleanup.unwrap();
        assert!(store.resolve(&account, &access).is_err());
    }
}
