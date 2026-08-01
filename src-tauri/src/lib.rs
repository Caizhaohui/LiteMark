//! LiteMark application library (used by `main.rs` and integration tests).

pub mod assets;
pub mod cli_files;
pub mod commands;
pub mod error;
pub mod files;
pub mod recovery;
pub mod session;
pub mod sidecar;

use cli_files::{collect_cli_files, filter_file_args, PendingCliFiles};
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
    // Cold-start file association: paths from double-click / shell open.
    let startup_files = collect_cli_files(std::env::args());
    if !startup_files.is_empty() {
        log::info!(
            "startup CLI files ({}): {:?}",
            startup_files.len(),
            startup_files
        );
    }
    let pending_cli = PendingCliFiles::new(startup_files);

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
        .manage(pending_cli)
        .invoke_handler(tauri::generate_handler![
            // M0
            ping::ping_sidecar,
            ping::warm_sidecar,
            // File association / CLI open
            take_pending_cli_files,
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
            // P1-1: warm the render sidecar in the background so the first preview
            // after double-click open does not wait on Node/crossnote cold start.
            let handle = app.handle().clone();
            let mgr = app.state::<SidecarManager>().inner().clone();
            let has_cli_files = {
                let pending = app.state::<PendingCliFiles>();
                !pending.peek().is_empty()
            };
            tauri::async_runtime::spawn(async move {
                mgr.set_app_handle(handle).await;
                if has_cli_files {
                    log::info!("[sidecar] CLI open paths present — warming immediately");
                }
                if let Err(e) = mgr.ensure_warm().await {
                    log::warn!("[sidecar] startup warm failed (preview will retry): {e}");
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
    {
        log::error!("LiteMark exited with error: {e}");
        std::process::exit(1);
    }
}

/// Consume cold-start CLI / file-association paths (once). Frontend opens them
/// on mount. Second-instance opens still use the `open-files` event.
///
/// P1-3: when paths are present, kick sidecar warm in parallel so open + warm
/// race instead of serializing behind the first preview render.
#[tauri::command]
async fn take_pending_cli_files(
    pending: tauri::State<'_, PendingCliFiles>,
    sidecar: tauri::State<'_, SidecarManager>,
) -> Result<Vec<String>, String> {
    let files = pending.take();
    if !files.is_empty() {
        let mgr = sidecar.inner().clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = mgr.ensure_warm().await {
                log::warn!("[sidecar] warm on CLI open failed: {e}");
            }
        });
    }
    Ok(files)
}

/// Shared single-instance callback body (kept top-level so the plugin init and
/// the `#[cfg]` helper agree on the signature).
#[cfg(desktop)]
fn single_instance_callback(app: &tauri::AppHandle, argv: Vec<String>, _cwd: String) {
    let app_handle = app.clone();
    // argv[0] is the executable; remaining args may include file paths.
    let files = filter_file_args(argv.into_iter().skip(1));
    if !files.is_empty() {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
        let _ = app_handle.emit("open-files", files);
    }
}
