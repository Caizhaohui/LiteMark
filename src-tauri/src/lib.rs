//! LiteMark application library (used by `main.rs` and integration tests).

pub mod assets;
pub mod commands;
pub mod error;
pub mod files;
pub mod recovery;
pub mod session;
pub mod sidecar;

use commands::{
    documents, export, file_dialogs, pandoc, ping, recent, recovery as recovery_cmd, render,
    settings as settings_cmd,
};
use session::SessionManager;
use sidecar::SidecarManager;
use tauri::{Emitter, Manager};

/// A simple logger that writes to stderr with a timestamp + level tag.
/// Deliberately minimal for M0/M1; replaced by a real logging facade in a later
/// milestone.
fn init_logging() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .try_init();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    log::info!("LiteMark core starting");

    let sidecar = SidecarManager::new();
    let sessions = SessionManager::new();

    let mut builder = tauri::Builder::default();

    // Single-instance handling (desktop only): forward second-instance file
    // args to the running instance (M1 minimal implementation).
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(single_instance_callback));
    }

    // M2: authorized local-asset protocol for preview images (no file://).
    builder = assets::register_local_asset_protocol(builder);

    if let Err(e) = builder
        .manage(sidecar)
        .manage(sessions)
        .invoke_handler(tauri::generate_handler![
            // M0
            ping::ping_sidecar,
            // M1 — documents
            documents::new_document,
            documents::open_file,
            documents::save_document,
            documents::save_as_document,
            documents::set_document_content,
            documents::get_document,
            documents::list_documents,
            documents::list_dirty_documents,
            documents::close_document,
            documents::set_active_document,
            documents::active_document,
            documents::check_external_change,
            documents::discard_recovery_snapshot,
            // M1 — native file dialogs
            file_dialogs::show_open_dialog,
            file_dialogs::show_save_dialog,
            file_dialogs::is_markdown_path,
            // M1 — recent files
            recent::get_recent_files,
            recent::set_recent_pinned,
            recent::clear_recent_files,
            // M1 — recovery
            recovery_cmd::get_pending_recovery,
            recovery_cmd::restore_recovery_snapshot,
            recovery_cmd::discard_recovery_snapshot_cmd,
            recovery_cmd::discard_all_recovery,
            // M2 — render / links / assets
            render::render_markdown,
            render::release_render_session,
            render::open_external_url,
            render::resolve_document_asset,
            // M3 — export
            export::export_html,
            export::export_pdf,
            export::cancel_export,
            export::probe_export_tools,
            export::get_last_export_dir,
            export::set_last_export_dir,
            export::get_third_party_notices,
            // M5 — pandoc / optional tools / settings
            pandoc::probe_pandoc,
            pandoc::probe_optional_tools,
            pandoc::export_with_pandoc,
            settings_cmd::get_settings,
            settings_cmd::set_settings,
            settings_cmd::trust_workspace,
            settings_cmd::untrust_workspace,
            settings_cmd::is_path_trusted,
            settings_cmd::get_custom_css,
            // M6 — diagnostics / updater stub
            settings_cmd::export_crash_log,
            settings_cmd::get_update_status,
        ])
        .setup(|app| {
            log::info!("LiteMark window initialized: {}", app.package_info().name);
            // Wire AppHandle into SidecarManager so export progress events forward.
            let handle = app.handle().clone();
            let mgr = app.state::<SidecarManager>().inner().clone();
            tauri::async_runtime::spawn(async move {
                mgr.set_app_handle(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
    {
        log::error!("LiteMark exited with error: {e}");
        std::process::exit(1);
    }
}

/// Shared single-instance callback body (kept top-level so the plugin init and
/// the `#[cfg]` helper agree on the signature).
#[cfg(desktop)]
fn single_instance_callback(app: &tauri::AppHandle, argv: Vec<String>, _cwd: String) {
    let app_handle = app.clone();
    let files: Vec<String> = argv
        .into_iter()
        .skip(1)
        .filter(|a| !a.starts_with('-') && std::path::Path::new(a).is_file())
        .collect();
    if !files.is_empty() {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
        let _ = app_handle.emit("open-files", files);
    }
}
