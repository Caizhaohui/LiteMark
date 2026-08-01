//! Crash-recovery store (DEVELOPMENT_PLAN.md §6.3).
//!
//! Every dirty document periodically writes a snapshot to
//! `%LOCALAPPDATA%\LiteMark\recovery\`. On the next launch the app reads all
//! pending snapshots and offers to restore them. After a successful, clean
//! save the corresponding snapshot is deleted.
//!
//! Per the plan, each document keeps at most a few recent snapshots; we keep
//! the latest one per `recovery_key` (newer revisions overwrite older for the
//! same key), plus a global cap.

use crate::error::{ErrorCode, SidecarError};
use crate::files::atomic_save::atomic_save;
use crate::files::paths::recovery_dir;
use crate::session::RecoverySnapshot;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Maximum number of recovery files retained globally (housekeeping bound).
const MAX_RECOVERY_FILES: usize = 50;

/// The on-disk snapshot shape (§6.3). `snake_case` keys are mapped to
/// camelCase in shared-protocol for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryEntry {
    pub session_id: String,
    pub original_path: Option<PathBuf>,
    pub captured_at: String,
    pub revision: u64,
    pub content: String,
    pub recovery_key: String,
}

impl RecoveryEntry {
    /// Filename for a snapshot: `<recovery_key>.json`. Since recovery_key is a
    /// content-hash of the path (or `new-<uuid>`), it is filesystem-safe and
    /// stable across restarts for the same document.
    fn file_name(&self) -> String {
        format!("{}.json", sanitize_key(&self.recovery_key))
    }
}

/// Strip anything that is not a filename-safe character.
fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Build a [`RecoveryEntry`] from a live snapshot plus the current timestamp.
pub fn entry_from_snapshot(snap: RecoverySnapshot) -> RecoveryEntry {
    RecoveryEntry {
        session_id: snap.session_id,
        original_path: snap.original_path,
        captured_at: Utc::now().to_rfc3339(),
        revision: snap.revision,
        content: snap.content,
        recovery_key: snap.recovery_key,
    }
}

/// Persist a snapshot to the recovery directory (atomic). Overwrites any prior
/// snapshot for the same recovery key (we keep only the newest per document).
pub fn write_snapshot(entry: &RecoveryEntry) -> Result<(), SidecarError> {
    let dir = recovery_dir()?;
    fs::create_dir_all(&dir).map_err(|e| {
        SidecarError::new(
            ErrorCode::FilePermissionDenied,
            format!("could not create recovery dir: {e}"),
        )
    })?;
    let path = dir.join(entry.file_name());
    let json = serde_json::to_string(entry).map_err(|e| {
        SidecarError::new(
            ErrorCode::FileChangedExternally,
            format!("encode snapshot: {e}"),
        )
    })?;
    atomic_save(&path, json.as_bytes())?;
    housekeep(&dir);
    Ok(())
}

/// Read all pending recovery snapshots. Files that fail to parse are skipped
/// (a corrupt single snapshot must not prevent restoring the others).
pub fn read_all() -> Result<Vec<RecoveryEntry>, SidecarError> {
    let dir = recovery_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for item in fs::read_dir(&dir).map_err(|e| {
        SidecarError::new(
            ErrorCode::FilePermissionDenied,
            format!("read recovery dir: {e}"),
        )
    })? {
        let Ok(item) = item else { continue };
        let path = item.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        match serde_json::from_str::<RecoveryEntry>(&text) {
            Ok(e) => entries.push(e),
            Err(e) => {
                log::warn!(
                    "[recovery] skipping corrupt snapshot {}: {e}",
                    path.display()
                );
            }
        }
    }
    // Sort newest-first by captured_at (ISO-8601 string compare is chronological).
    entries.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));
    Ok(entries)
}

/// Delete the snapshot for a given recovery key (called after a clean save or
/// explicit discard). Missing file is a no-op.
pub fn delete_snapshot(recovery_key: &str) -> Result<(), SidecarError> {
    let path = recovery_dir()?.join(format!("{}.json", sanitize_key(recovery_key)));
    if path.exists() {
        fs::remove_file(&path).map_err(|e| {
            SidecarError::new(
                ErrorCode::FilePermissionDenied,
                format!("delete recovery snapshot {}: {e}", path.display()),
            )
        })?;
    }
    Ok(())
}

