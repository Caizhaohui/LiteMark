//! M2 render / link / local-asset Tauri commands.
//!
//! The webview never talks to the Node sidecar directly. All render traffic
//! is brokered here so we can:
//! - attach the session's logical file path (for relative resources)
//! - enforce createSession / closeSession lifecycle
//! - open external URLs via the OS (no webview navigation)
//! - authorize local asset paths for the `lmlocal` custom protocol

use crate::error::{command_err, CommandResult, ErrorCode, SidecarError};
use crate::files::paths;
use crate::session::SessionManager;
use crate::sidecar::SidecarManager;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Component, Path, PathBuf};
use tauri::State;

/// Custom URI scheme used by the preview pane for authorized local images.
pub const LOCAL_ASSET_SCHEME: &str = "lmlocal";

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderMarkdownParams {
    pub session_id: String,
    pub markdown: String,
    pub revision: u64,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TocEntryWire {
    pub level: u32,
    pub text: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticWire {
    pub level: String,
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderMarkdownResult {
    pub html: String,
    pub toc: Vec<TocEntryWire>,
    pub diagnostics: Vec<DiagnosticWire>,
    pub render_ms: u64,
    pub revision: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveAssetResult {
    pub absolute_path: String,
    pub url: String,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Render in-memory Markdown via the Node/crossnote sidecar.
///
/// The source file is never written. `revision` is echoed so the UI can drop
/// stale responses when the user types faster than renders complete.
#[tauri::command]
pub async fn render_markdown(
    sessions: State<'_, SessionManager>,
    sidecar: State<'_, SidecarManager>,
    params: RenderMarkdownParams,
) -> CommandResult<RenderMarkdownResult> {
    let session = sessions.get(&params.session_id).await.map_err(command_err)?;
    let logical = session
        .file_path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned());

    // Ensure a sidecar render session exists (idempotent).
    let create_params = json!({
        "sessionId": params.session_id,
        "logicalFilePath": logical,
    });
    if let Err(e) = sidecar.request("createSession", create_params).await {
        // Non-fatal if the method already has a session; only fail hard on
        // crash / start failure so the subsequent render can still retry.
        log::warn!("[render] createSession: {e}");
        if matches!(
            e.code,
            ErrorCode::SidecarStartFailed | ErrorCode::SidecarCrashed
        ) {
            return Err(command_err(e));
        }
    }

    let mut render_params = json!({
        "sessionId": params.session_id,
        "markdown": params.markdown,
        "logicalFilePath": logical,
        "revision": params.revision,
    });
    if let Some(opts) = params.options {
        render_params["options"] = opts;
    }

    let result = sidecar
        .request("render", render_params)
        .await
        .map_err(command_err)?;

    let html = result
        .get("html")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let render_ms = result
        .get("renderMs")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let toc = parse_toc(result.get("toc"));
    let diagnostics = parse_diagnostics(result.get("diagnostics"));

    Ok(RenderMarkdownResult {
        html,
        toc,
        diagnostics,
        render_ms,
        revision: params.revision,
    })
}

/// Release the sidecar render session when a document tab is closed.
#[tauri::command]
pub async fn release_render_session(
    sidecar: State<'_, SidecarManager>,
    session_id: String,
) -> CommandResult<()> {
    let params = json!({ "sessionId": session_id });
    match sidecar.request("closeSession", params).await {
        Ok(_) => Ok(()),
        Err(e) if matches!(e.code, ErrorCode::SidecarCrashed | ErrorCode::SidecarStartFailed) => {
            // Sidecar not running — nothing to release.
            Ok(())
        }
        Err(e) => {
            // Unknown session or protocol noise is non-fatal on close.
            log::debug!("[render] closeSession: {e}");
            Ok(())
        }
    }
}

/// Open an external URL with the OS default handler. Only http(s)/mailto/tel.
/// Preview links never navigate the webview itself (§8.4).
#[tauri::command]
pub async fn open_external_url(url: String) -> CommandResult<()> {
    open_url_safe(&url).map_err(command_err)
}

/// Resolve a relative or absolute asset path against the document directory.
/// Returns an authorized absolute path + `lmlocal://` URL, or an error if the
/// path would escape the document directory (PATH_NOT_AUTHORIZED).
#[tauri::command]
pub async fn resolve_document_asset(
    sessions: State<'_, SessionManager>,
    session_id: String,
    href: String,
) -> CommandResult<ResolveAssetResult> {
    let session = sessions.get(&session_id).await.map_err(command_err)?;
    let doc_path = session.file_path.as_ref().ok_or_else(|| {
        command_err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "document has no file path; relative assets cannot be resolved for an unsaved document",
        ))
    })?;
    let absolute = authorize_asset_path(doc_path, &href).map_err(command_err)?;
    let abs_str = absolute.to_string_lossy().into_owned();
    let url = build_local_asset_url(&abs_str);
    Ok(ResolveAssetResult {
        absolute_path: abs_str,
        url,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_toc(value: Option<&serde_json::Value>) -> Vec<TocEntryWire> {
    let Some(arr) = value.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            Some(TocEntryWire {
                level: item.get("level").and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                text: item.get("text")?.as_str()?.to_string(),
                id: item.get("id")?.as_str()?.to_string(),
            })
        })
        .collect()
}

fn parse_diagnostics(value: Option<&serde_json::Value>) -> Vec<DiagnosticWire> {
    let Some(arr) = value.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            let level = item
                .get("level")
                .or_else(|| item.get("severity"))
                .and_then(|v| v.as_str())
                .unwrap_or("info")
                .to_string();
            Some(DiagnosticWire {
                level,
                code: item
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("RENDER")
                    .to_string(),
                message: item.get("message")?.as_str()?.to_string(),
                line: item
                    .get("line")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32),
            })
        })
        .collect()
}

