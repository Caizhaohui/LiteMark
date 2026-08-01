//! App settings persistence (M5/M6): trusted workspaces, pandoc path, custom CSS
//! path, wiki-link toggle. Stored under app-data as JSON.

use crate::error::{ErrorCode, SidecarError};
use crate::files::paths::app_data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Absolute directories the user has marked trusted (persistent confirm).
    #[serde(default)]
    pub trusted_workspaces: Vec<String>,
    /// Optional explicit Pandoc executable path.
    #[serde(default)]
    pub pandoc_path: Option<String>,
    /// Optional custom CSS file path (contents injected only after sanitize).
    #[serde(default)]
    pub custom_css_path: Option<String>,
    /// Enable wiki-link syntax in preview (default false — safer).
    #[serde(default)]
    pub enable_wiki_links: bool,
    /// Experimental: allow code-block execution design surface (always requires confirm).
    #[serde(default)]
    pub experimental_code_execution: bool,
    /// Auto-update check endpoint placeholder (M6; empty = disabled).
    #[serde(default)]
    pub update_endpoint: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            trusted_workspaces: Vec::new(),
            pandoc_path: None,
            custom_css_path: None,
            enable_wiki_links: false,
            experimental_code_execution: false,
            update_endpoint: None,
        }
    }
}

impl AppSettings {
    fn path() -> Result<PathBuf, SidecarError> {
        Ok(app_data_dir()?.join(SETTINGS_FILE))
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
        let path = dir.join(SETTINGS_FILE);
        let text = serde_json::to_string_pretty(self).map_err(|e| {
            SidecarError::new(ErrorCode::SaveAtomicReplaceFailed, format!("encode: {e}"))
        })?;
        crate::files::atomic_save::atomic_save(&path, text.as_bytes())?;
        Ok(())
    }

    pub fn is_trusted(&self, dir: &str) -> bool {
        let needle = normalize_dir(dir);
        self.trusted_workspaces
            .iter()
            .any(|w| normalize_dir(w) == needle)
    }

    pub fn trust(&mut self, dir: String) {
        let n = normalize_dir(&dir);
        if !self.trusted_workspaces.iter().any(|w| normalize_dir(w) == n) {
            self.trusted_workspaces.push(dir);
        }
    }

    pub fn untrust(&mut self, dir: &str) {
        let n = normalize_dir(dir);
        self.trusted_workspaces
            .retain(|w| normalize_dir(w) != n);
    }
}

fn normalize_dir(s: &str) -> String {
    s.trim()
        .trim_end_matches(['/', '\\'])
        .to_ascii_lowercase()
}
