//! `ping_sidecar` — the M0 architecture-verification command.
//!
//! Returns the sidecar's `ping` result (core version + crossnote version +
//! echoed timestamp) to the webview. Errors are serialized as structured
//! `SidecarError` JSON so the React layer can render actionable diagnostics.

use crate::error::{command_err, CommandResult, ErrorCode, SidecarError};
use crate::sidecar::SidecarManager;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Mirrors the TypeScript `PingResult` from shared-protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub version: String,
    #[serde(rename = "crossnoteVersion")]
    pub crossnote_version: String,
    #[serde(
        rename = "receivedAt",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub received_at: Option<String>,
}

/// Ping the render sidecar and return its version info.
#[tauri::command]
pub async fn ping_sidecar(
    state: State<'_, SidecarManager>,
    sent_at: Option<String>,
) -> CommandResult<PingResult> {
    let mut params = serde_json::Map::new();
    if let Some(ts) = sent_at {
        params.insert("sentAt".to_string(), serde_json::Value::String(ts));
    }
    let result = state
        .request("ping", serde_json::Value::Object(params))
        .await
        .map_err(|e: SidecarError| command_err(e))?;
    serde_json::from_value::<PingResult>(result)
        .map_err(|e| command_err(SidecarError::new(ErrorCode::ProtocolInvalid, e.to_string())))
}

/// Ensure the render sidecar process is running (spawn + ping if needed).
/// Used by the UI on mount / CLI open so preview does not pay cold-start.
#[tauri::command]
pub async fn warm_sidecar(state: State<'_, SidecarManager>) -> CommandResult<bool> {
    state.ensure_warm().await.map_err(command_err)?;
    Ok(true)
}
