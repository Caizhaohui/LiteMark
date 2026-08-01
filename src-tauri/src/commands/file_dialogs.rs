//! Native file dialogs (M1). LiteMark uses [`rfd`] (Rust native dialog) rather
//! than the Tauri `dialog` plugin, so **no dialog permission is exposed to the
//! webview** — consistent with the security model (all file access brokered by
//! Rust). The dialog returns only the chosen path; the webview then calls
//! `open_file` / `save_as_document` to actually read/write through Rust.

use crate::error::CommandResult;
use serde::{Deserialize, Serialize};

/// Filter shown in the dialog dropdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFilter {
    pub name: String,
    /// e.g. `["md", "markdown"]`.
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDialogOptions {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub filters: Vec<FileFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDialogOptions {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub filters: Vec<FileFilter>,
    /// Suggested filename.
    #[serde(default)]
    pub default_file_name: Option<String>,
    /// Optional starting directory (e.g. last export folder).
    #[serde(default)]
    pub default_directory: Option<String>,
}

/// Show an open-file dialog and return the chosen absolute path (or null).
#[tauri::command]
pub async fn show_open_dialog(options: OpenDialogOptions) -> CommandResult<Option<String>> {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(title) = &options.title {
        dialog = dialog.set_title(title);
    }
    for f in &options.filters {
        let exts: Vec<&str> = f.extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&f.name, &exts);
    }
    let result = dialog
        .pick_file()
        .await
        .map(|h| h.path().to_string_lossy().to_string());
    Ok(result)
}

/// Show a save-file dialog and return the chosen absolute path (or null).
#[tauri::command]
pub async fn show_save_dialog(options: SaveDialogOptions) -> CommandResult<Option<String>> {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(title) = &options.title {
        dialog = dialog.set_title(title);
    }
    if let Some(name) = &options.default_file_name {
        dialog = dialog.set_file_name(name);
    }
    if let Some(dir) = &options.default_directory {
        let p = std::path::PathBuf::from(dir);
        if p.is_dir() {
            dialog = dialog.set_directory(p);
        }
    }
    for f in &options.filters {
        let exts: Vec<&str> = f.extensions.iter().map(|s| s.as_str()).collect();
        dialog = dialog.add_filter(&f.name, &exts);
    }
    let result = dialog
        .save_file()
        .await
        .map(|h| h.path().to_string_lossy().to_string());
    Ok(result)
}

/// Convenience: validate that a path string looks like an allowed markdown
/// extension before the webview attempts to open it. This is defense-in-depth,
/// not the sole check — `open_file` itself handles arbitrary paths via Rust.
#[tauri::command]
pub async fn is_markdown_path(path: String) -> CommandResult<bool> {
    let p = std::path::Path::new(&path);
    let ok = matches!(
        p.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref(),
        Some("md") | Some("markdown") | Some("mdx") | Some("mkd")
    );
    Ok(ok)
}
