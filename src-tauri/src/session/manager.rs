//! `SessionManager`: Tauri-managed singleton holding all in-memory
//! [`DocumentSession`]s plus the active-tab id. Stored as Tauri state so
//! commands share one set of sessions. Concurrency is mediated by an async
//! mutex (sessions are mutated on the command's task).

use crate::error::{ErrorCode, SidecarError};
use crate::files::atomic_save::atomic_save;
use crate::files::encoding;
use crate::files::paths;
use crate::files::read_and_decode;
use crate::session::model::DocumentSession;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::async_runtime::Mutex;
use uuid::Uuid;

/// Cap how much content we read at once for the M1 temporary editor, to avoid
/// OOM on a pathologically large file. M2 has dedicated large-file degradation
/// for the preview; this is just a safety valve for M1.
const MAX_READ_BYTES: usize = 64 * 1024 * 1024; // 64 MiB

pub struct SessionManager {
    inner: Mutex<SessionManagerInner>,
}

struct SessionManagerInner {
    sessions: HashMap<String, DocumentSession>,
    /// Order of tabs (oldest first); the last element is the rightmost tab.
    order: Vec<String>,
    active_id: Option<String>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self {
            inner: Mutex::new(SessionManagerInner {
                sessions: HashMap::new(),
                order: Vec::new(),
                active_id: None,
            }),
        }
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a brand-new empty document and make it active. Returns its id.
    pub async fn new_document(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let session = DocumentSession::new_empty(id.clone());
        let mut g = self.inner.lock().await;
        g.sessions.insert(id.clone(), session);
        g.order.push(id.clone());
        g.active_id = Some(id.clone());
        id
    }

    /// Open a file from disk into a new session. Errors surface structured
    /// codes (FileNotFound / FilePermissionDenied / FileEncodingUnsupported).
    pub async fn open_file(&self, path: &Path) -> Result<String, SidecarError> {
        // Size guard before reading.
        let normalized = paths::normalize_long_path(path)?;
        let meta = std::fs::metadata(&normalized).map_err(|e| {
            let code = if e.kind() == std::io::ErrorKind::NotFound {
                ErrorCode::FileNotFound
            } else {
                ErrorCode::FilePermissionDenied
            };
            SidecarError::new(code, format!("stat {}: {e}", normalized.display()))
        })?;
        let len = meta.len() as usize;
        if len > MAX_READ_BYTES {
            return Err(SidecarError::new(
                ErrorCode::FileChangedExternally,
                format!(
                    "file is {} bytes which exceeds the M1 read limit of {}; open a smaller file for now",
                    len, MAX_READ_BYTES
                ),
            ));
        }

        let mtime_ms = crate::files::atomic_save::mtime_millis(&normalized);
        let read_only = meta.permissions().readonly();
        let decoded = tokio::task::spawn_blocking({
            let path = path.to_path_buf();
            move || read_and_decode(&path)
        })
        .await
        .map_err(|e| SidecarError::new(ErrorCode::FileNotFound, format!("join: {e}")))??;

        let id = Uuid::new_v4().to_string();
        let display_name = paths::display_name(Some(path));
        let session = DocumentSession::from_file(
            id.clone(),
            path.to_path_buf(),
            display_name,
            decoded,
            mtime_ms,
            read_only,
        );
        let mut g = self.inner.lock().await;
        g.sessions.insert(id.clone(), session);
        g.order.push(id.clone());
        g.active_id = Some(id.clone());
        Ok(id)
    }

    /// Save a session to its current file path (atomic save). Returns the new
    /// mtime ms and content hash.
    pub async fn save(&self, session_id: &str) -> Result<(i64, String), SidecarError> {
        let snapshot = {
            let g = self.inner.lock().await;
            let s = g
                .sessions
                .get(session_id)
                .ok_or_else(|| SidecarError::new(ErrorCode::FileNotFound, "no such session"))?
                .clone();
            s
        };
        if snapshot.read_only {
            return Err(SidecarError::new(
                ErrorCode::FilePermissionDenied,
                "session is read-only; use Save As to a writable location",
            ));
        }
        let path = snapshot.file_path.clone().ok_or_else(|| {
            SidecarError::new(
                ErrorCode::PathNotAuthorized,
                "session has no file path; use Save As",
            )
        })?;
        let bytes = encoding::encode(&snapshot.content, snapshot.encoding, snapshot.line_ending);
        let outcome = tokio::task::spawn_blocking(move || atomic_save(&path, &bytes))
            .await
            .map_err(|e| {
                SidecarError::new(ErrorCode::SaveAtomicReplaceFailed, format!("join: {e}"))
            })??;
        let mut g = self.inner.lock().await;
        if let Some(s) = g.sessions.get_mut(session_id) {
            s.mark_saved(outcome.content_hash.clone(), outcome.mtime_ms, None);
        }
        Ok((outcome.mtime_ms, outcome.content_hash))
    }

