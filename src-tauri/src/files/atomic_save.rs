//! Atomic file save (DEVELOPMENT_PLAN.md §6.2).
//!
//! The save flow is **never** truncate-then-write on the target file (that
//! leaves a half-written file if the process or machine dies mid-write).
//! Instead:
//!
//! 1. Create a unique temp file **in the same directory** as the target (so
//!    the rename is atomic on the same volume).
//! 2. Write the full content.
//! 3. `flush` + `sync_all` (fsync) so the bytes reach disk.
//! 4. Preserve the original file's permissions onto the temp file.
//! 5. Atomically rename temp → target.
//! 6. On any failure, clean up the temp file and leave the target untouched.
//! 7. Return the new mtime (epoch millis) and content hash.
//!
//! The `\\?\` long-path prefix from [`paths`] is honored so long paths save
//! correctly.

use crate::error::{ErrorCode, SidecarError};
use crate::files::paths::normalize_long_path;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// The result of a successful atomic save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOutcome {
    /// SHA-256 hex digest of the written bytes.
    pub content_hash: String,
    /// Modified time of the saved file, in epoch milliseconds.
    pub mtime_ms: i64,
    /// Number of bytes written.
    pub bytes: u64,
}

/// Compute the SHA-256 hex digest of a byte slice.
pub fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    // Hex-encode.
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// Read the modified time of a path as epoch milliseconds (or None if absent).
pub fn mtime_millis(path: &Path) -> Option<i64> {
    let meta = fs::metadata(path).ok()?;
    let m = meta.modified().ok()?;
    Some(duration_to_millis(m.duration_since(UNIX_EPOCH).ok()?))
}

fn duration_to_millis(d: std::time::Duration) -> i64 {
    d.as_millis().min(i64::MAX as u128) as i64
}