/// Delete all pending snapshots (e.g. user chooses "discard all").
pub fn delete_all() -> Result<usize, SidecarError> {
    let dir = recovery_dir()?;
    if !dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for item in fs::read_dir(&dir).map_err(|e| {
        SidecarError::new(
            ErrorCode::FilePermissionDenied,
            format!("read recovery dir: {e}"),
        )
    })? {
        let Ok(item) = item else { continue };
        let path = item.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json")
            && fs::remove_file(&path).is_ok()
        {
            count += 1;
        }
    }
    Ok(count)
}

/// Keep the recovery dir from growing without bound: if there are more than
/// `MAX_RECOVERY_FILES` snapshots, drop the oldest by `captured_at`.
fn housekeep(dir: &std::path::Path) {
    let Ok(items) = fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<(PathBuf, String)> = Vec::new();
    for item in items.flatten() {
        let path = item.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let captured = fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<RecoveryEntry>(&t).ok())
            .map(|e| e.captured_at)
            .unwrap_or_default();
        files.push((path, captured));
    }
    if files.len() <= MAX_RECOVERY_FILES {
        return;
    }
    // Sort oldest-first by captured_at; delete the surplus oldest.
    files.sort_by(|a, b| a.1.cmp(&b.1));
    let surplus = files.len().saturating_sub(MAX_RECOVERY_FILES);
    for (path, _) in files.into_iter().take(surplus) {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::atomic_save::content_hash;
    use tempfile::TempDir;

    /// We cannot trivially redirect the real recovery dir (it is fixed under
    /// LOCALAPPDATA), so these tests exercise the pure logic: serialization,
    /// filename derivation, and housekeeping bounds. The filesystem round-trip
    /// is validated by writing to a temp dir directly.
    fn temp_entry(content: &str, rev: u64, path: Option<&str>) -> RecoveryEntry {
        RecoveryEntry {
            session_id: "sess-1".into(),
            original_path: path.map(std::path::PathBuf::from),
            captured_at: format!("2026-07-29T12:00:{rev:02}Z"),
            revision: rev,
            content: content.into(),
            recovery_key: format!("key-{rev}"),
        }
    }

    #[test]
    fn filename_derived_from_recovery_key() {
        let e = temp_entry("x", 1, None);
        assert_eq!(e.file_name(), "key-1.json");
    }

    #[test]
    fn sanitize_replaces_unsafe_chars() {
        assert_eq!(sanitize_key("file-abc/\\:weird"), "file-abc___weird");
    }

    #[test]
    fn json_roundtrip_preserves_all_fields() {
        let e = temp_entry("# hi\n中文", 5, Some(r"D:\docs\a.md"));
        let json = serde_json::to_string(&e).unwrap();
        let back: RecoveryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.content, "# hi\n中文");
        assert_eq!(back.revision, 5);
        assert_eq!(back.recovery_key, "key-5");
        assert_eq!(
            back.original_path.as_deref(),
            Some(std::path::Path::new(r"D:\docs\a.md"))
        );
    }

    #[test]
    fn write_then_read_then_delete_roundtrip_in_temp_dir() {
        // Bypass the fixed recovery_dir by testing atomic_save + parse directly.
        let dir = TempDir::new().unwrap();
        let e = temp_entry("recovery content", 3, Some(r"C:\tmp\note.md"));
        let path = dir.path().join(e.file_name());
        let json = serde_json::to_string(&e).unwrap();
        atomic_save(&path, json.as_bytes()).unwrap();
        assert!(path.exists());

        let read_back: RecoveryEntry =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(read_back.content, "recovery content");

        std::fs::remove_file(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn housekeep_keeps_cap_in_temp_dir() {
        let dir = TempDir::new().unwrap();
        // Write more than MAX_RECOVERY_FILES by calling housekeep logic on the
        // temp dir directly.
        for i in 0..(MAX_RECOVERY_FILES + 5) {
            let e = temp_entry("c", i as u64, None);
            let path = dir.path().join(e.file_name());
            let json = serde_json::to_string(&e).unwrap();
            std::fs::write(&path, json).unwrap();
        }
        housekeep(dir.path());
        let remaining = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
            .count();
        assert!(
            remaining <= MAX_RECOVERY_FILES,
            "housekeep must cap to {MAX_RECOVERY_FILES}, got {remaining}"
        );
    }

    #[test]
    fn recovery_key_is_stable_for_same_path() {
        let p = r"D:\docs\stable.md";
        let k1 = format!("file-{}", content_hash(p.as_bytes()));
        let k2 = format!("file-{}", content_hash(p.as_bytes()));
        assert_eq!(k1, k2);
    }
}
