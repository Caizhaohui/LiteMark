//! Command-line / file-association open paths.
//!
//! When the user double-clicks a `.md` file (or passes paths on the command
//! line), Windows launches LiteMark with those paths as argv. The single-
//! instance plugin only forwards args from a *second* process; cold start must
//! read `std::env::args()` itself and hand them to the UI.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Markdown extensions we accept from the shell / file association.
const MARKDOWN_EXTS: &[&str] = &["md", "markdown", "mdx", "mkd", "mkdn", "mdown"];

/// Pending paths from cold-start argv, consumed once by the frontend.
#[derive(Default)]
pub struct PendingCliFiles {
    files: Mutex<Vec<String>>,
}

impl PendingCliFiles {
    pub fn new(files: Vec<String>) -> Self {
        Self {
            files: Mutex::new(files),
        }
    }

    /// Take all pending paths (empty afterwards).
    pub fn take(&self) -> Vec<String> {
        self.files
            .lock()
            .map(|mut g| std::mem::take(&mut *g))
            .unwrap_or_default()
    }

    /// Peek without consuming (used to prioritize sidecar warm on CLI open).
    pub fn peek(&self) -> Vec<String> {
        self.files
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    /// Append more paths (e.g. if we ever buffer before the UI is ready).
    #[allow(dead_code)]
    pub fn extend(&self, extra: Vec<String>) {
        if let Ok(mut g) = self.files.lock() {
            for p in extra {
                if !g.iter().any(|e| e == &p) {
                    g.push(p);
                }
            }
        }
    }
}

/// Collect openable markdown paths from process argv (skip exe name + flags).
pub fn collect_cli_files<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .skip(1)
        .filter_map(|a| normalize_arg(a.as_ref()))
        .collect()
}

fn normalize_arg(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches('"').trim();
    if trimmed.is_empty() || trimmed.starts_with('-') {
        return None;
    }
    let path = PathBuf::from(trimmed);
    if !is_markdown_path(&path) {
        return None;
    }
    // Prefer absolute; if relative, resolve against current_dir.
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    // Accept if it exists as a file, or still looks like a markdown path
    // (shell sometimes races; open_file will surface a clear error).
    if absolute.is_file() || is_markdown_path(&absolute) {
        Some(absolute.to_string_lossy().into_owned())
    } else {
        None
    }
}

fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|s| {
            let lower = s.to_ascii_lowercase();
            MARKDOWN_EXTS.iter().any(|ext| *ext == lower)
        })
        .unwrap_or(false)
}

/// Filter a list of argv strings (already collected) into markdown file paths.
pub fn filter_file_args(argv: impl IntoIterator<Item = String>) -> Vec<String> {
    argv.into_iter()
        .filter_map(|a| normalize_arg(&a))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_exe_and_flags() {
        let args = vec![
            "litemark.exe".to_string(),
            "--some-flag".to_string(),
            "note.md".to_string(),
        ];
        let files = collect_cli_files(args);
        // relative note.md may not exist; still collected if extension matches
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("note.md"));
    }

    #[test]
    fn rejects_non_markdown() {
        assert!(normalize_arg("C:\\tmp\\a.txt").is_none());
        assert!(normalize_arg("-flag").is_none());
    }

    #[test]
    fn accepts_quoted_md() {
        let p = normalize_arg(r#""D:\docs\hello.markdown""#);
        assert!(p.is_some());
        assert!(p.unwrap().ends_with("hello.markdown"));
    }
}
