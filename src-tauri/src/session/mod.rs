//! In-memory document sessions (M1). The `SessionManager` is the Tauri state
//! singleton that owns all open documents; the webview drives it exclusively
//! through Rust commands (no direct fs permission).

pub mod manager;
pub mod model;

pub use manager::{RecoverySnapshot, SessionManager};
pub use model::{DocumentSession, EditorMode};
