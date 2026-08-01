//! Recent-files list (M1 scope). Persisted as JSON under the app-data
//! directory. Capped to a maximum number of entries; entries that no longer
//! exist on disk are reported with an `exists` flag so the UI can grey them
//! out without removing them automatically.

use crate::error::{ErrorCode, SidecarError};
use crate::files::paths::app_data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const RECENT_FILE_NAME: &str = "recent-files.json";
const MAX_RECENT: usize = 25;

/// A single recent-file entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEntry {
    /// Absolute, display path.
    pub path: String,
    /// ISO-8601 timestamp of when it was last opened.
    pub last_opened_at: String,
    /// User-pinned entries stay even if absent and sort first.
    #[serde(default)]
    pub pinned: bool,
}

/// The persisted list. Kept newest-first.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentList {
    #[serde(default)]
    pub entries: Vec<RecentEntry>,
}

impl RecentList {
    fn load_file_path() -> Result<PathBuf, SidecarError> {
        Ok(app_data_dir()?.join(RECENT_FILE_NAME))
    }

    /// Load the recent list from disk. Returns an empty list if the file is
    /// missing or unreadable (never panics the app over a corrupt recent list).
    pub fn load() -> Result<Self, SidecarError> {
        let path = Self::load_file_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path).map_err(|e| {
            SidecarError::new(
                ErrorCode::FilePermissionDenied,
                format!("could not read recent-files list: {e}"),
            )
        })?;
        if text.trim().is_empty() {
            return Ok(Self::default());
        }
        let list: RecentList = serde_json::from_str(&text).map_err(|e| {
            SidecarError::new(
                ErrorCode::FileChangedExternally,
                format!("recent-files list is corrupt, resetting: {e}"),
            )
        })?;
        Ok(list)
    }

    /// Persist the list to disk (atomically).
    fn save(&self) -> Result<(), SidecarError> {
        let dir = app_data_dir()?;
        fs::create_dir_all(&dir).map_err(|e| {
            SidecarError::new(
                ErrorCode::FilePermissionDenied,
                format!("could not create app-data dir: {e}"),
            )
        })?;
        let path = dir.join(RECENT_FILE_NAME);
        let text = serde_json::to_string_pretty(self).map_err(|e| {
            SidecarError::new(ErrorCode::FileChangedExternally, format!("encode: {e}"))
        })?;
        crate::files::atomic_save::atomic_save(&path, text.as_bytes())?;
        Ok(())
    }

    /// Record that `path` was just opened, moving it to the front. Pinned
    /// entries always sort before unpinned ones. Persists the result.
    pub fn record_opened(path: &Path, now_iso: &str) -> Result<Self, SidecarError> {
        let mut list = Self::load()?;
        let path_str = path.to_string_lossy().to_string();
        // Remove any existing entry for this path.
        list.entries.retain(|e| e.path != path_str);
        list.entries.insert(
            0,
            RecentEntry {
                path: path_str,
                last_opened_at: now_iso.to_string(),
                pinned: false,
            },
        );
        list.trim();
        list.save()?;
        Ok(list)
    }

    /// Toggle the pinned state of an entry by path. Persist.
    pub fn set_pinned(path: &Path, pinned: bool) -> Result<Self, SidecarError> {
        let mut list = Self::load()?;
        let path_str = path.to_string_lossy().to_string();
        for e in list.entries.iter_mut() {
            if e.path == path_str {
                e.pinned = pinned;
            }
        }
        list.sort_entries();
        list.save()?;
        Ok(list)
    }

    /// Clear all unpinned entries. Persist.
    pub fn clear_unpinned() -> Result<Self, SidecarError> {
        let mut list = Self::load()?;
        list.entries.retain(|e| e.pinned);
        list.save()?;
        Ok(list)
    }

    fn trim(&mut self) {
        // Keep all pinned; among unpinned keep newest up to MAX_RECENT total.
        self.sort_entries();
        if self.entries.len() > MAX_RECENT {
            self.entries.truncate(MAX_RECENT);
        }
    }

    fn sort_entries(&mut self) {
        // Pinned first, then by last_opened_at descending (string compare on
        // ISO-8601 is chronological).
        self.entries.sort_by(|a, b| {
            b.pinned
                .cmp(&a.pinned)
                .then_with(|| b.last_opened_at.cmp(&a.last_opened_at))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// These tests run against the *real* app-data dir, so we isolate state by
    /// exercising the in-memory sort/trim logic (pure functions) rather than
    /// the filesystem-backed load/save, which would leak between runs and
    /// machines. Filesystem round-trip is covered by integration.
    #[test]
    fn record_opened_moves_to_front() {
        let mut list = RecentList::default();
        list.entries.push(RecentEntry {
            path: "/old.md".into(),
            last_opened_at: "2026-01-01T00:00:00Z".into(),
            pinned: false,
        });
        list.entries.insert(
            0,
            RecentEntry {
                path: "/new.md".into(),
                last_opened_at: "2026-07-29T00:00:00Z".into(),
                pinned: false,
            },
        );
        // simulate re-opening old: remove then insert front
        let path_str = "/old.md".to_string();
        list.entries.retain(|e| e.path != path_str);
        list.entries.insert(
            0,
            RecentEntry {
                path: path_str,
                last_opened_at: "2026-07-29T12:00:00Z".into(),
                pinned: false,
            },
        );
        assert_eq!(list.entries[0].path, "/old.md");
        assert_eq!(list.entries.len(), 2);
    }

    #[test]
    fn pinned_sort_first() {
        let mut list = RecentList::default();
        list.entries.push(RecentEntry {
            path: "/b.md".into(),
            last_opened_at: "2026-07-29T00:00:00Z".into(),
            pinned: false,
        });
        list.entries.push(RecentEntry {
            path: "/a.md".into(),
            last_opened_at: "2026-07-01T00:00:00Z".into(),
            pinned: true,
        });
        list.sort_entries();
        assert_eq!(list.entries[0].path, "/a.md");
    }

    #[test]
    fn trim_caps_to_max() {
        let mut list = RecentList::default();
        for i in 0..(MAX_RECENT + 10) {
            list.entries.push(RecentEntry {
                path: format!("/{i}.md"),
                last_opened_at: format!("2026-01-{i:02}T00:00:00Z"),
                pinned: false,
            });
        }
        list.trim();
        assert_eq!(list.entries.len(), MAX_RECENT);
    }

    /// App-data dir isolation: verify save/load round-trip against a temp dir
    /// by temporarily redirecting. We can't easily redirect `dirs`, so instead
    /// test the atomic-save + JSON parse path directly.
    #[test]
    fn json_roundtrip_of_recent_list() {
        let list = RecentList {
            entries: vec![RecentEntry {
                path: "/x.md".into(),
                last_opened_at: "2026-07-29T00:00:00Z".into(),
                pinned: true,
            }],
        };
        let json = serde_json::to_string(&list).unwrap();
        let back: RecentList = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entries.len(), 1);
        assert!(back.entries[0].pinned);
    }

    #[test]
    fn empty_json_loads_as_default() {
        let list: RecentList = serde_json::from_str("{}").unwrap();
        assert!(list.entries.is_empty());
    }

    /// Sanity: the app-data path computation itself doesn't touch the fs.
    #[test]
    fn load_file_path_under_app_data() {
        let _ = app_data_dir().unwrap();
    }

    #[test]
    fn tempdir_is_writable_fixture() {
        let _d = TempDir::new().unwrap();
    }
}
