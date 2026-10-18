//! Typed, inert node inventory changes. No arbitrary file-write IPC contract.
use super::{
    Workspace,
    document::Document,
    model::{Node, entity_id, invalid},
    storage,
};
use crate::{
    contracts::*,
    policy::{Principal, authorize_edit},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NodeChange {
    Create { node: Node },
    Update { node: Node },
    Import { source: String, replace: bool },
    Remove { id: String },
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NodeMutationRequest {
    pub context: RevisionRequest,
    pub change: NodeChange,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NodeCommitRequest {
    pub mutation: NodeMutationRequest,
    pub digest: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct NodeChangePreview {
    pub digest: String,
    pub before: Option<Node>,
    pub after: Option<Node>,
}
struct Prepared {
    path: String,
    source: Option<String>,
    preview: NodeChangePreview,
}
fn parse(source: &str) -> Result<Node> {
    let doc = Document::parse(source)?;
    let node: Node = toml_edit::de::from_document(doc.ast).map_err(|_| invalid())?;
    node.validate(&node.id)?;
    Ok(node)
}
fn patch_field(table: &mut dyn toml_edit::TableLike, source: &dyn toml_edit::TableLike, key: &str) {
    if let Some(value) = source.get(key) {
        let mut value = value.clone();
        if let (Some(old), Some(new)) = (
            table.get(key).and_then(toml_edit::Item::as_value),
            value.as_value_mut(),
        ) {
            *new.decor_mut() = old.decor().clone();
        }
        if let Some(current) = table.get_mut(key) {
            *current = value;
        } else {
            table.insert(key, value);
        }
    } else {
        table.remove(key);
    }
}
fn patch(source: &str, node: &Node) -> Result<String> {
    let mut doc = Document::parse(source)?;
    let before = serde_json::to_value(parse(source)?).map_err(|_| invalid())?;
    let after = serde_json::to_value(node).map_err(|_| invalid())?;
    let text = toml_edit::ser::to_string_pretty(node).map_err(|_| invalid())?;
    let next = Document::parse(&text)?;
    for key in [
        "schema", "id", "host", "port", "user", "labels", "tunnel", "jump",
    ] {
        if before[key] != after[key] {
            patch_field(doc.ast.as_table_mut(), next.ast.as_table(), key);
        }
    }
    let auth = doc
        .ast
        .get_mut("auth")
        .and_then(toml_edit::Item::as_table_like_mut)
        .ok_or_else(invalid)?;
    let next_auth = next
        .ast
        .get("auth")
        .and_then(toml_edit::Item::as_table_like)
        .ok_or_else(invalid)?;
    for key in [
        "method",
        "key_ref",
        "key_id",
        "password_ref",
        "agent_socket",
    ] {
        if before["auth"][key] != after["auth"][key] {
            patch_field(auth, next_auth, key);
        }
    }
    Ok(doc.text())
}
impl Workspace {
    pub fn node_details(
        &self,
        principal: Principal,
        request: &RevisionRequest,
    ) -> Result<Vec<Node>> {
        authorize_edit(principal)?;
        self.check(&request.workspace_id, &request.revision)?;
        if self.loaded.schema > 2 {
            return Err(AppError::new(ErrorCode::UnsupportedSchema));
        }
        Ok(self.loaded.nodes.clone())
    }
    fn prepare_node(
        &self,
        principal: Principal,
        request: &NodeMutationRequest,
    ) -> Result<Prepared> {
        authorize_edit(principal)?;
        self.check(&request.context.workspace_id, &request.context.revision)?;
        let manifest = self
            .loaded
            .manifest
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::UnsupportedSchema))?;
        let (id, source, replacing) = match &request.change {
            NodeChange::Create { node } => {
                node.validate(&node.id)?;
                (
                    node.id.clone(),
                    Some(toml_edit::ser::to_string_pretty(node).map_err(|_| invalid())?),
                    false,
                )
            }
            NodeChange::Update { node } => {
                node.validate(&node.id)?;
                if !self.loaded.nodes.iter().any(|n| n.id == node.id) {
                    return Err(invalid());
                }
                let path = format!("{}/{}.toml", manifest.paths.nodes, node.id);
                (
                    node.id.clone(),
                    Some(patch(&storage::read_text(&self.dir, &path)?, node)?),
                    true,
                )
            }
            NodeChange::Import { source, replace } => {
                let node = parse(source)?;
                (node.id, Some(source.clone()), *replace)
            }
            NodeChange::Remove { id } => (id.clone(), None, true),
        };
        if !entity_id(&id) {
            return Err(invalid());
        }
        let before = self.loaded.nodes.iter().find(|node| node.id == id).cloned();
        // Import replacement must be explicit; create never silently overwrites.
        if before.is_some() && !replacing || source.is_none() && before.is_none() {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        let path = format!("{}/{}.toml", manifest.paths.nodes, id);
        let candidate = storage::preview_file(&self.dir, &path, source.as_deref())?;
        let after = candidate.nodes.iter().find(|node| node.id == id).cloned();
        // Frame the exact bytes, operation, path and base revision. Recompute at
        // commit, so a modified draft or changed graph cannot reuse the preview.
        let binding = serde_json::to_vec(&(&request.context, &request.change, &path, &source))
            .map_err(|_| invalid())?;
        Ok(Prepared {
            path,
            source,
            preview: NodeChangePreview {
                digest: storage::hash(&binding),
                before,
                after,
            },
        })
    }
    pub fn preview_node(
        &self,
        principal: Principal,
        request: &NodeMutationRequest,
    ) -> Result<NodeChangePreview> {
        Ok(self.prepare_node(principal, request)?.preview)
    }
    pub fn commit_node(
        &mut self,
        principal: Principal,
        request: &NodeCommitRequest,
        cancel: &Cancellation,
    ) -> Result<WorkspaceSnapshot> {
        let prepared = self.prepare_node(principal, &request.mutation)?;
        if prepared.preview.digest != request.digest {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        let candidate = storage::atomic_file(
            &self.dir,
            &request.mutation.context.revision,
            &prepared.path,
            prepared.source.as_deref(),
            cancel,
        )?;
        self.loaded = candidate;
        // Configuration edits always revoke execution trust, even if the caller
        // only changed labels. Saving metadata does not connect to a node.
        self.trusted = false;
        Ok(self.snapshot())
    }
}
