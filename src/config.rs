use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    pub preview: PreviewConfig,
    pub render: RenderConfig,
    pub export: ExportConfig,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub file_config: FileConfig,
    pub port: u16,
    pub theme: String,
    pub open_browser: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PreviewConfig {
    pub theme: String,
    pub scroll_sync: bool,
    pub debounce_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    pub math: bool,
    pub mermaid: bool,
    pub highlight: bool,
    pub callout: bool,
    pub emoji: bool,
    pub lightbox: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportConfig {
    pub embed_images: bool,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            preview: PreviewConfig::default(),
            render: RenderConfig::default(),
            export: ExportConfig::default(),
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            file_config: FileConfig::default(),
            port: 0,
            theme: "github-light".to_string(),
            open_browser: true,
        }
    }
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            theme: "github-light".to_string(),
            scroll_sync: true,
            debounce_ms: 200,
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            math: true,
            mermaid: true,
            highlight: true,
            callout: true,
            emoji: true,
            lightbox: true,
        }
    }
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            embed_images: true,
        }
    }
}

impl FileConfig {
    /// Load .litemark.toml by searching the given dir and its ancestors (like git config).
    pub fn load(start: &Path) -> Self {
        let mut dir = if start.is_file() {
            start.parent().unwrap_or(start).to_path_buf()
        } else {
            start.to_path_buf()
        };

        loop {
            let config_path = dir.join(".litemark.toml");
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!(
                            "Warning: failed to parse {}: {}",
                            config_path.display(),
                            e
                        );
                        return Self::default();
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
        Self::default()
    }
}

impl RuntimeConfig {
    pub fn effective_theme(&self) -> &str {
        if self.theme.is_empty() {
            &self.file_config.preview.theme
        } else {
            &self.theme
        }
    }
}
