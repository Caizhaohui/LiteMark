//! Recent-files commands (M1).

use crate::error::command_err;
use crate::error::CommandResult;
use crate::files::recent::{RecentEntry, RecentList};
use std::path::PathBuf;

/// Get the recent-files list.
#[tauri::command]
pub async fn get_recent_files() -> CommandResult<Vec<RecentEntryWire>> {
    let list = RecentList::load().map_err(command_err)?;
    Ok(list
        .entries
        .into_iter()
        .map(RecentEntryWire::from)
        .collect())
}

/// Pin or unpin a recent entry.
#[tauri::command]
pub async fn set_recent_pinned(path: String, pinned: bool) -> CommandResult<()> {
    RecentList::set_pinned(&PathBuf::from(path), pinned).map_err(command_err)?;
    Ok(())
}

/// Clear all unpinned recent entries.
#[tauri::command]
pub async fn clear_recent_files() -> CommandResult<()> {
    RecentList::clear_unpinned().map_err(command_err)?;
    Ok(())
}

/// Wire shape for a recent entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentEntryWire {
    pub path: String,
    #[serde(rename = "lastOpenedAt")]
    pub last_opened_at: String,
    #[serde(default)]
    pub pinned: bool,
}

impl From<RecentEntry> for RecentEntryWire {
    fn from(e: RecentEntry) -> Self {
        Self {
            path: e.path,
            last_opened_at: e.last_opened_at,
            pinned: e.pinned,
        }
    }
}