/// Validate and open a URL. Rejects anything that is not a safe scheme.
pub fn open_url_safe(url: &str) -> Result<(), SidecarError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(SidecarError::new(
            ErrorCode::UntrustedOperationBlocked,
            "empty URL",
        ));
    }
    let lower = trimmed.to_ascii_lowercase();
    let allowed = lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:");
    if !allowed {
        return Err(SidecarError::new(
            ErrorCode::UntrustedOperationBlocked,
            format!("refusing to open URL with disallowed scheme: {trimmed}"),
        ));
    }
    // Block javascript: and data: even if someone tries mixed-case tricks.
    if lower.contains("javascript:") || lower.starts_with("data:") {
        return Err(SidecarError::new(
            ErrorCode::UntrustedOperationBlocked,
            "refusing to open scripted URL",
        ));
    }

    open_with_os(trimmed)
}

fn open_with_os(url: &str) -> Result<(), SidecarError> {
    #[cfg(windows)]
    {
        // `cmd /C start "" <url>` — empty title argument required by `start`.
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| {
                SidecarError::new(
                    ErrorCode::UntrustedOperationBlocked,
                    format!("failed to open URL: {e}"),
                )
            })?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        std::process::Command::new(opener)
            .arg(url)
            .spawn()
            .map_err(|e| {
                SidecarError::new(
                    ErrorCode::UntrustedOperationBlocked,
                    format!("failed to open URL: {e}"),
                )
            })?;
        Ok(())
    }
}

/// Build `lmlocal://localhost/<url-encoded-absolute-path>`.
pub fn build_local_asset_url(absolute_path: &str) -> String {
    format!(
        "{LOCAL_ASSET_SCHEME}://localhost/{}",
        urlencoding_encode(absolute_path)
    )
}

/// Minimal path-safe percent-encoding (encode everything except unreserved).
fn urlencoding_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            // Keep path separators readable for debugging; the protocol
            // decoder accepts both encoded and raw forms.
            b'/' | b'\\' | b':' => out.push(*b as char),
            _ => {
                out.push('%');
                out.push(nibble(b >> 4));
                out.push(nibble(b & 0x0f));
            }
        }
    }
    out
}

fn nibble(n: u8) -> char {
    char::from(if n < 10 { b'0' + n } else { b'A' + (n - 10) })
}

/// Decode a path from a `lmlocal://localhost/...` request URI.
pub fn decode_local_asset_path(uri: &str) -> Result<PathBuf, SidecarError> {
    // Accept both `lmlocal://localhost/C:/foo` and `lmlocal:///C:/foo`.
    let stripped = uri
        .strip_prefix(&format!("{LOCAL_ASSET_SCHEME}://localhost/"))
        .or_else(|| uri.strip_prefix(&format!("{LOCAL_ASSET_SCHEME}:///")))
        .or_else(|| uri.strip_prefix(&format!("{LOCAL_ASSET_SCHEME}:")))
        .unwrap_or(uri);
    let decoded = percent_decode(stripped);
    if decoded.is_empty() {
        return Err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "empty asset path",
        ));
    }
    Ok(PathBuf::from(decoded))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Resolve `href` against the document file and ensure the result stays under
