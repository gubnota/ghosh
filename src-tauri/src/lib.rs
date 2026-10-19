mod commands;
mod platform;
mod services;
use std::sync::Arc;
use tauri::Manager;

pub fn run() {
    let pending = Arc::new(services::DocumentOpenQueue::default());
    let setup_pending = Arc::clone(&pending);
    let app = tauri::Builder::default()
        .setup(move |app| {
            let services = Arc::new(services::Services::new(
                app.path().app_data_dir()?,
                Arc::clone(&setup_pending),
            )?);
            app.manage(services);
            for arg in std::env::args().skip(1).filter(|s| s.ends_with(".ghosh")) {
                let _ = setup_pending.request(arg, app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::run_start,
            commands::runs_list,
            commands::run_stop,
            commands::automations_list,
            commands::automation_read,
            commands::automation_preview,
            commands::automation_commit,
            commands::node_connection_preview,
            commands::node_connection_test,
            commands::terminal_open_local,
            commands::terminal_open_node,
            commands::terminal_poll,
            commands::terminal_input,
            commands::terminal_resize,
            commands::terminal_close,
            commands::workspace_startup,
            commands::workspace_close,
            commands::workspace_forget_recent,
            commands::workspace_open,
            commands::workspace_create,
            commands::workspace_snapshot,
            commands::workspace_rename,
            commands::workspace_reload,
            commands::workspace_trust,
            commands::workspace_pick,
            commands::workspace_recent,
            commands::workspace_pending,
            commands::operations_snapshot,
            commands::jobs_cancel,
            commands::nodes_list,
            commands::nodes_preview,
            commands::nodes_commit,
            commands::workspace_key,
            commands::workspace_key_preview,
            commands::workspace_key_commit,
            commands::node_identity,
            commands::keys_generate,
            commands::keys_relink,
            commands::keys_list,
            commands::keys_preview,
            commands::keys_commit,
            commands::keys_default,
            commands::keys_forget,
            commands::keys_pick,
            commands::approvals_decide
        ])
        .build(tauri::generate_context!())
        .expect("Ghosh desktop failed");
    app.run(move |app, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            app.state::<Arc<services::Services>>().terminals.close_all();
        }
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Opened { urls } = event {
            for url in urls {
                if let Ok(path) = url.to_file_path() {
                    let _ = pending.request(path.to_string_lossy().into(), app);
                }
            }
        }
    });
}