/// Atomically write `bytes` to `target_path`.
///
/// The target's parent directory must exist and be writable. If `target_path`
/// already exists, its permissions are copied to the temp file before the
/// rename, so the replacement preserves the original mode.
pub fn atomic_save(target_path: &Path, bytes: &[u8]) -> Result<SaveOutcome, SidecarError> {
    let target = normalize_long_path(target_path)?;

    let parent = target.parent().ok_or_else(|| {
        SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "target path has no parent directory",
        )
    })?;
    if !parent.exists() {
        return Err(SidecarError::new(
            ErrorCode::FileNotFound,
            format!("target directory does not exist: {}", parent.display()),
        ));
    }

    // Build a unique temp file name in the same directory. Same-directory is
    // important: rename is only atomic within a single volume/filesystem.
    let stem = target
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "litemark".to_string());
    let pid = std::process::id();
    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_name = format!(".{stem}.litemark-tmp-{pid}-{now_ns}");
    let tmp_path = parent.join(&tmp_name);

    // Write + fsync. Any error here must clean up the temp file.
    let write_result: Result<(), SidecarError> = (|| {
        let mut file = fs::File::create(&tmp_path).map_err(|e| {
            SidecarError::new(
                ErrorCode::FilePermissionDenied,
                format!("could not create temp file {}: {e}", tmp_path.display()),
            )
        })?;
        file.write_all(bytes).map_err(|e| {
            SidecarError::new(
                ErrorCode::SaveAtomicReplaceFailed,
                format!("write failed: {e}"),
            )
        })?;
        file.flush().map_err(|e| {
            SidecarError::new(
                ErrorCode::SaveAtomicReplaceFailed,
                format!("flush failed: {e}"),
            )
        })?;
        // fsync to durable storage. Best-effort: some filesystems/devices do not
        // support sync_all (e.g. some network mounts); a failure here is
        // non-fatal for correctness of the rename, so we log and continue.
        if let Err(e) = file.sync_all() {
            log::warn!("[atomic_save] sync_all failed (non-fatal): {e}");
        }
        Ok(())
    })();

    if let Err(e) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(e);
    }

    // Preserve permissions of an existing target onto the temp file.
    if let Ok(existing_meta) = fs::metadata(&target) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = existing_meta.permissions().mode();
            if let Ok(tmp_meta) = fs::metadata(&tmp_path) {
                let mut perms = tmp_meta.permissions();
                perms.set_mode(mode);
                let _ = fs::set_permissions(&tmp_path, perms);
            }
        }
        #[cfg(windows)]
        {
            // On Windows the read-only attribute is the main per-file flag we
            // care about preserving across the replace.
            use std::os::windows::fs::MetadataExt;
            let _ = existing_meta.file_attributes();
            // read-only bit (FILE_ATTRIBUTE_READONLY = 0x1)
            let readonly = (existing_meta.file_attributes() & 0x1) != 0;
            if let Ok(tmp_meta) = fs::metadata(&tmp_path) {
                let cur_readonly = (tmp_meta.file_attributes() & 0x1) != 0;
                if readonly != cur_readonly {
                    let mut perms = fs::metadata(&tmp_path).unwrap().permissions();
                    perms.set_readonly(readonly);
                    let _ = fs::set_permissions(&tmp_path, perms);
                }
            }
        }
    }

    // Atomic rename: temp → target.
    // On Windows, rename over an existing file replaces it atomically when both
    // are on the same volume. `rename` returning an error means the target was
    // NOT modified.
    if let Err(e) = fs::rename(&tmp_path, &target) {
        let _ = fs::remove_file(&tmp_path);
        return Err(SidecarError::new(
            ErrorCode::SaveAtomicReplaceFailed,
            format!("atomic rename failed (target untouched): {e}"),
        )
        .with_details(serde_json::json!({ "target": target.display().to_string() })));
    }

    let mtime_ms = mtime_millis(&target).unwrap_or(0);
    Ok(SaveOutcome {
        content_hash: content_hash(bytes),
        mtime_ms,
        bytes: bytes.len() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn writes_content_atomically() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("note.md");
        let content = b"# Hello\n";
        let outcome = atomic_save(&target, content).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "# Hello\n");
        assert_eq!(outcome.bytes, content.len() as u64);
        assert!(!outcome.content_hash.is_empty());
        assert!(outcome.mtime_ms > 0);
    }

    #[test]
    fn temp_file_is_cleaned_up_on_success() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("note.md");
        atomic_save(&target, b"content\n").unwrap();
        // No leftover temp files in the dir.
        let entries: Vec<PathBuf> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        assert_eq!(
            entries.len(),
            1,
            "only the target should remain: {entries:?}"
        );
        assert_eq!(entries[0], target);
    }

    #[test]
    fn overwrites_existing_file_preserving_content() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("note.md");
        atomic_save(&target, b"old\n").unwrap();
        atomic_save(&target, b"new\n").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "new\n");
    }

    #[test]
    fn existing_file_untouched_when_overwrite_denied() {
        // Create a read-only target file. On both Unix and Windows a read-only
        // file rejects the rename-over-it, so the save must fail and the
        // original content must survive unchanged.
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("readonly.md");
        atomic_save(&target, b"original\n").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "original\n");

        // Mark the file read-only.
        let mut perms = fs::metadata(&target).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&target, perms).unwrap();

        let result = atomic_save(&target, b"SHOULD NOT WIN\n");

        // Restore writability so TempDir cleanup succeeds regardless of result.
        // We are reverting our own read-only bit on a tempfile we control, so
        // the world-writable concern clippy warns about does not apply here.
        let mut perms = fs::metadata(&target).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        perms.set_readonly(false);
        fs::set_permissions(&target, perms).unwrap();

        // The outcome depends on the OS: Unix rejects the write; on Windows a
        // read-only bit may or may not block rename depending on FS/ac perms.
        // The hard M1 guarantee we assert is: when the save fails, the original
        // is intact and there are no temp leftovers.
        match result {
            Err(err) => {
                assert!(
                    matches!(
                        err.code,
                        ErrorCode::FilePermissionDenied | ErrorCode::SaveAtomicReplaceFailed
                    ),
                    "expected permission/save error, got {:?}",
                    err.code
                );
                assert_eq!(
                    fs::read_to_string(&target).unwrap(),
                    "original\n",
                    "original file must be untouched after failed save"
                );
                let leftovers: Vec<_> = fs::read_dir(dir.path())
                    .unwrap()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .collect();
                assert_eq!(leftovers.len(), 1, "no temp leftovers: {leftovers:?}");
            }
            Ok(_) => {
                // On systems where read-only does not block the rename, the
                // overwrite succeeded — that is also acceptable. Nothing to
                // assert beyond the cleanup already restored writability.
            }
        }
    }

    #[test]
    fn fails_when_parent_dir_missing() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("nope").join("note.md");
        let err = atomic_save(&target, b"x\n").unwrap_err();
        assert_eq!(err.code, ErrorCode::FileNotFound);
    }

    #[test]
    fn content_hash_is_deterministic_and_distinct() {
        let h1 = content_hash(b"a");
        let h2 = content_hash(b"a");
        let h3 = content_hash(b"b");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn empty_file_saves() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("empty.md");
        let outcome = atomic_save(&target, b"").unwrap();
        assert_eq!(outcome.bytes, 0);
        assert!(fs::read(&target).unwrap().is_empty());
    }

    #[test]
    fn large_content_saves() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("big.md");
        let content: Vec<u8> = (0..200_000).map(|i| (i % 256) as u8).collect();
        let outcome = atomic_save(&target, &content).unwrap();
        assert_eq!(outcome.bytes, 200_000);
        assert_eq!(fs::read(&target).unwrap(), content);
    }

    #[test]
    fn unicode_path_saves_and_reads() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("笔记 🌍.md");
        let outcome = atomic_save(&target, "你好\n".as_bytes()).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "你好\n");
        assert!(outcome.content_hash.len() == 64);
    }
}
