//! M3 export commands: HTML / PDF via the Node sidecar, with path authorization,
//! long timeouts, cancel, browser probe, and last-export-dir prefs.
//!
//! The webview never chooses a write path freely — it only receives a path
//! from the native save dialog, which we re-validate here before forwarding
//! to the sidecar.

use crate::error::{command_err, CommandResult, ErrorCode, SidecarError};
use crate::files::export_prefs::ExportPrefs;
use crate::files::paths;
use crate::session::SessionManager;
use crate::sidecar::SidecarManager;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// PDF export can take well over the default 10s render timeout (§9.2).
const EXPORT_HTML_TIMEOUT: Duration = Duration::from_secs(60);
const EXPORT_PDF_TIMEOUT: Duration = Duration::from_secs(180);

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportHtmlParams {
    pub session_id: String,
    pub output_path: String,
    #[serde(default)]
    pub markdown: Option<String>,
    #[serde(default)]
    pub offline: Option<bool>,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
    #[serde(default)]
    pub job_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPdfParams {
    pub session_id: String,
    pub output_path: String,
    #[serde(default)]
    pub markdown: Option<String>,
    #[serde(default)]
    pub page: Option<serde_json::Value>,
    #[serde(default)]
    pub browser_path: Option<String>,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
    #[serde(default)]
    pub job_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportCommandResult {
    pub output_path: String,
    pub bytes: u64,
    pub job_id: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalToolStatusWire {
    pub name: String,
    pub available: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeExportToolsResult {
    pub browser: ExternalToolStatusWire,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn export_html(
    app: AppHandle,
    sessions: State<'_, SessionManager>,
    sidecar: State<'_, SidecarManager>,
    params: ExportHtmlParams,
) -> CommandResult<ExportCommandResult> {
    let job_id = params
        .job_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let session = sessions.get(&params.session_id).await.map_err(command_err)?;
    let markdown = params.markdown.unwrap_or_else(|| session.content.clone());
    let logical = session
        .file_path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned());

    let output = authorize_export_path(&params.output_path, &["html", "htm"], session.file_path.as_deref())
        .map_err(command_err)?;

    emit_progress(
        &app,
        &job_id,
        "preparing",
        0.02,
        Some("Starting HTML export"),
    );

    let mut sidecar_params = json!({
        "sessionId": params.session_id,
        "markdown": markdown,
        "logicalFilePath": logical,
        "outputPath": output.to_string_lossy(),
        "offline": params.offline.unwrap_or(true),
        "jobId": job_id,
    });
    if let Some(opts) = params.options {
        sidecar_params["options"] = opts;
    }

    let result = sidecar
        .request_with_timeout("exportHtml", sidecar_params, EXPORT_HTML_TIMEOUT)
        .await
        .map_err(command_err)?;

    let bytes = result.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
    let out_path = result
        .get("outputPath")
        .and_then(|v| v.as_str())
        .unwrap_or(&params.output_path)
        .to_string();

    if let Some(parent) = Path::new(&out_path).parent() {
        let _ = ExportPrefs::set_last_export_dir(parent);
    }

    emit_progress(&app, &job_id, "finalizing", 1.0, Some("Done"));

    Ok(ExportCommandResult {
        output_path: out_path,
        bytes,
        job_id,
        format: "html".into(),
    })
}

#[tauri::command]
pub async fn export_pdf(
    app: AppHandle,
    sessions: State<'_, SessionManager>,
    sidecar: State<'_, SidecarManager>,
    params: ExportPdfParams,
) -> CommandResult<ExportCommandResult> {
    let job_id = params
        .job_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let session = sessions.get(&params.session_id).await.map_err(command_err)?;
    let markdown = params.markdown.unwrap_or_else(|| session.content.clone());
    let logical = session
        .file_path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned());

    let output = authorize_export_path(&params.output_path, &["pdf"], session.file_path.as_deref())
        .map_err(command_err)?;

    emit_progress(
        &app,
        &job_id,
        "preparing",
        0.02,
        Some("Starting PDF export"),
    );

    let mut sidecar_params = json!({
        "sessionId": params.session_id,
        "markdown": markdown,
        "logicalFilePath": logical,
        "outputPath": output.to_string_lossy(),
        "jobId": job_id,
        "browserPath": params.browser_path,
    });
    if let Some(page) = params.page {
        sidecar_params["page"] = page;
    }
    if let Some(opts) = params.options {
        sidecar_params["options"] = opts;
    }

    let result = sidecar
        .request_with_timeout("exportPdf", sidecar_params, EXPORT_PDF_TIMEOUT)
        .await
        .map_err(command_err)?;

    let bytes = result.get("bytes").and_then(|v| v.as_u64()).unwrap_or(0);
    let out_path = result
        .get("outputPath")
        .and_then(|v| v.as_str())
        .unwrap_or(&params.output_path)
        .to_string();

    if let Some(parent) = Path::new(&out_path).parent() {
        let _ = ExportPrefs::set_last_export_dir(parent);
    }

    emit_progress(&app, &job_id, "finalizing", 1.0, Some("Done"));

    Ok(ExportCommandResult {
        output_path: out_path,
        bytes,
        job_id,
        format: "pdf".into(),
    })
}

#[tauri::command]
pub async fn cancel_export(
    sidecar: State<'_, SidecarManager>,
    job_id: String,
) -> CommandResult<bool> {
    let params = json!({ "jobId": job_id });
    match sidecar
        .request_with_timeout("cancelJob", params, Duration::from_secs(5))
        .await
    {
        Ok(v) => Ok(v
            .get("cancelled")
            .and_then(|x| x.as_bool())
            .unwrap_or(true)),
        Err(e) if matches!(e.code, ErrorCode::SidecarCrashed | ErrorCode::SidecarStartFailed) => {
            Ok(false)
        }
        Err(e) => Err(command_err(e)),
    }
}

#[tauri::command]
pub async fn probe_export_tools(
    sidecar: State<'_, SidecarManager>,
) -> CommandResult<ProbeExportToolsResult> {
    let result = sidecar
        .request_with_timeout("probeExternalTools", json!({}), Duration::from_secs(15))
        .await
        .map_err(command_err)?;

    let browser = result
        .get("tools")
        .and_then(|t| t.as_array())
        .and_then(|arr| arr.first())
        .map(|b| ExternalToolStatusWire {
            name: b
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Browser")
                .to_string(),
            available: b
                .get("available")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            path: b
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            version: b
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
        .unwrap_or(ExternalToolStatusWire {
            name: "Browser".into(),
            available: false,
            path: None,
            version: None,
        });

    Ok(ProbeExportToolsResult { browser })
}

#[tauri::command]
pub async fn get_last_export_dir() -> CommandResult<Option<String>> {
    Ok(ExportPrefs::last_export_dir().map(|p| p.to_string_lossy().into_owned()))
}

#[tauri::command]
pub async fn set_last_export_dir(path: String) -> CommandResult<()> {
    let p = PathBuf::from(path);
    ExportPrefs::set_last_export_dir(&p).map_err(command_err)
}

/// Return the bundled third-party notices text for the About / Licenses dialog.
#[tauri::command]
pub async fn get_third_party_notices() -> CommandResult<String> {
    // Prefer a resource next to the executable / resource dir; fall back to
    // repo-relative paths for `tauri dev`.
    let candidates = [
        "THIRD_PARTY_NOTICES.md",
        "../THIRD_PARTY_NOTICES.md",
        "../../THIRD_PARTY_NOTICES.md",
    ];
    if let Ok(cwd) = std::env::current_dir() {
        for rel in candidates {
            let p = cwd.join(rel);
            if p.is_file() {
                if let Ok(text) = std::fs::read_to_string(&p) {
                    return Ok(text);
                }
            }
        }
    }
    // Also try next to the executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in ["THIRD_PARTY_NOTICES.md", "licenses/THIRD_PARTY_NOTICES.md"] {
                let p = dir.join(name);
                if p.is_file() {
                    if let Ok(text) = std::fs::read_to_string(&p) {
                        return Ok(text);
                    }
                }
            }
        }
    }
    Ok(include_str!("../../../THIRD_PARTY_NOTICES.md").to_string())
}

// ---------------------------------------------------------------------------
// Path authorization
// ---------------------------------------------------------------------------

/// Validate that `path` is a safe absolute export destination.
///
/// Rules:
/// - Must be absolute (after normalization).
/// - Extension must be in `allowed_exts`.
/// - Must not equal the source Markdown path (never overwrite the document).
/// - Parent directory must exist (or we create it if missing and writable).
fn authorize_export_path(
    path: &str,
    allowed_exts: &[&str],
    source_md: Option<&Path>,
) -> Result<PathBuf, SidecarError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "export path is empty",
        ));
    }
    let candidate = PathBuf::from(trimmed);
    if !candidate.is_absolute() {
        return Err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "export path must be absolute",
        ));
    }
    let normalized = paths::normalize_long_path(&candidate)?;

    let ext = normalized
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if !allowed_exts.iter().any(|a| a.eq_ignore_ascii_case(&ext)) {
        return Err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            format!(
                "export path must end with one of: {}",
                allowed_exts.join(", ")
            ),
        ));
    }

    if let Some(src) = source_md {
        if let (Ok(a), Ok(b)) = (
            paths::normalize_long_path(src),
            paths::normalize_long_path(&normalized),
        ) {
            if a == b {
                return Err(SidecarError::new(
                    ErrorCode::PathNotAuthorized,
                    "refusing to overwrite the source Markdown file with an export",
                ));
            }
        }
    }

    if let Some(parent) = normalized.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SidecarError::new(
                    ErrorCode::FilePermissionDenied,
                    format!("could not create export directory {}: {e}", parent.display()),
                )
            })?;
        }
        // Ensure parent is a directory (not a file).
        if parent.exists() && !parent.is_dir() {
            return Err(SidecarError::new(
                ErrorCode::PathNotAuthorized,
                "export parent path is not a directory",
            ));
        }
    }

    Ok(normalized)
}

