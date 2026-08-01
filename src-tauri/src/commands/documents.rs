//! Document-lifecycle Tauri commands (M1): new / open / save / save_as /
//! set_content / get / list / list_dirty / close / set_active / active_id /
//! check_external_change. The webview calls only these — it has no `fs`
//! permission, so all file IO is brokered here through [`SessionManager`].

use crate::error::{command_err, CommandResult};
use crate::files::recent::RecentList;
use crate::recovery::{entry_from_snapshot, write_snapshot};
use crate::session::{DocumentSession, SessionManager};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

/// Trimmed view of a session for `list` (avoids shipping huge contents when
/// the UI only needs tab metadata).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    #[serde(default, rename = "filePath")]
    pub file_path: Option<PathBuf>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "savedContentHash")]
    pub saved_content_hash: String,
    pub encoding: String,
    #[serde(rename = "lineEnding")]
    pub line_ending: String,
    pub dirty: bool,
    #[serde(rename = "readOnly")]
    pub read_only: bool,
    pub revision: u64,
    #[serde(rename = "externalMtimeMs")]
    pub external_mtime_ms: Option<i64>,
    pub active: bool,
}

impl SessionSummary {
    fn from_session(s: &DocumentSession, active: bool) -> Self {
        Self {
            id: s.id.clone(),
            file_path: s.file_path.clone(),
            display_name: s.display_name.clone(),
            saved_content_hash: s.saved_content_hash.clone(),
            encoding: serde_json::to_value(s.encoding)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "utf-8".into()),
            line_ending: serde_json::to_value(s.line_ending)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| "lf".into()),
            dirty: s.dirty,
            read_only: s.read_only,
            revision: s.revision,
            external_mtime_ms: s.external_mtime_ms,
            active,
        }
    }
}

/// Create a new empty document; returns its session id.
#[tauri::command]
pub async fn new_document(state: State<'_, SessionManager>) -> CommandResult<String> {
    Ok(state.new_document().await)
}

/// Open a file from an absolute path into a new session.
#[tauri::command]
pub async fn open_file(state: State<'_, SessionManager>, path: String) -> CommandResult<String> {
    let path = PathBuf::from(path);
    let id = state.open_file(&path).await.map_err(command_err)?;
    // Record in recent files (best-effort; never fail the open on this).
    if let Err(e) = RecentList::record_opened(&path, &Utc::now().to_rfc3339()) {
        log::warn!("[documents] could not update recent files: {e}");
    }
    Ok(id)
}

/// Save a session to its current path.
#[tauri::command]
pub async fn save_document(
    state: State<'_, SessionManager>,
    session_id: String,
) -> CommandResult<SaveResult> {
    let (mtime_ms, content_hash) = state.save(&session_id).await.map_err(command_err)?;
    // A clean save clears the recovery snapshot for this document.
    if let Some(snap) = state.snapshot_for_recovery(&session_id).await {
        if let Err(e) = crate::recovery::delete_snapshot(&snap.recovery_key) {
            log::warn!("[documents] could not clear recovery snapshot after save: {e}");
        }
    }
    Ok(SaveResult {
        mtime_ms,
        content_hash,
    })
}

