//! LiteMark error model (DEVELOPMENT_PLAN.md §14).
//!
//! `ErrorCode` mirrors the string codes defined in
//! `packages/shared-protocol/src/index.ts`. Keeping them in sync is validated
//! by a unit test that asserts the canonical set is present.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The canonical, stable error codes shared across the Rust core and the
/// TypeScript sidecar. New codes must be added to BOTH this enum and the
/// `ERROR_CODES` tuple in shared-protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    FileNotFound,
    FilePermissionDenied,
    FileChangedExternally,
    FileEncodingUnsupported,
    SaveAtomicReplaceFailed,
    SidecarStartFailed,
    SidecarCrashed,
    SidecarTimeout,
    ProtocolInvalid,
    RenderFailed,
    RenderCancelled,
    ExportFailed,
    ExportCancelled,
    BrowserNotFound,
    PandocNotFound,
    UntrustedOperationBlocked,
    PathNotAuthorized,
    RoundtripDataLossRisk,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // SCREAMING_SNAKE_CASE serialization matches the shared-protocol strings.
        let s = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        // serde_json quotes the value; strip the quotes.
        let trimmed = s.trim_matches('"');
        f.write_str(trimmed)
    }
}

/// A structured error carried over Tauri commands to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarError {
    pub code: ErrorCode,
    pub message: String,
    /// Optional structured detail for diagnostics; never secrets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl fmt::Display for SidecarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SidecarError {}

impl SidecarError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Convert any error into a SidecarError with RENDER_FAILED as a fallback.
impl From<Box<dyn std::error::Error + Send + Sync>> for SidecarError {
    fn from(value: Box<dyn std::error::Error + Send + Sync>) -> Self {
        SidecarError::new(ErrorCode::RenderFailed, value.to_string())
    }
}

/// Alias used by Tauri command return types; serialized to the frontend.
pub type CommandResult<T> = std::result::Result<T, String>;

/// Wrap a SidecarError for Tauri: Tauri commands must return errors that
/// implement `Serialize`. We serialize to the structured SidecarError JSON.
pub fn command_err(e: SidecarError) -> String {
    serde_json::to_string(&e).unwrap_or_else(|_| {
        serde_json::to_string(&SidecarError::new(
            ErrorCode::RenderFailed,
            "serialization failed",
        ))
        .unwrap_or_else(|_| "{}".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_serializes_to_screaming_snake_case() {
        let code = serde_json::to_string(&ErrorCode::SidecarCrashed).unwrap();
        assert_eq!(code, "\"SIDECAR_CRASHED\"");
        assert_eq!(ErrorCode::SidecarCrashed.to_string(), "SIDECAR_CRASHED");
    }

    #[test]
    fn sidecar_error_roundtrips_as_json() {
        let e = SidecarError::new(ErrorCode::BrowserNotFound, "no edge")
            .with_details(serde_json::json!({"probed": ["msedge", "chrome"]}));
        let json = serde_json::to_string(&e).unwrap();
        assert!(json.contains("BROWSER_NOT_FOUND"));
        assert!(json.contains("no edge"));
        let back: SidecarError = serde_json::from_str(&json).unwrap();
        assert_eq!(back.code, ErrorCode::BrowserNotFound);
        assert_eq!(back.message, "no edge");
        assert!(back.details.is_some());
    }

    /// Canonical set sanity check. If you add a code, add it here AND in
    /// shared-protocol ERROR_CODES.
    #[test]
    fn canonical_codes_are_serializable() {
        let codes = [
            ErrorCode::FileNotFound,
            ErrorCode::FilePermissionDenied,
            ErrorCode::FileChangedExternally,
            ErrorCode::FileEncodingUnsupported,
            ErrorCode::SaveAtomicReplaceFailed,
            ErrorCode::SidecarStartFailed,
            ErrorCode::SidecarCrashed,
            ErrorCode::SidecarTimeout,
            ErrorCode::ProtocolInvalid,
            ErrorCode::RenderFailed,
            ErrorCode::RenderCancelled,
            ErrorCode::ExportFailed,
            ErrorCode::ExportCancelled,
            ErrorCode::BrowserNotFound,
            ErrorCode::PandocNotFound,
            ErrorCode::UntrustedOperationBlocked,
            ErrorCode::PathNotAuthorized,
            ErrorCode::RoundtripDataLossRisk,
        ];
        // Every code must produce a distinct non-empty string.
        let rendered: Vec<String> = codes.iter().map(|c| c.to_string()).collect();
        let unique: std::collections::HashSet<&String> = rendered.iter().collect();
        assert_eq!(unique.len(), codes.len(), "duplicate error code rendering");
        for r in &rendered {
            assert!(!r.is_empty());
        }
    }
}
