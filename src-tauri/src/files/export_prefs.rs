//! Last-used export directory (M3). Persisted under app-data so the save
//! dialog can open in a sensible place without giving the webview any fs
//! permission.

use crate::error::{ErrorCode, SidecarError};
use crate::files::paths::app_data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const PREFS_FILE: &str = "export-prefs.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportPrefs {
    /// Absolute directory last used for HTML/PDF export.
    #[serde(default)]
    pub last_export_dir: Option<String>,
}

impl ExportPrefs {
    fn path() -> Result<PathBuf, SidecarError> {
        Ok(app_data_dir()?.join(PREFS_FILE))
    }

    pub fn load() -> Self {
        let Ok(path) = Self::path() else {
            return Self::default();
        };
        if !path.exists() {
            return Self::default();
        }
        let Ok(text) = fs::read_to_string(&path) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), SidecarError> {
        let dir = app_data_dir()?;
        fs::create_dir_all(&dir).map_err(|e| {
            SidecarError::new(
                ErrorCode::FilePermissionDenied,
                format!("could not create app-data dir: {e}"),
            )
        })?;
        let path = dir.join(PREFS_FILE);
        let text = serde_json::to_string_pretty(self).map_err(|e| {
            SidecarError::new(ErrorCode::SaveAtomicReplaceFailed, format!("encode: {e}"))
        })?;
        crate::files::atomic_save::atomic_save(&path, text.as_bytes())?;
        Ok(())
    }

    pub fn set_last_export_dir(dir: &Path) -> Result<(), SidecarError> {
        let mut prefs = Self::load();
        prefs.last_export_dir = Some(dir.to_string_lossy().into_owned());
        prefs.save()
    }

    pub fn last_export_dir() -> Option<PathBuf> {
        Self::load()
            .last_export_dir
            .map(PathBuf::from)
            .filter(|p| p.is_dir())
    }
}
