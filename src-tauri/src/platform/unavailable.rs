use ghosh_core::{
    contracts::{AppError, ErrorCode, Result},
    policy::SecretAccess,
    secrets::{SecretRef, SecretStore, SecretValue},
};

/// Legacy Keychain references never fall back to plaintext storage on Linux.
/// The file-based SSH key library is a separate service.
pub struct UnavailableStore;
impl SecretStore for UnavailableStore {
    fn resolve(&self, _: &SecretRef, _: &SecretAccess) -> Result<SecretValue> {
        Err(AppError::new(ErrorCode::SecretUnavailable))
    }
    fn put(&self, _: &SecretRef, _: &SecretValue, _: &SecretAccess) -> Result<()> {
        Err(AppError::new(ErrorCode::SecretUnavailable))
    }
    fn delete(&self, _: &SecretRef, _: &SecretAccess) -> Result<()> {
        Err(AppError::new(ErrorCode::SecretUnavailable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghosh_core::policy::Principal;

    #[test]
    fn legacy_secrets_fail_closed_for_all_operations() {
        let reference = SecretRef::parse("secret://keychain/test-unavailable").unwrap();
        let access =
            SecretAccess::authorize(Principal::NativeService, "01J9Z3K8X0Q5T7R2M4N6P8V1WA", true)
                .unwrap();
        let store = UnavailableStore;
        assert_eq!(
            store.resolve(&reference, &access).err().unwrap().code,
            ErrorCode::SecretUnavailable
        );
        assert_eq!(
            store
                .put(
                    &reference,
                    &SecretValue::new(b"test-only".to_vec()),
                    &access
                )
                .unwrap_err()
                .code,
            ErrorCode::SecretUnavailable
        );
        assert_eq!(
            store.delete(&reference, &access).unwrap_err().code,
            ErrorCode::SecretUnavailable
        );
    }
}
