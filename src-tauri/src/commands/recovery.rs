//! Recovery commands (M1). On launch the webview queries pending recovery
//! snapshots; the user can restore or discard them.

use crate::error::{command_err, CommandResult};
use crate::recovery::{delete_all, read_all, RecoveryEntry};
use crate::session::SessionManager;
use tauri::State;

/// A pending recovery snapshot offered to the user.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecoveryEntryWire {
    pub session_id: String,
    #[serde(rename = "originalPath")]
    pub original_path: Option<std::path::PathBuf>,
    #[serde(rename = "capturedAt")]
    pub captured_at: String,
    pub revision: u64,
    pub content: String,
    #[serde(rename = "recoveryKey")]
    pub recovery_key: String,
}

impl From<RecoveryEntry> for RecoveryEntryWire {
    fn from(e: RecoveryEntry) -> Self {
        Self {
            session_id: e.session_id,
            original_path: e.original_path,
            captured_at: e.captured_at,
            revision: e.revision,
            content: e.content,
            recovery_key: e.recovery_key,
        }
    }
}

/// Return all pending recovery snapshots (newest first).
#[tauri::command]
pub async fn get_pending_recovery() -> CommandResult<Vec<RecoveryEntryWire>> {
    let entries = read_all().map_err(command_err)?;
    Ok(entries.into_iter().map(RecoveryEntryWire::from).collect())
}

/// Restore a recovery snapshot into a new session and delete the snapshot.
/// Returns the new session id (the document is dirty until saved).
#[tauri::command]
pub async fn restore_recovery_snapshot(
    state: State<'_, SessionManager>,
    recovery_key: String,
) -> CommandResult<String> {
    let entries = read_all().map_err(command_err)?;
    let entry = entries
        .into_iter()
        .find(|e| e.recovery_key == recovery_key)
        .ok_or_else(|| {
            command_err(crate::error::SidecarError::new(
                crate::error::ErrorCode::FileNotFound,
                "recovery snapshot not found",
            ))
        })?;
    let id = state.new_document().await;
    // Seed the new session with the recovered content.
    state
        .set_content(&id, entry.content.clone())
        .await
        .map_err(command_err)?;
    // If there is an original path, remember it so Save writes back there.
    // (We set it via save_as semantics lazily; for M1 we leave file_path=None
    //  and let the user explicitly Save / Save As, which is the safer default.)
    delete_snapshot_cmd(recovery_key.clone()).await?;
    Ok(id)
}

/// Discard a single recovery snapshot.
#[tauri::command]
pub async fn discard_recovery_snapshot_cmd(recovery_key: String) -> CommandResult<()> {
    delete_snapshot_cmd(recovery_key).await
}

async fn delete_snapshot_cmd(recovery_key: String) -> CommandResult<()> {
    crate::recovery::delete_snapshot(&recovery_key).map_err(command_err)
}

/// Discard all pending recovery snapshots; returns how many were removed.
#[tauri::command]
pub async fn discard_all_recovery() -> CommandResult<usize> {
    delete_all().map_err(command_err)
}
