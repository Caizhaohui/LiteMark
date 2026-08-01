//! Settings + trusted workspace + crash log export (M5/M6).

use crate::error::{command_err, CommandResult, ErrorCode, SidecarError};
use crate::files::settings::AppSettings;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsWire {
    pub trusted_workspaces: Vec<String>,
    pub pandoc_path: Option<String>,
    pub custom_css_path: Option<String>,
    pub enable_wiki_links: bool,
    pub experimental_code_execution: bool,
    pub update_endpoint: Option<String>,
}

impl From<AppSettings> for SettingsWire {
    fn from(s: AppSettings) -> Self {
        Self {
            trusted_workspaces: s.trusted_workspaces,
            pandoc_path: s.pandoc_path,
            custom_css_path: s.custom_css_path,
            enable_wiki_links: s.enable_wiki_links,
            experimental_code_execution: s.experimental_code_execution,
            update_endpoint: s.update_endpoint,
        }
    }
}

#[tauri::command]
pub async fn get_settings() -> CommandResult<SettingsWire> {
    Ok(AppSettings::load().into())
}

#[tauri::command]
pub async fn set_settings(settings: SettingsWire) -> CommandResult<()> {
    let s = AppSettings {
        trusted_workspaces: settings.trusted_workspaces,
        pandoc_path: settings.pandoc_path,
        custom_css_path: settings.custom_css_path,
        enable_wiki_links: settings.enable_wiki_links,
        experimental_code_execution: settings.experimental_code_execution,
        update_endpoint: settings.update_endpoint,
    };
    s.save().map_err(command_err)
}

#[tauri::command]
pub async fn trust_workspace(path: String) -> CommandResult<SettingsWire> {
    let mut s = AppSettings::load();
    // Require the path to exist and be a directory — user confirmation is the UI's job.
    let p = PathBuf::from(&path);
    if !p.is_dir() {
        return Err(command_err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "trusted workspace must be an existing directory",
        )));
    }
    s.trust(path);
    s.save().map_err(command_err)?;
    Ok(s.into())
}

#[tauri::command]
pub async fn untrust_workspace(path: String) -> CommandResult<SettingsWire> {
    let mut s = AppSettings::load();
    s.untrust(&path);
    s.save().map_err(command_err)?;
    Ok(s.into())
}

#[tauri::command]
pub async fn is_path_trusted(path: String) -> CommandResult<bool> {
    let s = AppSettings::load();
    // A file path is trusted if its parent directory (or itself) is listed.
    let p = PathBuf::from(&path);
    let dir = if p.is_dir() {
        p
    } else {
        p.parent()
            .map(|x| x.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(&path))
    };
    Ok(s.is_trusted(&dir.to_string_lossy()))
}

/// Read custom CSS only if the path is set and the file is under a trusted
/// workspace or explicitly configured. Strips `<script` as defense in depth.
#[tauri::command]
pub async fn get_custom_css() -> CommandResult<Option<String>> {
    let s = AppSettings::load();
    let Some(path) = s.custom_css_path else {
        return Ok(None);
    };
    let p = PathBuf::from(&path);
    if !p.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&p).map_err(|e| {
        command_err(SidecarError::new(
            ErrorCode::FilePermissionDenied,
            format!("read custom CSS: {e}"),
        ))
    })?;
    // Block obvious script injection via CSS files.
    if text.to_ascii_lowercase().contains("<script")
        || text.to_ascii_lowercase().contains("expression(")
        || text.to_ascii_lowercase().contains("javascript:")
    {
        return Err(command_err(SidecarError::new(
            ErrorCode::UntrustedOperationBlocked,
            "custom CSS contains forbidden constructs",
        )));
    }
    Ok(Some(text))
}

/// Export recent stderr/log buffer to a user-chosen path is not available;
/// instead dump a crash report snapshot under app-data and return its path.
#[tauri::command]
pub async fn export_crash_log() -> CommandResult<String> {
    let dir = crate::files::paths::app_data_dir()
        .map_err(command_err)?
        .join("logs");
    std::fs::create_dir_all(&dir).map_err(|e| {
        command_err(SidecarError::new(
            ErrorCode::FilePermissionDenied,
            format!("create logs dir: {e}"),
        ))
    })?;
    let stamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ");
    let path = dir.join(format!("crash-report-{stamp}.txt"));
    let body = format!(
        "LiteMark crash/diagnostic report\n\
         generated: {}\n\
         version: {}\n\
         os: {}\n\
         note: Full crash dumps are written by the OS. This file records app state for support.\n\
         trusted_workspaces: {}\n",
        chrono::Utc::now().to_rfc3339(),
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        AppSettings::load().trusted_workspaces.join("; "),
    );
    std::fs::write(&path, body).map_err(|e| {
        command_err(SidecarError::new(
            ErrorCode::FilePermissionDenied,
            format!("write crash log: {e}"),
        ))
    })?;
    Ok(path.to_string_lossy().into_owned())
}

/// Placeholder state for M6 updater — never auto-downloads without endpoint.
#[tauri::command]
pub async fn get_update_status() -> CommandResult<UpdateStatus> {
    let s = AppSettings::load();
    Ok(UpdateStatus {
        enabled: s
            .update_endpoint
            .as_ref()
            .map(|e| !e.is_empty())
            .unwrap_or(false),
        endpoint: s.update_endpoint,
        message: "Automatic updates are disabled until an update endpoint and code signing are configured.".into(),
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub message: String,
}