/// Save a session to a new path.
#[tauri::command]
pub async fn save_as_document(
    state: State<'_, SessionManager>,
    session_id: String,
    path: String,
) -> CommandResult<SaveResult> {
    let path = PathBuf::from(path);
    let (mtime_ms, content_hash) = state
        .save_as(&session_id, &path)
        .await
        .map_err(command_err)?;
    if let Err(e) = RecentList::record_opened(&path, &Utc::now().to_rfc3339()) {
        log::warn!("[documents] could not update recent files after Save As: {e}");
    }
    if let Some(snap) = state.snapshot_for_recovery(&session_id).await {
        if let Err(e) = crate::recovery::delete_snapshot(&snap.recovery_key) {
            log::warn!("[documents] could not clear recovery snapshot after Save As: {e}");
        }
    }
    Ok(SaveResult {
        mtime_ms,
        content_hash,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveResult {
    #[serde(rename = "mtimeMs")]
    pub mtime_ms: i64,
    #[serde(rename = "contentHash")]
    pub content_hash: String,
}

/// Apply an edit from the editor (full content replace for M1).
#[tauri::command]
pub async fn set_document_content(
    state: State<'_, SessionManager>,
    session_id: String,
    content: String,
) -> CommandResult<()> {
    state
        .set_content(&session_id, content)
        .await
        .map_err(command_err)?;
    // Persist a recovery snapshot on each edit so a crash never loses work.
    if let Some(snap) = state.snapshot_for_recovery(&session_id).await {
        let entry = entry_from_snapshot(snap);
        if let Err(e) = write_snapshot(&entry) {
            log::warn!("[documents] could not write recovery snapshot: {e}");
        }
    }
    Ok(())
}

/// Get a full session (including content) for the editor.
#[tauri::command]
pub async fn get_document(
    state: State<'_, SessionManager>,
    session_id: String,
) -> CommandResult<DocumentSessionWire> {
    let s = state.get(&session_id).await.map_err(command_err)?;
    Ok(DocumentSessionWire::from(s))
}

/// List all sessions as summaries (no content).
#[tauri::command]
pub async fn list_documents(
    state: State<'_, SessionManager>,
) -> CommandResult<Vec<SessionSummary>> {
    let active = state.active_id().await;
    let sessions = state.list().await;
    Ok(sessions
        .iter()
        .map(|s| SessionSummary::from_session(s, active.as_deref() == Some(&s.id)))
        .collect())
}

/// Session ids that are currently dirty.
#[tauri::command]
pub async fn list_dirty_documents(state: State<'_, SessionManager>) -> CommandResult<Vec<String>> {
    Ok(state.list_dirty().await)
}

/// Close a session. Returns whether it was dirty before removal, so the caller
/// can decide whether to have prompted first.
///
/// M2: also best-effort releases the crossnote render session so preview
/// state does not leak after the tab is closed (§ M2 acceptance).
#[tauri::command]
pub async fn close_document(
    state: State<'_, SessionManager>,
    sidecar: State<'_, crate::sidecar::SidecarManager>,
    session_id: String,
) -> CommandResult<bool> {
    let was_dirty = state.close(&session_id).await.map_err(command_err)?;
    // Best-effort: release the render sidecar session (ignore if sidecar is down).
    let params = serde_json::json!({ "sessionId": session_id });
    if let Err(e) = sidecar.request("closeSession", params).await {
        log::debug!("[documents] closeSession on close_document: {e}");
    }
    Ok(was_dirty)
}

/// Set the active tab.
#[tauri::command]
pub async fn set_active_document(
    state: State<'_, SessionManager>,
    session_id: Option<String>,
) -> CommandResult<()> {
    state.set_active(session_id.as_deref()).await;
    Ok(())
}

/// Get the active session id (if any).
#[tauri::command]
pub async fn active_document(state: State<'_, SessionManager>) -> CommandResult<Option<String>> {
    Ok(state.active_id().await)
}

/// Check whether the on-disk file changed since open/save.
#[tauri::command]
pub async fn check_external_change(
    state: State<'_, SessionManager>,
    session_id: String,
) -> CommandResult<bool> {
    state
        .check_external_change(&session_id)
        .await
        .map_err(command_err)
}

/// Wire shape for a session sent to the frontend: camelCase keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSessionWire {
    pub id: String,
    #[serde(default, rename = "filePath")]
    pub file_path: Option<PathBuf>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub content: String,
    #[serde(rename = "savedContentHash")]
    pub saved_content_hash: String,
    pub encoding: String,
    #[serde(rename = "lineEnding")]
    pub line_ending: String,
    pub dirty: bool,
    #[serde(rename = "readOnly")]
    pub read_only: bool,
    pub mode: String,
    pub revision: u64,
    #[serde(rename = "lastSavedRevision")]
    pub last_saved_revision: u64,
    #[serde(rename = "externalMtimeMs")]
    pub external_mtime_ms: Option<i64>,
    #[serde(rename = "recoveryKey")]
    pub recovery_key: String,
}

impl From<DocumentSession> for DocumentSessionWire {
    fn from(s: DocumentSession) -> Self {
        let encoding = serde_json::to_value(s.encoding)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "utf-8".into());
        let line_ending = serde_json::to_value(s.line_ending)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "lf".into());
        let mode = serde_json::to_value(s.mode)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "source".into());
        Self {
            id: s.id,
            file_path: s.file_path,
            display_name: s.display_name,
            content: s.content,
            saved_content_hash: s.saved_content_hash,
            encoding,
            line_ending,
            dirty: s.dirty,
            read_only: s.read_only,
            mode,
            revision: s.revision,
            last_saved_revision: s.last_saved_revision,
            external_mtime_ms: s.external_mtime_ms,
            recovery_key: s.recovery_key,
        }
    }
}

/// Discard a recovery snapshot by key (no-op if missing). Used when the user
/// explicitly discards a recovery offer without restoring it.
#[tauri::command]
pub async fn discard_recovery_snapshot(recovery_key: String) -> CommandResult<()> {
    crate::recovery::delete_snapshot(&recovery_key).map_err(command_err)
}
