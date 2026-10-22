//! Explicit one-file schema migration and inert SSH identity selection.
use super::{
    Workspace,
    document::Document,
    model::{AuthMethod, Node, invalid, key_id},
    storage,
};
use crate::{
    contracts::*,
    policy::{Principal, authorize_edit},
    ssh_keys::{KeyStatus, KeysSnapshot, PublicKeyMetadata},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceKeyRequest {
    pub context: RevisionRequest,
    pub default_key: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspaceKeyPreview {
    pub digest: String,
    pub before_schema: u32,
    pub before_default: Option<String>,
    pub after_default: Option<String>,
    pub backup: String,
    pub legacy_nodes: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceKeyCommit {
    pub request: WorkspaceKeyRequest,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct IdentityPreview {
    pub source: String,
    pub key_id: Option<String>,
    pub public: Option<PublicKeyMetadata>,
    pub status: Option<KeyStatus>,
    pub registry_revision: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NodeIdentityRequest {
    pub context: RevisionRequest,
    pub node: Node,
}
/// Metadata only. Explicit missing assignments never fall through to defaults.
pub fn select_identity(
    node: &Node,
    workspace_default: Option<&str>,
    keys: &KeysSnapshot,
) -> IdentityPreview {
    let mut result = IdentityPreview {
        source: String::new(),
        key_id: None,
        public: None,
        status: None,
        registry_revision: keys.revision.clone(),
    };
    let choice = match node.effective_auth_method() {
        AuthMethod::System => {
            result.source = "system".into();
            None
        }
        AuthMethod::Agent => {
            result.source = "agent".into();
            None
        }
        AuthMethod::Password => {
            result.source = "password".into();
            None
        }
        AuthMethod::Key if node.schema < 2 => {
            result.source = "legacy_keychain".into();
            None
        }
        AuthMethod::Key => {
            let (source, id) = if let Some(id) = node.auth.key_id.as_deref() {
                ("node", Some(id))
            } else if let Some(id) = workspace_default {
                ("workspace", Some(id))
            } else {
                ("global", keys.default_key.as_deref())
            };
            result.source = source.into();
            id
        }
    };
    if let Some(id) = choice {
        result.key_id = Some(id.into());
        if let Some(key) = keys.keys.iter().find(|key| key.id == id) {
            result.public = Some(key.public.clone());
            result.status = Some(key.status);
        } else {
            result.status = Some(KeyStatus::Missing);
        }
    } else if matches!(node.auth.method, AuthMethod::Key) && node.schema >= 2 {
        result.status = Some(KeyStatus::Missing);
    }
    result
}
impl Workspace {
    pub fn workspace_key(
        &self,
        principal: Principal,
        context: &RevisionRequest,
    ) -> Result<Option<String>> {
        authorize_edit(principal)?;
        self.check(&context.workspace_id, &context.revision)?;
        Ok(self
            .loaded
            .manifest
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::UnsupportedSchema))?
            .workspace
            .default_key
            .clone())
    }
    fn prepare_key(
        &self,
        principal: Principal,
        request: &WorkspaceKeyRequest,
    ) -> Result<(String, WorkspaceKeyPreview)> {
        authorize_edit(principal)?;
        self.check(&request.context.workspace_id, &request.context.revision)?;
        let manifest = self
            .loaded
            .manifest
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::UnsupportedSchema))?;
        if request.default_key.as_ref().is_some_and(|id| !key_id(id)) {
            return Err(invalid());
        }
        let mut doc = Document::parse(&self.loaded.doc.text())?;
        let decor = doc.ast["schema"]
            .as_value()
            .ok_or_else(invalid)?
            .decor()
            .clone();
        doc.ast["schema"] = toml_edit::value(2);
        *doc.ast["schema"]
            .as_value_mut()
            .ok_or_else(invalid)?
            .decor_mut() = decor;
        let workspace = doc.ast["workspace"].as_table_mut().ok_or_else(invalid)?;
        match &request.default_key {
            Some(id) => {
                workspace.insert("default_key", toml_edit::value(id));
            }
            None => {
                workspace.remove("default_key");
            }
        }
        let text = doc.text();
        storage::preview_file(&self.dir, "ghosh.toml", Some(&text))?;
        let digest = storage::hash(&serde_json::to_vec(&(request, &text)).map_err(|_| invalid())?);
        let preview = WorkspaceKeyPreview {
            digest,
            before_schema: manifest.schema,
            before_default: manifest.workspace.default_key.clone(),
            after_default: request.default_key.clone(),
            backup: format!(".ghosh-manifest-{}.backup", request.context.revision),
            legacy_nodes: self
                .loaded
                .nodes
                .iter()
                .filter(|node| node.schema == 1 && matches!(node.auth.method, AuthMethod::Key))
                .map(|node| node.id.clone())
                .collect(),
        };
        Ok((text, preview))
    }
    pub fn preview_workspace_key(
        &self,
        principal: Principal,
        request: &WorkspaceKeyRequest,
    ) -> Result<WorkspaceKeyPreview> {
        Ok(self.prepare_key(principal, request)?.1)
    }
    pub fn commit_workspace_key(
        &mut self,
        principal: Principal,
        request: &WorkspaceKeyCommit,
        cancel: &Cancellation,
    ) -> Result<WorkspaceSnapshot> {
        let (text, preview) = self.prepare_key(principal, &request.request)?;
        if preview.digest != request.digest {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        cancel.check()?;
        // Publish a complete recoverable backup first. This changes only the manifest;
        // legacy nodes remain inert and are migrated individually by explicit editing.
        storage::backup_manifest(
            &self.dir,
            &request.request.context.revision,
            &preview.backup,
        )?;
        let candidate = storage::atomic_file(
            &self.dir,
            &request.request.context.revision,
            "ghosh.toml",
            Some(&text),
            cancel,
        )?;
        self.loaded = candidate;
        Ok(self.snapshot())
    }
}
