//! Bounded, revision-bound authoring. No arbitrary filesystem write endpoint.
use super::{
    Workspace,
    model::{Manifest, entity_id, invalid},
    storage,
};
use crate::{
    automation::*,
    contracts::*,
    policy::{Principal, authorize_edit},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AutomationReadRequest {
    pub context: RevisionRequest,
    pub id: String,
    pub kind: AutomationKind,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AutomationMutation {
    pub context: RevisionRequest,
    pub document: AutomationDocument,
    pub replace: bool,
    pub remove: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct AutomationPreview {
    pub digest: String,
    pub validation: AutomationValidation,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AutomationCommit {
    pub mutation: AutomationMutation,
    pub digest: String,
}
pub(super) fn source_kind(manifest: &Manifest, path: &str) -> Option<AutomationKind> {
    let path = Path::new(path);
    let filename = path.file_name()?.to_str()?;
    if path.parent() == Some(Path::new(&manifest.paths.playbooks)) {
        if filename.ends_with(".ansible.yml") {
            Some(AutomationKind::Ansible)
        } else if filename.ends_with(".yml") || filename.ends_with(".yaml") {
            Some(AutomationKind::Playbook)
        } else {
            None
        }
    } else if path.parent() == Some(Path::new(&manifest.paths.scripts)) {
        match path.extension()?.to_str()? {
            "sh" => Some(AutomationKind::Shell),
            "py" => Some(AutomationKind::Python),
            "ts" | "js" => Some(AutomationKind::Deno),
            _ => None,
        }
    } else {
        None
    }
}
fn source_id(path: &str, kind: AutomationKind) -> Option<String> {
    let name = Path::new(path).file_name()?.to_str()?;
    let id = if kind == AutomationKind::Ansible {
        name.strip_suffix(".ansible.yml")?
    } else {
        Path::new(name).file_stem()?.to_str()?
    };
    entity_id(id).then(|| id.to_owned())
}
impl Workspace {
    fn automation_files(
        &self,
        principal: Principal,
        context: &RevisionRequest,
    ) -> Result<Vec<(String, AutomationKind, String)>> {
        authorize_edit(principal)?;
        self.check(&context.workspace_id, &context.revision)?;
        let manifest = self
            .loaded
            .manifest
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::UnsupportedSchema))?;
        let mut files = vec![];
        let mut ids = BTreeSet::new();
        for folder in [&manifest.paths.playbooks, &manifest.paths.scripts] {
            storage::safe_path(&self.dir, folder)?;
            for entry in self.dir.read_dir(folder)? {
                let entry = entry?;
                let path = Path::new(folder)
                    .join(entry.file_name())
                    .to_str()
                    .ok_or_else(invalid)?
                    .to_owned();
                if let Some(kind) = source_kind(manifest, &path) {
                    let id = source_id(&path, kind).ok_or_else(invalid)?;
                    // Ambiguous IDs (foo.yml/foo.yaml or foo.js/foo.ts) are not runnable.
                    if !ids.insert((kind.extension(), id.clone())) {
                        return Err(AppError::new(ErrorCode::Conflict));
                    }
                    files.push((id, kind, path));
                }
            }
        }
        files.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(files)
    }
    pub fn automations(
        &self,
        principal: Principal,
        context: &RevisionRequest,
    ) -> Result<Vec<AutomationSummary>> {
        self.automation_files(principal, context)?
            .into_iter()
            .map(|(id, kind, path)| {
                let document = AutomationDocument {
                    id,
                    kind,
                    source: storage::read_text(&self.dir, &path)?,
                };
                Ok(AutomationSummary {
                    id: document.id.clone(),
                    kind,
                    validation: validate(&document),
                })
            })
            .collect()
    }
    pub fn automation_read(
        &self,
        principal: Principal,
        request: &AutomationReadRequest,
    ) -> Result<AutomationDocument> {
        let (_, kind, path) = self
            .automation_files(principal, &request.context)?
            .into_iter()
            .find(|(id, kind, _)| id == &request.id && kind == &request.kind)
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        Ok(AutomationDocument {
            id: request.id.clone(),
            kind,
            source: storage::read_text(&self.dir, &path)?,
        })
    }
    fn automation_prepare(
        &self,
        principal: Principal,
        mutation: &AutomationMutation,
    ) -> Result<(String, AutomationPreview)> {
        let files = self.automation_files(principal, &mutation.context)?;
        let document = &mutation.document;
        if !entity_id(&document.id) {
            return Err(invalid());
        }
        let existing = files
            .iter()
            .find(|(id, kind, _)| id == &document.id && kind == &document.kind);
        if existing.is_some() && !mutation.replace || mutation.remove && existing.is_none() {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        let manifest = self.loaded.manifest.as_ref().ok_or_else(invalid)?;
        let folder = if document.kind.is_script() {
            &manifest.paths.scripts
        } else {
            &manifest.paths.playbooks
        };
        let path = existing
            .map(|(_, _, path)| path.clone())
            .unwrap_or_else(|| format!("{folder}/{}.{}", document.id, document.kind.extension()));
        let validation = if mutation.remove {
            AutomationValidation {
                valid: true,
                name: document.id.clone(),
                task_count: 0,
                issues: vec![],
            }
        } else {
            validate(document)
        };
        let binding = serde_json::to_vec(&(mutation, &path)).map_err(|_| invalid())?;
        Ok((
            path,
            AutomationPreview {
                digest: storage::hash(&binding),
                validation,
            },
        ))
    }
    pub fn automation_preview(
        &self,
        principal: Principal,
        mutation: &AutomationMutation,
    ) -> Result<AutomationPreview> {
        Ok(self.automation_prepare(principal, mutation)?.1)
    }
    pub fn automation_commit(
        &mut self,
        principal: Principal,
        request: &AutomationCommit,
        cancel: &Cancellation,
    ) -> Result<WorkspaceSnapshot> {
        let (path, preview) = self.automation_prepare(principal, &request.mutation)?;
        if preview.digest != request.digest {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        if !preview.validation.valid {
            return Err(invalid());
        }
        self.loaded = storage::atomic_file(
            &self.dir,
            &request.mutation.context.revision,
            &path,
            (!request.mutation.remove).then_some(request.mutation.document.source.as_str()),
            cancel,
        )?;
        Ok(self.snapshot())
    }
}
