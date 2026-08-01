//! File-system operations for documents (M1). All file IO is brokered by Rust
//! — the webview has no `fs` permission (see `docs/security.md`). This module
//! groups the pure, well-tested building blocks: encoding detection, atomic
//! save, path normalization, and the recent-files list.

pub mod atomic_save;
pub mod encoding;
pub mod export_prefs;
pub mod paths;
pub mod recent;
pub mod settings;

/// Read a file fully and decode it (encoding + line endings), returning the
/// decoded content along with metadata. Honors long paths.
pub fn read_and_decode(
    path: &std::path::Path,
) -> Result<encoding::DecodedFile, crate::error::SidecarError> {
    let normalized = paths::normalize_long_path(path)?;
    let bytes = std::fs::read(&normalized).map_err(|e| {
        let code = if e.kind() == std::io::ErrorKind::NotFound {
            crate::error::ErrorCode::FileNotFound
        } else if e.kind() == std::io::ErrorKind::PermissionDenied {
            crate::error::ErrorCode::FilePermissionDenied
        } else {
            crate::error::ErrorCode::FileNotFound
        };
        crate::error::SidecarError::new(code, format!("read {}: {e}", normalized.display()))
    })?;
    encoding::decode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn read_and_decode_roundtrips_utf8_bom_crlf() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("note.md");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(encoding::UTF8_BOM);
        bytes.extend_from_slice(b"title\r\nbody\r\n");
        fs::write(&p, &bytes).unwrap();

        let decoded = read_and_decode(&p).unwrap();
        assert_eq!(decoded.encoding, encoding::Encoding::Utf8Bom);
        assert_eq!(decoded.line_ending, encoding::LineEnding::Crlf);
        assert_eq!(decoded.content, "title\nbody\n");
    }

    #[test]
    fn read_and_decode_missing_file_is_file_not_found() {
        let dir = TempDir::new().unwrap();
        let p = dir.path().join("nope.md");
        let err = read_and_decode(&p).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::FileNotFound);
    }
}