    /// Save a session to a new path (Save As). Updates the session's path,
    /// display name, encoding (default UTF-8 / detected), and dirty state.
    pub async fn save_as(
        &self,
        session_id: &str,
        path: &Path,
    ) -> Result<(i64, String), SidecarError> {
        let snapshot = {
            let g = self.inner.lock().await;
            g.sessions
                .get(session_id)
                .ok_or_else(|| SidecarError::new(ErrorCode::FileNotFound, "no such session"))?
                .clone()
        };
        let bytes = encoding::encode(&snapshot.content, snapshot.encoding, snapshot.line_ending);
        let save_path = path.to_path_buf();
        let outcome = tokio::task::spawn_blocking(move || atomic_save(&save_path, &bytes))
            .await
            .map_err(|e| {
                SidecarError::new(ErrorCode::SaveAtomicReplaceFailed, format!("join: {e}"))
            })??;
        let mut g = self.inner.lock().await;
        if let Some(s) = g.sessions.get_mut(session_id) {
            // Recompute recovery key for the new path so future snapshots file
            // under the new identity.
            let new_key = path
                .to_str()
                .map(|st| {
                    format!(
                        "file-{}",
                        crate::files::atomic_save::content_hash(st.as_bytes())
                    )
                })
                .unwrap_or_else(|| format!("new-{}", s.id));
            s.recovery_key = new_key;
            s.read_only = false;
            s.mark_saved(
                outcome.content_hash.clone(),
                outcome.mtime_ms,
                Some(path.to_path_buf()),
            );
        }
        Ok((outcome.mtime_ms, outcome.content_hash))
    }

    /// Apply an edit from the editor.
    pub async fn set_content(&self, session_id: &str, content: String) -> Result<(), SidecarError> {
        let mut g = self.inner.lock().await;
        let s = g
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| SidecarError::new(ErrorCode::FileNotFound, "no such session"))?;
        s.set_content(content);
        Ok(())
    }

    /// Snapshot a session for serialization to the frontend.
    pub async fn get(&self, session_id: &str) -> Result<DocumentSession, SidecarError> {
        let mut g = self.inner.lock().await;
        let s = g
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| SidecarError::new(ErrorCode::FileNotFound, "no such session"))?;
        s.recompute_dirty();
        Ok(s.clone())
    }

    /// List all sessions in tab order, recomputing dirty.
    pub async fn list(&self) -> Vec<DocumentSession> {
        let mut g = self.inner.lock().await;
        for s in g.sessions.values_mut() {
            s.recompute_dirty();
        }
        g.order
            .iter()
            .filter_map(|id| g.sessions.get(id).cloned())
            .collect()
    }

    /// Sessions whose content differs from the saved hash.
    pub async fn list_dirty(&self) -> Vec<String> {
        let mut g = self.inner.lock().await;
        let mut out = Vec::new();
        for s in g.sessions.values_mut() {
            s.recompute_dirty();
            if s.dirty {
                out.push(s.id.clone());
            }
        }
        out
    }

    /// Close a session. Returns true if it was dirty *before* removal (so the
    /// caller can prompt before calling this).
    pub async fn close(&self, session_id: &str) -> Result<bool, SidecarError> {
        let mut g = self.inner.lock().await;
        let s = g
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| SidecarError::new(ErrorCode::FileNotFound, "no such session"))?;
        s.recompute_dirty();
        let was_dirty = s.dirty;
        g.sessions.remove(session_id);
        g.order.retain(|id| id != session_id);
        if g.active_id.as_deref() == Some(session_id) {
            g.active_id = g.order.last().cloned();
        }
        Ok(was_dirty)
    }

    pub async fn set_active(&self, session_id: Option<&str>) {
        let mut g = self.inner.lock().await;
        g.active_id = session_id.map(|s| s.to_string());
    }

    pub async fn active_id(&self) -> Option<String> {
        self.inner.lock().await.active_id.clone()
    }

    /// Check whether the on-disk file for a session has changed since open/save
    /// (by mtime). Returns true if an external change is suspected. A missing
    /// file is treated as "not externally changed" (deletion is a separate UX).
    pub async fn check_external_change(&self, session_id: &str) -> Result<bool, SidecarError> {
        let g = self.inner.lock().await;
        let s = g
            .sessions
            .get(session_id)
            .ok_or_else(|| SidecarError::new(ErrorCode::FileNotFound, "no such session"))?;
        let Some(path) = &s.file_path else {
            return Ok(false);
        };
        let Some(known_mtime) = s.external_mtime_ms else {
            return Ok(false);
        };
        let normalized = paths::normalize_long_path(path)?;
        match crate::files::atomic_save::mtime_millis(&normalized) {
            Some(current) => Ok(current != known_mtime),
            None => Ok(false),
        }
    }

    /// Take an immutable snapshot for recovery writing (does not recompute).
    pub async fn snapshot_for_recovery(&self, session_id: &str) -> Option<RecoverySnapshot> {
        let g = self.inner.lock().await;
        let s = g.sessions.get(session_id)?;
        Some(RecoverySnapshot {
            session_id: s.id.clone(),
            original_path: s.file_path.clone(),
            content: s.content.clone(),
            revision: s.revision,
            recovery_key: s.recovery_key.clone(),
        })
    }
}

