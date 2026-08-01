//! Pandoc probe + advanced export (M5).
//!
//! Pandoc is optional. Missing tool → structured PANDOC_NOT_FOUND.
//! Arguments are always passed as an argv array — never shell-concatenated.

use crate::error::{command_err, CommandResult, ErrorCode, SidecarError};
use crate::files::paths;
use crate::files::settings::AppSettings;
use crate::session::SessionManager;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tauri::State;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PandocStatus {
    pub available: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PandocExportParams {
    pub session_id: String,
    pub output_path: String,
    /// One of: docx | epub | latex | latex
    pub format: String,
    #[serde(default)]
    pub markdown: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PandocExportResult {
    pub output_path: String,
    pub bytes: u64,
    pub format: String,
}

fn resolve_pandoc() -> Option<PathBuf> {
    let settings = AppSettings::load();
    if let Some(p) = settings.pandoc_path {
        let path = PathBuf::from(&p);
        if path.is_file() {
            return Some(path);
        }
    }
    // PATH lookup
    which_pandoc()
}

fn which_pandoc() -> Option<PathBuf> {
    let name = if cfg!(windows) { "pandoc.exe" } else { "pandoc" };
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

async fn pandoc_version(bin: &Path) -> Option<String> {
    let mut cmd = Command::new(bin);
    cmd.arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let output = cmd.output().await.ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines().next().map(|s| s.trim().to_string())
}

#[tauri::command]
pub async fn probe_pandoc() -> CommandResult<PandocStatus> {
    match resolve_pandoc() {
        Some(path) => {
            let version = pandoc_version(&path).await;
            Ok(PandocStatus {
                available: true,
                path: Some(path.to_string_lossy().into_owned()),
                version,
            })
        }
        None => Ok(PandocStatus {
            available: false,
            path: None,
            version: None,
        }),
    }
}

#[tauri::command]
pub async fn probe_optional_tools() -> CommandResult<OptionalToolsStatus> {
    // Non-blocking probes — never fail startup.
    let pandoc = probe_pandoc().await.unwrap_or(PandocStatus {
        available: false,
        path: None,
        version: None,
    });
    let graphviz = which_on_path(&["dot", "dot.exe"]);
    let plantuml = which_on_path(&["plantuml", "plantuml.jar", "plantuml.exe"]);
    Ok(OptionalToolsStatus {
        pandoc,
        graphviz: ToolPresence {
            name: "Graphviz".into(),
            available: graphviz.is_some(),
            path: graphviz.map(|p| p.to_string_lossy().into_owned()),
        },
        plantuml: ToolPresence {
            name: "PlantUML".into(),
            available: plantuml.is_some(),
            path: plantuml.map(|p| p.to_string_lossy().into_owned()),
        },
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPresence {
    pub name: String,
    pub available: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalToolsStatus {
    pub pandoc: PandocStatus,
    pub graphviz: ToolPresence,
    pub plantuml: ToolPresence,
}

fn which_on_path(names: &[&str]) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    for dir in std::env::split_paths(&path_var) {
        for name in names {
            let c = dir.join(name);
            if c.is_file() {
                return Some(c);
            }
        }
    }
    None
}

#[tauri::command]
pub async fn export_with_pandoc(
    sessions: State<'_, SessionManager>,
    params: PandocExportParams,
) -> CommandResult<PandocExportResult> {
    let bin = resolve_pandoc().ok_or_else(|| {
        command_err(SidecarError::new(
            ErrorCode::PandocNotFound,
            "Pandoc is not installed or not on PATH. Install Pandoc or set its path in Settings.",
        ))
    })?;

    let format = params.format.to_ascii_lowercase();
    let (ext, to_fmt) = match format.as_str() {
        "docx" => ("docx", "docx"),
        "epub" => ("epub", "epub"),
        "latex" | "tex" => ("tex", "latex"),
        other => {
            return Err(command_err(SidecarError::new(
                ErrorCode::UntrustedOperationBlocked,
                format!("unsupported pandoc format: {other}"),
            )));
        }
    };

    let session = sessions.get(&params.session_id).await.map_err(command_err)?;
    let markdown = params.markdown.unwrap_or_else(|| session.content.clone());

    let output = authorize_pandoc_output(&params.output_path, ext, session.file_path.as_deref())
        .map_err(command_err)?;

    // Write markdown to a temp input — never overwrite source.
    let tmp = std::env::temp_dir().join(format!(
        "litemark-pandoc-{}.md",
        uuid::Uuid::new_v4()
    ));
    tokio::fs::write(&tmp, markdown.as_bytes())
        .await
        .map_err(|e| {
            command_err(SidecarError::new(
                ErrorCode::ExportFailed,
                format!("write temp markdown: {e}"),
            ))
        })?;

    // Argv array only — no shell.
    let out_str = output.to_string_lossy().into_owned();
    let in_str = tmp.to_string_lossy().into_owned();
    let args = vec![
        in_str.clone(),
        "-f".into(),
        "gfm".into(),
        "-t".into(),
        to_fmt.into(),
        "-o".into(),
        out_str.clone(),
    ];

    let mut pandoc_cmd = Command::new(&bin);
    pandoc_cmd
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        pandoc_cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let output_proc = pandoc_cmd.output().await.map_err(|e| {
        command_err(SidecarError::new(
            ErrorCode::ExportFailed,
            format!("spawn pandoc: {e}"),
        ))
    })?;

    let _ = tokio::fs::remove_file(&tmp).await;

    if !output_proc.status.success() {
        let stderr = String::from_utf8_lossy(&output_proc.stderr);
        return Err(command_err(SidecarError::new(
            ErrorCode::ExportFailed,
            format!("pandoc failed: {}", stderr.trim()),
        ).with_details(serde_json::json!({
            "args": args,
            "status": output_proc.status.code(),
        }))));
    }

    let meta = tokio::fs::metadata(&output).await.map_err(|e| {
        command_err(SidecarError::new(
            ErrorCode::ExportFailed,
            format!("export missing after pandoc: {e}"),
        ))
    })?;

    Ok(PandocExportResult {
        output_path: out_str,
        bytes: meta.len(),
        format,
    })
}

fn authorize_pandoc_output(
    path: &str,
    expected_ext: &str,
    source_md: Option<&Path>,
) -> Result<PathBuf, SidecarError> {
    let candidate = PathBuf::from(path.trim());
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
    if ext != expected_ext {
        return Err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            format!("export path must end with .{expected_ext}"),
        ));
    }
    if let Some(src) = source_md {
        if let Ok(a) = paths::normalize_long_path(src) {
            if a == normalized {
                return Err(SidecarError::new(
                    ErrorCode::PathNotAuthorized,
                    "refusing to overwrite the source Markdown file",
                ));
            }
        }
    }
    if let Some(parent) = normalized.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SidecarError::new(
                    ErrorCode::FilePermissionDenied,
                    format!("create export dir: {e}"),
                )
            })?;
        }
    }
    Ok(normalized)
}
