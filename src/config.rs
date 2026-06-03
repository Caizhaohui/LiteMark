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
    pub fn load(dir: &Path) -> Self {
        let primary_path = dir.join(".litemark.toml");
        let legacy_path = dir.join(".stillmark.toml");

        for config_path in [&primary_path, &legacy_path] {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                return match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!(
                            "Warning: failed to parse {}: {}",
                            config_path.display(),
                            e
                        );
                        Self::default()
                    }
                };
            }
        }

        Self::default()
    }
}

impl RuntimeConfig {
    pub fn for_file(file_path: &Path, port: u16, theme: String, open_browser: bool) -> Self {
        let dir = file_path.parent().unwrap_or_else(|| Path::new("."));
        Self {
            file_config: FileConfig::load(dir),
            port,
            theme,
            open_browser,
        }
    }

    pub fn effective_theme(&self) -> &str {
        if self.theme.is_empty() {
            &self.file_config.preview.theme
        } else {
            &self.theme
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FileConfig, RuntimeConfig};
    use std::path::PathBuf;

    fn fixture_dir(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn loads_litemark_toml_from_fixture_dir() {
        let cfg = FileConfig::load(&fixture_dir("config-enabled"));
        assert_eq!(cfg.preview.theme, "github-dark");
        assert_eq!(cfg.preview.debounce_ms, 450);
        assert!(!cfg.render.math);
        assert!(!cfg.render.mermaid);
        assert!(cfg.render.highlight);
        assert!(!cfg.render.lightbox);
    }

    #[test]
    fn falls_back_to_legacy_stillmark_toml() {
        let cfg = FileConfig::load(&fixture_dir("legacy-config"));
        assert_eq!(cfg.preview.theme, "github-dark");
        assert_eq!(cfg.preview.debounce_ms, 275);
        assert!(!cfg.render.highlight);
    }

    #[test]
    fn runtime_config_prefers_cli_theme_when_present() {
        let file = fixture_dir("config-enabled").join("sample.md");
        let cfg = RuntimeConfig::for_file(&file, 0, "github-light".to_string(), true);
        assert_eq!(cfg.effective_theme(), "github-light");
    }

    #[test]
    fn runtime_config_uses_file_theme_when_cli_theme_is_empty() {
        let file = fixture_dir("config-enabled").join("sample.md");
        let cfg = RuntimeConfig::for_file(&file, 0, String::new(), true);
        assert_eq!(cfg.effective_theme(), "github-dark");
    }
}
