use crate::services::Services;
use ghosh_core::{
    approvals::ApprovalDecisionRequest,
    contracts::*,
    operations::{JobKind, OperationsSnapshot},
    ssh::*,
    ssh_keys::*,
    terminal::*,
    workspace::{identity::*, inventory::*, model::Node},
};
use std::{path::Path, sync::Arc};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

pub(crate) fn validate_caller(
    label: &str,
    scheme: &str,
    host: Option<&str>,
    port: Option<u16>,
) -> Result<()> {
    let bundled = (scheme == "tauri" && host == Some("localhost"))
        || (scheme == "http" && host == Some("tauri.localhost"));
    let dev = cfg!(debug_assertions)
        && scheme == "http"
        && host == Some("127.0.0.1")
        && port == Some(1420);
    if label != "main" || !(bundled || dev) {
        return Err(AppError::new(ErrorCode::Denied));
    }
    Ok(())
}
fn caller(window: &WebviewWindow) -> Result<()> {
    let url = window.url().map_err(|_| AppError::new(ErrorCode::Denied))?;
    validate_caller(window.label(), url.scheme(), url.host_str(), url.port())
}
async fn run<T: Send + 'static>(f: impl FnOnce() -> Result<T> + Send + 'static) -> Result<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|_| AppError::new(ErrorCode::Internal))?
}
#[tauri::command]
pub async fn node_connection_preview(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: NodeConnectionRequest,
) -> Result<NodeConnectionPreview> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.connection_preview(request)).await
}
#[tauri::command]
pub async fn node_connection_test(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: NodeConnectionCommit,
) -> Result<NodeConnectionResult> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.connection_test(request)).await
}
#[tauri::command]
pub async fn terminal_open_node(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: NodeTerminalRequest,
) -> Result<TerminalSession> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.terminal_open_node(request)).await
}
#[tauri::command]
pub async fn terminal_open_local(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    dimensions: TerminalSize,
) -> Result<TerminalSession> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.terminals.open_local(dimensions)).await
}
#[tauri::command]
pub async fn terminal_poll(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    id: String,
    ack: u32,
) -> Result<TerminalPacket> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.terminals.poll(&id, ack)).await
}
#[tauri::command]
pub async fn terminal_input(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    id: String,
    bytes: Vec<u8>,
) -> Result<()> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.terminals.input(&id, bytes)).await
}
#[tauri::command]
pub async fn terminal_resize(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    id: String,
    dimensions: TerminalSize,
) -> Result<()> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.terminals.resize(&id, dimensions)).await
}
#[tauri::command]
pub async fn terminal_close(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    id: String,
) -> Result<()> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.terminals.close(&id)).await
}
#[tauri::command]
pub async fn keys_generate(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: KeyGenerateRequest,
) -> Result<KeysSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.generate_key(request)).await
}
#[tauri::command]
pub async fn keys_relink(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: KeyRelinkRequest,
) -> Result<KeysSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.relink_key(request)).await
}
#[tauri::command]
pub async fn workspace_key(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: RevisionRequest,
) -> Result<Option<String>> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.workspace_key(request)).await
}
#[tauri::command]
pub async fn workspace_key_preview(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: WorkspaceKeyRequest,
) -> Result<WorkspaceKeyPreview> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.preview_workspace_key(request)).await
}
#[tauri::command]
pub async fn workspace_key_commit(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: WorkspaceKeyCommit,
) -> Result<WorkspaceSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.commit_workspace_key(request)).await
}
#[tauri::command]
pub async fn node_identity(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: NodeIdentityRequest,
) -> Result<IdentityPreview> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.identity_preview(request)).await
}
#[tauri::command]
pub async fn keys_list(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
) -> Result<KeysSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.keys()).await
}
#[tauri::command]
pub async fn keys_preview(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: KeyImportRequest,
) -> Result<KeyImportPreview> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.preview_key(request)).await
}
#[tauri::command]
pub async fn keys_commit(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: KeyCommitRequest,
) -> Result<KeysSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.commit_key(request)).await
}
#[tauri::command]
pub async fn keys_default(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: KeyRegistryRequest,
) -> Result<KeysSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.default_key(request)).await
}
#[tauri::command]
pub async fn keys_forget(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: KeyRegistryRequest,
) -> Result<KeysSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.forget_key(request)).await
}
#[tauri::command]
pub async fn keys_pick(window: WebviewWindow) -> Result<Option<String>> {
    caller(&window)?;
    Ok(rfd::AsyncFileDialog::new()
        .set_title("Choose an existing SSH private key")
        .pick_file()
        .await
        .map(|p| p.path().to_string_lossy().into()))
}
fn tracked<T>(
    service: &Services,
    app: &AppHandle,
    kind: JobKind,
    action: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let job = service.jobs.begin(kind, false)?;
    let _ = app.emit_to("main", "operations-changed", ());
    let result = action();
    job.finish(&result)?;
    let _ = app.emit_to("main", "operations-changed", ());
    result
}
#[tauri::command]
pub async fn operations_snapshot(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
) -> Result<OperationsSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.operations()).await
}
#[tauri::command]
pub async fn jobs_cancel(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    id: String,
) -> Result<()> {
    caller(&window)?;
    state.jobs.cancel(&id)?;
    let _ = window.emit_to("main", "operations-changed", ());
    Ok(())
}
#[tauri::command]
pub async fn approvals_decide(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: ApprovalDecisionRequest,
) -> Result<()> {
    caller(&window)?;
    let service = state.inner().clone();
    let app = window.app_handle().clone();
    run(move || {
        let result = service.decide(request);
        let _ = app.emit_to("main", "operations-changed", ());
        result
    })
    .await
}
#[tauri::command]
pub async fn nodes_list(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: RevisionRequest,
) -> Result<Vec<Node>> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.node_details(request)).await
}
#[tauri::command]
pub async fn nodes_preview(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: NodeMutationRequest,
) -> Result<NodeChangePreview> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.preview_node(request)).await
}
#[tauri::command]
pub async fn nodes_commit(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: NodeCommitRequest,
) -> Result<WorkspaceSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    let app = window.app_handle().clone();
    run(move || {
        tracked(&service, &app, JobKind::InventorySave, || {
            service.commit_node(request)
        })
    })
    .await
}
#[tauri::command]
pub async fn workspace_startup(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
) -> Result<Option<WorkspaceSnapshot>> {
    caller(&window)?;
    let service = state.inner().clone();
    let app = window.app_handle().clone();
    run(move || service.startup(&app)).await
}
#[tauri::command]
pub async fn workspace_close(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: RevisionRequest,
) -> Result<()> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.close(request)).await
}
#[tauri::command]
pub async fn workspace_forget_recent(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    path: Option<String>,
) -> Result<Vec<RecentWorkspace>> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.forget_recent(path)).await
}
#[tauri::command]
pub async fn workspace_open(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    path: String,
) -> Result<WorkspaceSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    let app = window.app_handle().clone();
    run(move || {
        tracked(&service, &app, JobKind::WorkspaceOpen, || {
            service.open(Path::new(&path), &app)
        })
    })
    .await
}
#[tauri::command]
pub async fn workspace_create(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: CreateRequest,
) -> Result<WorkspaceSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    let app = window.app_handle().clone();
    run(move || {
        tracked(&service, &app, JobKind::WorkspaceCreate, || {
            service.create(request, &app)
        })
    })
    .await
}
#[tauri::command]
pub async fn workspace_snapshot(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
) -> Result<Option<WorkspaceSnapshot>> {
    caller(&window)?;
    let service = state.inner().clone();
    run(move || service.snapshot()).await
}
#[tauri::command]
pub async fn workspace_rename(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: RenameRequest,
) -> Result<WorkspaceSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    let app = window.app_handle().clone();
    run(move || {
        tracked(&service, &app, JobKind::WorkspaceSave, || {
            service.rename(request)
        })
    })
    .await
}
#[tauri::command]
pub async fn workspace_reload(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: RevisionRequest,
) -> Result<WorkspaceSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    let app = window.app_handle().clone();
    run(move || {
        tracked(&service, &app, JobKind::WorkspaceReload, || {
            service.reload(request)
        })
    })
    .await
}
#[tauri::command]
pub async fn workspace_trust(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
    request: TrustRequest,
) -> Result<WorkspaceSnapshot> {
    caller(&window)?;
    let service = state.inner().clone();
    let app = window.app_handle().clone();
    run(move || {
        tracked(&service, &app, JobKind::WorkspaceTrust, || {
            service.trust(request)
        })
    })
    .await
}
#[tauri::command]
pub async fn workspace_recent(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
) -> Result<Vec<RecentWorkspace>> {
    caller(&window)?;
    state.recent()
}
#[tauri::command]
pub async fn workspace_pending(
    window: WebviewWindow,
    state: State<'_, Arc<Services>>,
) -> Result<Option<String>> {
    caller(&window)?;
    state.pending()
}
#[tauri::command]
pub async fn workspace_pick(window: WebviewWindow, kind: String) -> Result<Option<String>> {
    caller(&window)?;
    match kind.as_str() {
        "parent" => Ok(rfd::AsyncFileDialog::new()
            .set_title("Choose a location for your workspace")
            .pick_folder()
            .await
            .map(|p| p.path().to_string_lossy().into())),
        "workspace" => Ok(rfd::AsyncFileDialog::new()
            .set_title("Open a Ghosh workspace")
            .add_filter("Ghosh workspace", &["ghosh"])
            .pick_file()
            .await
            .map(|p| p.path().to_string_lossy().into())),
        _ => Err(AppError::new(ErrorCode::Denied)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_local_main_shell_can_call_commands() {
        assert!(validate_caller("main", "tauri", Some("localhost"), None).is_ok());
        for (label, scheme, host, port) in [
            ("browser-child", "tauri", Some("localhost"), None),
            ("main", "https", Some("example.com"), None),
            ("main", "http", Some("127.0.0.1"), Some(9000)),
            ("main", "file", None, None),
        ] {
            assert!(validate_caller(label, scheme, host, port).is_err());
        }
    }
}