fn emit_progress(app: &AppHandle, job_id: &str, stage: &str, progress: f64, message: Option<&str>) {
    let payload = json!({
        "jobId": job_id,
        "stage": stage,
        "progress": progress,
        "message": message,
    });
    let _ = app.emit("export-progress", payload);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn rejects_non_absolute() {
        let err = authorize_export_path("out.pdf", &["pdf"], None).unwrap_err();
        assert_eq!(err.code, ErrorCode::PathNotAuthorized);
    }

    #[test]
    fn rejects_wrong_extension() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("x.md");
        let err = authorize_export_path(&p.to_string_lossy(), &["pdf"], None).unwrap_err();
        assert_eq!(err.code, ErrorCode::PathNotAuthorized);
    }

    #[test]
    fn rejects_overwriting_source_md() {
        let dir = TempDir::new().unwrap();
        let md = dir.path().join("note.md");
        fs::write(&md, b"# hi").unwrap();
        let err = authorize_export_path(&md.to_string_lossy(), &["md", "pdf"], Some(&md))
            .unwrap_err();
        // extension md not allowed for pdf list — use pdf with same path trick:
        let pdf_named_md = md.clone();
        let err2 = authorize_export_path(
            &pdf_named_md.to_string_lossy(),
            &["md"],
            Some(&md),
        )
        .unwrap_err();
        assert_eq!(err2.code, ErrorCode::PathNotAuthorized);
        let _ = err;
    }

    #[test]
    fn accepts_valid_pdf_path() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("out.pdf");
        let ok = authorize_export_path(&p.to_string_lossy(), &["pdf"], None).unwrap();
        assert_eq!(ok.extension().unwrap(), "pdf");
    }
}
