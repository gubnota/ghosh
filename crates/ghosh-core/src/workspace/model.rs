use crate::{
    contracts::{AppError, ErrorCode, Result},
    policy::{REQUIRED_APPROVALS, validate_approvals},
    secrets::SecretRef,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path};

pub fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidDocument)
}
pub fn entity_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && (id.as_bytes()[0].is_ascii_lowercase() || id.as_bytes()[0].is_ascii_digit())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b))
}
pub fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.len() > 120 || name.chars().any(char::is_control) {
        return Err(invalid());
    }
    Ok(())
}
pub fn relative_path(value: &str) -> Result<()> {
    if value.is_empty()
        || value.contains('\\')
        || !Path::new(value)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
    {
        return Err(AppError::new(ErrorCode::InvalidPath));
    }
    Ok(())
}
fn port() -> u16 {
    22
}
fn direct() -> String {
    "direct".into()
}
fn approval_defaults() -> Vec<String> {
    REQUIRED_APPROVALS.iter().map(|s| (*s).into()).collect()
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct WorkspaceMetadata {
    pub name: String,
    pub id: String,
    #[schemars(with = "String")]
    pub created: toml_edit::Datetime,
    pub default_profile: Option<String>,
    pub default_tunnel: Option<String>,
    pub default_key: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct Paths {
    pub nodes: String,
    pub profiles: String,
    pub tunnels: String,
    pub certs: String,
    pub playbooks: String,
    pub recordings: String,
    pub recipes: String,
    pub state: String,
    pub scripts: String,
    pub templates: String,
    pub data: String,
}
impl Default for Paths {
    fn default() -> Self {
        Self {
            nodes: "nodes".into(),
            profiles: "profiles".into(),
            tunnels: "tunnels".into(),
            certs: "certs".into(),
            playbooks: "playbooks".into(),
            recordings: "recordings".into(),
            recipes: "recipes".into(),
            state: "state".into(),
            scripts: "scripts".into(),
            templates: "templates".into(),
            data: "data".into(),
        }
    }
}
impl Paths {
    pub fn all(&self) -> [&str; 11] {
        [
            &self.nodes,
            &self.profiles,
            &self.tunnels,
            &self.certs,
            &self.playbooks,
            &self.recordings,
            &self.recipes,
            &self.state,
            &self.scripts,
            &self.templates,
            &self.data,
        ]
    }
    pub fn validate(&self) -> Result<()> {
        let all = self.all();
        for (i, a) in all.iter().enumerate() {
            relative_path(a)?;
            for b in &all[..i] {
                if Path::new(a).starts_with(b) || Path::new(b).starts_with(a) {
                    return Err(AppError::new(ErrorCode::InvalidPath));
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct Mcp {
    pub enabled: bool,
    pub read_only: bool,
    pub require_approval: Vec<String>,
}
impl Default for Mcp {
    fn default() -> Self {
        Self {
            enabled: false,
            read_only: true,
            require_approval: approval_defaults(),
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Manifest {
    #[schemars(range(min = 1))]
    pub schema: u32,
    pub workspace: WorkspaceMetadata,
    #[serde(default)]
    pub paths: Paths,
    #[serde(default)]
    pub mcp: Mcp,
}
impl Manifest {
    pub fn validate(&self) -> Result<()> {
        if !(1..=2).contains(&self.schema) {
            return Err(AppError::new(ErrorCode::UnsupportedSchema));
        }
        validate_name(&self.workspace.name)?;
        if ulid::Ulid::from_string(&self.workspace.id)
            .map(|id| id.to_string() != self.workspace.id)
            .unwrap_or(true)
        {
            return Err(invalid());
        }
        let created = self.workspace.created.to_string();
        if time::OffsetDateTime::parse(&created, &time::format_description::well_known::Rfc3339)
            .map(|t| t.offset() != time::UtcOffset::UTC)
            .unwrap_or(true)
        {
            return Err(invalid());
        }
        if let Some(id) = &self.workspace.default_key
            && (self.schema < 2 || !key_id(id))
        {
            return Err(invalid());
        }
        self.paths.validate()?;
        validate_approvals(&self.mcp.require_approval)?;
        for id in [
            &self.workspace.default_profile,
            &self.workspace.default_tunnel,
        ]
        .into_iter()
        .flatten()
        {
            if !entity_id(id) {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Key,
    Agent,
    Password,
}
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, ts_rs::TS)]
pub struct Auth {
    pub method: AuthMethod,
    pub key_ref: Option<String>,
    pub key_id: Option<String>,
    pub password_ref: Option<String>,
    pub agent_socket: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, ts_rs::TS)]
pub struct Node {
    pub schema: u32,
    pub id: String,
    pub host: String,
    #[serde(default = "port")]
    pub port: u16,
    pub user: String,
    pub auth: Auth,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default = "direct")]
    pub tunnel: String,
    pub jump: Option<String>,
}
impl Node {
    pub fn validate(&self, file_id: &str) -> Result<()> {
        if !(1..=2).contains(&self.schema)
            || !entity_id(&self.id)
            || self.id != file_id
            || self.port == 0
            || self.host.is_empty()
            || self.host.len() > 253
            || self
                .host
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || "/@\\".contains(c))
            || self.user.is_empty()
            || self.user.len() > 128
            || self.user.chars().any(char::is_control)
            || self.labels.len() > 64
            || self.labels.iter().any(|label| !entity_id(label))
        {
            return Err(invalid());
        }
        if !entity_id(&self.tunnel) {
            return Err(invalid());
        }
        if let Some(jump) = &self.jump
            && !jump.is_empty()
            && !entity_id(jump)
        {
            return Err(invalid());
        }
        if let Some(reference) = &self.auth.key_ref {
            SecretRef::parse(reference)?;
        }
        if let Some(reference) = &self.auth.password_ref {
            SecretRef::parse(reference)?;
        }
        if self
            .auth
            .key_id
            .as_ref()
            .is_some_and(|id| self.schema < 2 || !key_id(id))
            || self.schema == 2 && self.auth.key_ref.is_some()
            || !matches!(self.auth.method, AuthMethod::Key) && self.auth.key_id.is_some()
        {
            return Err(invalid());
        }
        match self.auth.method {
            AuthMethod::Key
                if (self.schema == 1 && self.auth.key_ref.is_none())
                    || self.auth.password_ref.is_some() =>
            {
                return Err(invalid());
            }
            AuthMethod::Password
                if self.auth.password_ref.is_none() || self.auth.key_ref.is_some() =>
            {
                return Err(invalid());
            }
            AuthMethod::Agent
                if self.auth.key_ref.is_some() || self.auth.password_ref.is_some() =>
            {
                return Err(invalid());
            }
            _ => (),
        }
        Ok(())
    }
}

/// Canonical opaque user-local key identity, never a path or secret URI.
pub fn key_id(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok_and(|value| value.to_string() == id)
}
