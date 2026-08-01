//! Path normalization and app-directory resolution (DEVELOPMENT_PLAN.md §6.3,
//! M1 acceptance: paths including "长路径、中文路径、emoji 文件名").
//!
//! Responsibilities:
//! - Resolve the recovery directory under `%LOCALAPPDATA%\LiteMark\recovery`.
//! - Resolve the app-data directory (recent files list, etc.).
//! - Canonicalize/normalize paths, applying the `\\?\` long-path prefix on
//!   Windows so paths longer than `MAX_PATH` (260) work without registry hacks.

use crate::error::{ErrorCode, SidecarError};
use std::path::{Path, PathBuf};

/// The vendor/app segments appended under LOCALAPPDATA.
const VENDOR: &str = "LiteMark";
const APP: &str = "LiteMark";
const RECOVERY_DIR_NAME: &str = "recovery";

/// The vendor/app directory under LOCALAPPDATA: `%LOCALAPPDATA%\LiteMark`.
pub fn app_data_dir() -> Result<PathBuf, SidecarError> {
    let base = dirs::data_local_dir().ok_or_else(|| {
        SidecarError::new(
            ErrorCode::PathNotAuthorized,
            "could not resolve the local app-data directory (LOCALAPPDATA)",
        )
    })?;
    Ok(base.join(VENDOR).join(APP))
}

/// The recovery directory: `%LOCALAPPDATA%\LiteMark\LiteMark\recovery` (§6.3).
pub fn recovery_dir() -> Result<PathBuf, SidecarError> {
    Ok(app_data_dir()?.join(RECOVERY_DIR_NAME))
}

/// Resolve `dirs::data_local_dir()`. On Windows this is `%LOCALAPPDATA%`.
/// Exposed for tests and diagnostics.
pub fn local_app_data() -> Option<PathBuf> {
    dirs::data_local_dir()
}

/// Normalize a user-supplied path. On Windows, if the lexical path exceeds
/// `MAX_PATH` after becoming absolute, the `\\?\` prefix is applied so the
/// Win32 API accepts long paths without per-machine LongPathsEnabled registry.
///
/// This is lexical (no filesystem access); it does not resolve symlinks or
/// verify existence.
pub fn normalize_long_path(path: &Path) -> Result<PathBuf, SidecarError> {
    // Make the path absolute relative to CWD if it isn't already. Use
    // current_dir (no IO on the target itself).
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        let cwd = std::env::current_dir().map_err(|e| {
            SidecarError::new(
                ErrorCode::PathNotAuthorized,
                format!("cwd resolution failed: {e}"),
            )
        })?;
        cwd.join(path)
    };

    Ok(if cfg!(windows) {
        apply_long_prefix(&absolute)
    } else {
        absolute
    })
}

/// Apply the `\\?\` prefix to an absolute Windows path if it is long enough to
/// risk exceeding `MAX_PATH`. Verbatim paths bypass the Win32 path
/// normalization (which caps at 260), enabling long paths.
pub fn apply_long_prefix(absolute: &Path) -> PathBuf {
    let s = absolute.to_string_lossy().into_owned();
    const PREFIX: &str = r"\\?\";
    // Already verbatim — leave as is.
    if s.starts_with(PREFIX) {
        return absolute.to_path_buf();
    }
    // Only add the prefix when the path is genuinely long; short paths work
    // without it and some APIs behave oddly with a verbatim prefix.
    if s.len() > 248 {
        // Convert forward slashes to backslashes for the verbatim form.
        let normalized = s.replace('/', r"\");
        return PathBuf::from(format!("{PREFIX}{normalized}"));
    }
    absolute.to_path_buf()
}

/// Derive a display name (for tabs) from a file path: the file stem, or
/// "Untitled" if the path is None.
pub fn display_name(file_path: Option<&Path>) -> String {
    match file_path {
        Some(p) => p
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Untitled".to_string()),
        None => "Untitled".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_uses_file_stem() {
        let p = Path::new(r"D:\docs\我的笔记.md");
        assert_eq!(display_name(Some(p)), "我的笔记");
    }

    #[test]
    fn display_name_handles_emoji_filename() {
        let p = Path::new(r"C:\tmp\hello 🌍.markdown");
        assert_eq!(display_name(Some(p)), "hello 🌍");
    }

    #[test]
    fn display_name_untitled_when_none() {
        assert_eq!(display_name(None), "Untitled");
    }

    #[test]
    fn recovery_dir_is_under_localappdata() {
        let dir = recovery_dir().unwrap();
        let s = dir.to_string_lossy().to_string();
        assert!(
            s.ends_with("recovery"),
            "recovery dir should end with 'recovery': {s}"
        );
        assert!(
            s.contains("LiteMark"),
            "recovery dir should be under LiteMark: {s}"
        );
    }

    #[test]
    fn app_data_dir_is_deterministic() {
        let a = app_data_dir().unwrap();
        let b = app_data_dir().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn long_path_gets_verbatim_prefix() {
        // Build a path > 248 chars.
        let mut long = String::from("D:\\");
        for _ in 0..50 {
            long.push_str("subdir\\");
        }
        long.push_str("file.md");
        let p = Path::new(&long);
        let normalized = normalize_long_path(p).unwrap();
        let s = normalized.to_string_lossy().to_string();
        assert!(
            s.starts_with(r"\\?\"),
            "long path should get verbatim prefix: {s}"
        );
    }

    #[test]
    fn short_path_keeps_no_prefix() {
        let p = Path::new(r"D:\docs\note.md");
        let normalized = normalize_long_path(p).unwrap();
        let s = normalized.to_string_lossy().to_string();
        assert!(
            !s.starts_with(r"\\?\"),
            "short path should not get verbatim prefix: {s}"
        );
    }

    #[test]
    fn chinese_path_normalizes() {
        let p = Path::new(r"D:\文档\笔记\测试.md");
        let normalized = normalize_long_path(p).unwrap();
        assert!(normalized.to_string_lossy().contains("测试"));
    }

    #[test]
    fn already_verbatim_path_is_left_alone() {
        let p = PathBuf::from(r"\\?\D:\deep\nested\file.md");
        let normalized = normalize_long_path(&p).unwrap();
        let s = normalized.to_string_lossy().to_string();
        // Should not be double-prefixed.
        assert!(!s.starts_with(r"\\?\\?\"));
        assert!(s.starts_with(r"\\?\"));
    }
}