/// A minimal snapshot used by the recovery store.
pub struct RecoverySnapshot {
    pub session_id: String,
    pub original_path: Option<PathBuf>,
    pub content: String,
    pub revision: u64,
    pub recovery_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn new_open_save_saveas_close_lifecycle() {
        let mgr = SessionManager::new();

        // new empty
        let id = mgr.new_document().await;
        assert_eq!(mgr.active_id().await.as_deref(), Some(id.as_str()));
        assert_eq!(mgr.list().await.len(), 1);

        // edit
        mgr.set_content(&id, "# Hello\n".into()).await.unwrap();
        let s = mgr.get(&id).await.unwrap();
        assert!(s.dirty);

        // save as to a temp file
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("doc.md");
        let (mtime, hash) = mgr.save_as(&id, &path).await.unwrap();
        assert!(mtime > 0);
        assert!(!hash.is_empty());
        let s = mgr.get(&id).await.unwrap();
        assert!(!s.dirty);
        assert_eq!(s.file_path.as_deref(), Some(path.as_path()));
        assert_eq!(
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read: {e}")),
            "# Hello\n"
        );

        // edit + save (to existing path)
        mgr.set_content(&id, "# Hello world\n".into())
            .await
            .unwrap();
        mgr.save(&id).await.unwrap();
        let s = mgr.get(&id).await.unwrap();
        assert!(!s.dirty);
        assert_eq!(fs::read_to_string(&path).unwrap(), "# Hello world\n");

        // close (was clean now) -> not dirty
        let was_dirty = mgr.close(&id).await.unwrap();
        assert!(!was_dirty);
        assert!(mgr.list().await.is_empty());
    }

    #[tokio::test]
    async fn open_file_reads_and_decodes() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("existing.md");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(encoding::UTF8_BOM);
        bytes.extend_from_slice(b"title\r\nbody\r\n");
        fs::write(&path, &bytes).unwrap();

        let mgr = SessionManager::new();
        let id = mgr.open_file(&path).await.unwrap();
        let s = mgr.get(&id).await.unwrap();
        assert_eq!(s.encoding, encoding::Encoding::Utf8Bom);
        assert_eq!(s.line_ending, encoding::LineEnding::Crlf);
        assert_eq!(s.content, "title\nbody\n");
        assert!(!s.dirty);
        assert!(s.external_mtime_ms.is_some());
    }

    #[tokio::test]
    async fn open_missing_file_errors() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nope.md");
        let mgr = SessionManager::new();
        let err = mgr.open_file(&path).await.unwrap_err();
        assert_eq!(err.code, ErrorCode::FileNotFound);
    }

    #[tokio::test]
    async fn save_without_path_requires_save_as() {
        let mgr = SessionManager::new();
        let id = mgr.new_document().await;
        mgr.set_content(&id, "x\n".into()).await.unwrap();
        let err = mgr.save(&id).await.unwrap_err();
        assert_eq!(err.code, ErrorCode::PathNotAuthorized);
    }

    #[tokio::test]
    async fn external_change_detected_after_mtime_change() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ext.md");
        fs::write(&path, "v1\n").unwrap();
        let mgr = SessionManager::new();
        let id = mgr.open_file(&path).await.unwrap();
        assert!(!mgr.check_external_change(&id).await.unwrap());

        // Simulate external modification with a clearly different mtime.
        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(&path, "v2\n").unwrap();
        assert!(mgr.check_external_change(&id).await.unwrap());
    }

    #[tokio::test]
    async fn close_removes_session_and_updates_active() {
        let mgr = SessionManager::new();
        let a = mgr.new_document().await;
        let b = mgr.new_document().await;
        assert_eq!(mgr.active_id().await.as_deref(), Some(b.as_str()));
        mgr.close(&b).await.unwrap();
        // active falls back to the previous tab
        assert_eq!(mgr.active_id().await.as_deref(), Some(a.as_str()));
    }
}
