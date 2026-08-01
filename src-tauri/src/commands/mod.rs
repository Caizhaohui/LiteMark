//! Tauri commands exposed to the webview.
//!
//! The webview never talks to the sidecar directly; it goes through these Rust
//! commands, which enforce timeouts, correlation, and structured errors.
//! M1: document lifecycle, native file dialogs, recent files, recovery.
//! M2: render_markdown, release_render_session, open_external_url, assets.
//! M3: export_html, export_pdf, cancel_export, probe_export_tools, licenses.

pub mod documents;
pub mod export;
pub mod file_dialogs;
pub mod pandoc;
pub mod ping;
pub mod recent;
pub mod recovery;
pub mod render;
pub mod settings;
