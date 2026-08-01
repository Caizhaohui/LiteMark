//! The Rust `DocumentSession` (DEVELOPMENT_PLAN.md §6.1). Mirrors the
//! TypeScript `DocumentSession` in shared-protocol. `dirty` is **derived from
//! the content hash**, never set by the UI as a guess.

use crate::files::atomic_save::content_hash;
use crate::files::encoding::{Encoding, LineEnding};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The editor mode. M1 only uses `source` (a temporary textarea); `hybrid` and
/// `preview` arrive in later milestones. All three are defined now so the
/// session shape is stable across milestones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EditorMode {
    #[default]
    Source,
    Hybrid,
    Preview,
}

/// A single open document held in memory. Serialized to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSession {
    pub id: String,
    /// None for a brand-new, unsaved document.
    #[serde(default)]
    pub file_path: Option<PathBuf>,
    pub display_name: String,
    pub content: String,
    /// SHA-256 hex of the bytes last confirmed saved (or the initial content).
    pub saved_content_hash: String,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    /// Derived from the content hash on every read — the UI must not set it.
    #[serde(default)]
    pub dirty: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub mode: EditorMode,
    /// Monotonic, incremented on every edit.
    #[serde(default)]
    pub revision: u64,
    /// The revision at the last successful save.
    #[serde(default)]
    pub last_saved_revision: u64,
    /// mtime (epoch ms) of the file on disk at open/save time, used for
    /// external-modification detection. None when there is no file yet.
    #[serde(default)]
    pub external_mtime_ms: Option<i64>,
    /// Stable key under which recovery snapshots are filed.
    pub recovery_key: String,
}

impl DocumentSession {
    /// Create a brand-new, empty, unsaved document.
    pub fn new_empty(id: String) -> Self {
        let initial = String::new();
        let hash = content_hash(initial.as_bytes());
        let recovery_key = format!("new-{id}");
        Self {
            id,
            file_path: None,
            display_name: "Untitled".to_string(),
            content: initial,
            saved_content_hash: hash,
            encoding: Encoding::Utf8,
            line_ending: LineEnding::Lf,
            dirty: false,
            read_only: false,
            mode: EditorMode::Source,
            revision: 0,
            last_saved_revision: 0,
            external_mtime_ms: None,
            recovery_key,
        }
    }

    /// Create a session from a freshly-decoded file.
    pub fn from_file(
        id: String,
        file_path: PathBuf,
        display_name: String,
        decoded: crate::files::encoding::DecodedFile,
        mtime_ms: Option<i64>,
        read_only: bool,
    ) -> Self {
        let hash = content_hash(
            crate::files::encoding::encode(&decoded.content, decoded.encoding, decoded.line_ending)
                .as_slice(),
        );
        let recovery_key = file_path
            .to_str()
            .map(|s| format!("file-{}", content_hash(s.as_bytes())))
            .unwrap_or_else(|| format!("new-{id}"));
        Self {
            id,
            file_path: Some(file_path),
            display_name,
            content: decoded.content,
            saved_content_hash: hash,
            encoding: decoded.encoding,
            line_ending: decoded.line_ending,
            dirty: false,
            read_only,
            mode: EditorMode::Source,
            revision: 0,
            last_saved_revision: 0,
            external_mtime_ms: mtime_ms,
            recovery_key,
        }
    }

    /// Recompute `dirty` by hashing the current content the same way the
    /// encoder would on save. This is the single source of truth for dirty.
    pub fn recompute_dirty(&mut self) {
        let encoded =
            crate::files::encoding::encode(&self.content, self.encoding, self.line_ending);
        self.dirty = content_hash(encoded.as_slice()) != self.saved_content_hash;
    }

    /// Apply an edit from the editor: replace the whole content (simplest M1
    /// model — Monaco/Milkdown in later milestones can be more granular).
    pub fn set_content(&mut self, new_content: String) {
        if self.content != new_content {
            self.content = new_content;
            self.revision = self.revision.saturating_add(1);
        }
        self.recompute_dirty();
    }

    /// Mark the session as just-saved with the given outcome.
    pub fn mark_saved(
        &mut self,
        content_hash_hex: String,
        mtime_ms: i64,
        file_path: Option<PathBuf>,
    ) {
        self.saved_content_hash = content_hash_hex;
        self.last_saved_revision = self.revision;
        self.external_mtime_ms = Some(mtime_ms);
        if let Some(p) = file_path {
            self.file_path = Some(p);
            if let Some(stem) = self
                .file_path
                .as_ref()
                .and_then(|p| p.file_stem())
                .and_then(|s| s.to_str())
            {
                self.display_name = stem.to_string();
            }
        }
        self.dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty_is_not_dirty() {
        let s = DocumentSession::new_empty("s1".into());
        let mut s = s;
        s.recompute_dirty();
        assert!(!s.dirty);
    }

    #[test]
    fn edit_makes_dirty_and_bumps_revision() {
        let mut s = DocumentSession::new_empty("s1".into());
        s.set_content("# hi\n".into());
        assert!(s.dirty);
        assert_eq!(s.revision, 1);
    }

    #[test]
    fn identical_set_content_does_not_bump_revision() {
        let mut s = DocumentSession::new_empty("s1".into());
        s.set_content("same\n".into());
        let rev = s.revision;
        s.set_content("same\n".into());
        assert_eq!(s.revision, rev, "no-op edit must not bump revision");
    }

    #[test]
    fn dirty_derived_from_hash_not_guessed() {
        // Even if dirty were force-set true, recompute resets it when content
        // matches the saved hash.
        let mut s = DocumentSession::new_empty("s1".into());
        s.dirty = true; // a UI bug sets it
        s.recompute_dirty();
        assert!(!s.dirty, "recompute must override a guessed dirty flag");
    }

    #[test]
    fn mark_saved_clears_dirty_and_updates_hash() {
        let mut s = DocumentSession::new_empty("s1".into());
        s.set_content("edited\n".into());
        assert!(s.dirty);
        let h = content_hash(b"edited\n");
        s.mark_saved(h.clone(), 12345, None);
        assert!(!s.dirty);
        assert_eq!(s.saved_content_hash, h);
        assert_eq!(s.external_mtime_ms, Some(12345));
    }

    #[test]
    fn session_serializes_to_frontend_shape() {
        let s = DocumentSession::new_empty("s1".into());
        let json = serde_json::to_value(&s).unwrap();
        // camelCase via serde rename defaults on fields with underscores? No —
        // we keep snake_case on the wire and map in shared-protocol. Just
        // sanity-check core fields exist.
        assert!(json.get("id").is_some());
        assert!(json.get("saved_content_hash").is_some());
        assert_eq!(json["encoding"], "utf-8");
        assert_eq!(json["line_ending"], "lf");
    }
}
