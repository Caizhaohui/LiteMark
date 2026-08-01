//! Custom `lmlocal://` URI scheme for authorized local preview assets (M2).
//!
//! Preview HTML must never use raw `file://` URLs (§8.1). Instead, image and
//! other resource references are rewritten to `lmlocal://localhost/<path>`,
//! and this protocol handler serves the bytes only when the path lies under
//! an open document's parent directory.

use crate::commands::render::{
    decode_local_asset_path, read_authorized_asset, LOCAL_ASSET_SCHEME,
};
use crate::error::ErrorCode;
use crate::files::paths;
use crate::session::SessionManager;
use std::path::{Component, Path};
use tauri::{
    http::{header::CONTENT_TYPE, Request, Response, StatusCode},
    AppHandle, Manager, Runtime,
};

/// Register the asynchronous `lmlocal` protocol on the Tauri builder.
pub fn register_local_asset_protocol<R: Runtime>(
    builder: tauri::Builder<R>,
) -> tauri::Builder<R> {
    builder.register_asynchronous_uri_scheme_protocol(LOCAL_ASSET_SCHEME, |ctx, request, responder| {
        let app = ctx.app_handle().clone();
        tauri::async_runtime::spawn(async move {
            let response = handle_request(&app, &request).await;
            responder.respond(response);
        });
    })
}

async fn handle_request<R: Runtime>(
    app: &AppHandle<R>,
    request: &Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let uri = request.uri().to_string();
    let path = match decode_local_asset_path(&uri) {
        Ok(p) => p,
        Err(e) => {
            return error_response(StatusCode::BAD_REQUEST, &e.message);
        }
    };

    // Authorize against every open document's parent directory.
    let sessions = match app.try_state::<SessionManager>() {
        Some(s) => s,
        None => {
            return error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "session manager not ready",
            );
        }
    };
    let list = sessions.list().await;
    if !is_authorized_for_any(&path, &list) {
        log::warn!(
            "[lmlocal] denied path outside open document dirs: {}",
            path.display()
        );
        return error_response(StatusCode::FORBIDDEN, "path not authorized");
    }

    match read_authorized_asset(&path) {
        Ok((bytes, mime)) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, mime)
            .header("Cache-Control", "no-cache")
            .header("Access-Control-Allow-Origin", "*")
            .body(bytes)
            .unwrap_or_else(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "build failed")),
        Err(e) => {
            let status = match e.code {
                ErrorCode::FileNotFound => StatusCode::NOT_FOUND,
                ErrorCode::FilePermissionDenied => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            error_response(status, &e.message)
        }
    }
}

fn error_response(status: StatusCode, message: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(message.as_bytes().to_vec())
        .unwrap_or_else(|_| {
            Response::new(Vec::new())
        })
}

fn is_authorized_for_any(path: &Path, sessions: &[crate::session::DocumentSession]) -> bool {
    let Ok(normalized) = paths::normalize_long_path(path) else {
        return false;
    };
    for s in sessions {
        let Some(doc) = &s.file_path else {
            continue;
        };
        let Some(parent) = doc.parent() else {
            continue;
        };
        let Ok(root) = paths::normalize_long_path(parent) else {
            continue;
        };
        if path_inside(&normalized, &root) {
            return true;
        }
    }
    false
}

fn path_inside(path: &Path, root: &Path) -> bool {
    let path_c = comps(path);
    let root_c = comps(root);
    if path_c.len() < root_c.len() {
        return false;
    }
    path_c
        .iter()
        .zip(root_c.iter())
        .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

fn comps(path: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for c in path.components() {
        match c {
            Component::Prefix(p) => out.push(p.as_os_str().to_string_lossy().into_owned()),
            Component::RootDir => out.push("/".into()),
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(s) => out.push(s.to_string_lossy().into_owned()),
        }
    }
    out
}
