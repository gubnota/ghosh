use crate::contracts::{AppError, ErrorCode, Result};
use zeroize::Zeroizing;

/// Never serializable; Debug is deliberately absent.
pub struct SecretValue(Zeroizing<Vec<u8>>);
impl SecretValue {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }
    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct SecretRef(String);
impl SecretRef {
    pub fn parse(value: &str) -> Result<Self> {
        let id = value
            .strip_prefix("secret://keychain/")
            .ok_or_else(|| AppError::new(ErrorCode::InvalidDocument))?;
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        {
            return Err(AppError::new(ErrorCode::InvalidDocument));
        }
        Ok(Self(id.into()))
    }
    pub fn account(&self) -> &str {
        &self.0
    }
}
/// Only native services receive this port; never exposed over desktop IPC.
pub trait SecretStore: Send + Sync {
    fn resolve(
        &self,
        reference: &SecretRef,
        access: &crate::policy::SecretAccess,
    ) -> Result<SecretValue>;
    fn put(
        &self,
        reference: &SecretRef,
        value: &SecretValue,
        access: &crate::policy::SecretAccess,
    ) -> Result<()>;
    fn delete(&self, reference: &SecretRef, access: &crate::policy::SecretAccess) -> Result<()>;
}