/// the document's parent directory (no `..` escape).
pub fn authorize_asset_path(doc_path: &Path, href: &str) -> Result<PathBuf, SidecarError> {
    let href = href.trim();
    if href.is_empty() {
        return Err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "empty asset href",
        ));
    }
    // Reject obvious remote / dangerous schemes early.
    let lower = href.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("javascript:")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
        || lower.starts_with('#')
    {
        return Err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "not a local asset path",
        ));
    }

    let candidate = if lower.starts_with("file://") {
        // file:///C:/path or file://localhost/C:/path
        let rest = href
            .trim_start_matches("file://")
            .trim_start_matches("localhost")
            .trim_start_matches('/');
        // On Windows, file:///C:/foo → C:/foo after the above; file://C:/ may
        // leave a leading slash before the drive letter.
        let rest = rest.strip_prefix('/').unwrap_or(rest);
        PathBuf::from(rest)
    } else if Path::new(href).is_absolute() {
        PathBuf::from(href)
    } else {
        let parent = doc_path.parent().ok_or_else(|| {
            SidecarError::new(
                ErrorCode::PathNotAuthorized,
                "document path has no parent directory",
            )
        })?;
        parent.join(href)
    };

    let normalized = paths::normalize_long_path(&candidate)?;
    let doc_parent = doc_path.parent().ok_or_else(|| {
        SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "document path has no parent directory",
        )
    })?;
    let doc_root = paths::normalize_long_path(doc_parent)?;

    if !is_path_inside(&normalized, &doc_root) {
        return Err(SidecarError::new(
            ErrorCode::PathNotAuthorized,
            format!(
                "asset path escapes the document directory: {}",
                normalized.display()
            ),
        ));
    }

    if !normalized.is_file() {
        return Err(SidecarError::new(
            ErrorCode::FileNotFound,
            format!("asset not found: {}", normalized.display()),
        ));
    }

    Ok(normalized)
}

/// True if `path` is equal to or a descendant of `root` (lexical, after
/// component normalization — rejects `..` escape).
fn is_path_inside(path: &Path, root: &Path) -> bool {
    let path_c = normalize_components(path);
    let root_c = normalize_components(root);
    if path_c.len() < root_c.len() {
        return false;
    }
    path_c
        .iter()
        .zip(root_c.iter())
        .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

fn normalize_components(path: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for c in path.components() {
        match c {
            Component::Prefix(p) => out.push(p.as_os_str().to_string_lossy().into_owned()),
            Component::RootDir => out.push(String::from("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(s) => out.push(s.to_string_lossy().into_owned()),
        }
    }
    out
}

/// Read an authorized local asset for the custom protocol handler.
pub fn read_authorized_asset(absolute: &Path) -> Result<(Vec<u8>, &'static str), SidecarError> {
    let bytes = std::fs::read(absolute).map_err(|e| {
        let code = if e.kind() == std::io::ErrorKind::NotFound {
            ErrorCode::FileNotFound
        } else {
            ErrorCode::FilePermissionDenied
        };
        SidecarError::new(code, format!("read {}: {e}", absolute.display()))
    })?;
    let mime = mime_for_path(absolute);
    Ok((bytes, mime))
}

fn mime_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        Some("md") | Some("markdown") => "text/markdown; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn open_url_rejects_javascript() {
        let err = open_url_safe("javascript:alert(1)").unwrap_err();
        assert_eq!(err.code, ErrorCode::UntrustedOperationBlocked);
    }

    #[test]
    fn open_url_rejects_file_scheme() {
        let err = open_url_safe("file:///C:/Windows/System32").unwrap_err();
        assert_eq!(err.code, ErrorCode::UntrustedOperationBlocked);
    }

    #[test]
    fn asset_stays_under_doc_dir() {
        let dir = TempDir::new().unwrap();
        let img = dir.path().join("pic.png");
        fs::write(&img, b"png").unwrap();
        let doc = dir.path().join("note.md");
        fs::write(&doc, b"# hi").unwrap();

        let ok = authorize_asset_path(&doc, "pic.png").unwrap();
        assert_eq!(ok, paths::normalize_long_path(&img).unwrap());

        let escape = authorize_asset_path(&doc, "../secret.png");
        assert!(escape.is_err());
    }

    #[test]
    fn local_url_roundtrip_encoding() {
        let path = r"D:\docs\我的\note-image.png";
        let url = build_local_asset_url(path);
        assert!(url.starts_with("lmlocal://localhost/"));
        let back = decode_local_asset_path(&url).unwrap();
        assert_eq!(back, PathBuf::from(path));
    }
}
