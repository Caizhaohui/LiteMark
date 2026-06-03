use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine as _};

use crate::assets;
use crate::config::RuntimeConfig;
use crate::markdown::ParsedDocument;

pub fn export_html(doc: &ParsedDocument, config: &RuntimeConfig, source_path: &Path) -> String {
    let render = &config.file_config.render;
    let theme = config.effective_theme();
    let title = &doc.title;

    let theme_css = assets::get_asset_text(&format!("themes/{}.css", theme))
        .unwrap_or_else(|| include_str!("../../assets/themes/github-light.css").to_string());

    let katex_css = if render.math {
        assets::get_asset_text("vendor/katex.min.css").unwrap_or_default()
    } else {
        String::new()
    };

    let highlight_css = if render.highlight {
        assets::get_asset_text("vendor/highlight.min.css").unwrap_or_default()
    } else {
        String::new()
    };

    let katex_js = if render.math {
        assets::get_asset_text("vendor/katex.min.js").unwrap_or_default()
    } else {
        String::new()
    };
    let auto_render_js = if render.math {
        assets::get_asset_text("vendor/auto-render.min.js").unwrap_or_default()
    } else {
        String::new()
    };
    let mermaid_js = if render.mermaid {
        assets::get_asset_text("vendor/mermaid.min.js").unwrap_or_default()
    } else {
        String::new()
    };
    let highlight_js = if render.highlight {
        assets::get_asset_text("vendor/highlight.min.js").unwrap_or_default()
    } else {
        String::new()
    };

    let front_matter_html = doc
        .front_matter
        .as_ref()
        .map(|fm| fm.to_html())
        .unwrap_or_default();
    let body_html = if config.file_config.export.embed_images {
        embed_local_images(&doc.html, source_path)
    } else {
        doc.html.clone()
    };

    let export_js = r#"
        document.addEventListener('DOMContentLoaded', function() {
            if (typeof renderMathInElement !== 'undefined') {
                renderMathInElement(document.body, {
                    delimiters: [
                        {left: '$$', right: '$$', display: true},
                        {left: '$', right: '$', display: false},
                        {left: '\\(', right: '\\)', display: false},
                        {left: '\\[', right: '\\]', display: true}
                    ],
                    throwOnError: false
                });
            }
            if (typeof mermaid !== 'undefined') {
                mermaid.initialize({startOnLoad: true, theme: 'default'});
            }
            if (typeof hljs !== 'undefined') {
                hljs.highlightAll();
            }
        });
    "#;

    let filename = source_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="{theme}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
{theme_css}
{katex_css}
{highlight_css}
    </style>
</head>
<body>
    <div class="litemark-container">
        <div class="litemark-header">
            <span class="litemark-filename">{filename}</span>
            <span class="litemark-export-badge">Exported by LiteMark</span>
        </div>
        <article class="litemark-content" id="preview-content">
            {front_matter_html}
            {html}
        </article>
    </div>
    <script>{katex_js}</script>
    <script>{auto_render_js}</script>
    <script>{mermaid_js}</script>
    <script>{highlight_js}</script>
    <script>{export_js}</script>
</body>
</html>"#,
        theme = theme,
        title = title,
        filename = filename,
        theme_css = theme_css,
        katex_css = katex_css,
        highlight_css = highlight_css,
        front_matter_html = front_matter_html,
        html = body_html,
        katex_js = katex_js,
        auto_render_js = auto_render_js,
        mermaid_js = mermaid_js,
        highlight_js = highlight_js,
        export_js = export_js,
    )
}

fn embed_local_images(html: &str, source_path: &Path) -> String {
    let mut result = html.to_string();
    let base_dir = source_path.parent().unwrap_or_else(|| Path::new("."));
    let mut search_from = 0usize;

    while let Some(img_pos_rel) = result[search_from..].find("<img src=\"") {
        let img_pos = search_from + img_pos_rel;
        let src_start = img_pos + "<img src=\"".len();
        let Some(src_end_rel) = result[src_start..].find('"') else {
            break;
        };
        let src_end = src_start + src_end_rel;
        let src = result[src_start..src_end].to_string();

        if src.starts_with("http://")
            || src.starts_with("https://")
            || src.starts_with("data:")
            || src.starts_with("file:")
        {
            search_from = src_end;
            continue;
        }

        let image_path = base_dir.join(&src);
        let Ok(bytes) = std::fs::read(&image_path) else {
            search_from = src_end;
            continue;
        };
        let mime = match image_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref()
        {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("webp") => "image/webp",
            _ => {
                search_from = src_end;
                continue;
            }
        };

        let data_uri = format!("data:{};base64,{}", mime, STANDARD.encode(bytes));
        result.replace_range(src_start..src_end, &data_uri);
        search_from = src_start + data_uri.len();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::export_html;
    use crate::config::{FileConfig, RuntimeConfig};
    use crate::markdown::parse_with_render_config;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_fixture_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("litemark-{name}-{unique}"));
        fs::create_dir_all(&dir).expect("create temp fixture dir");
        dir
    }

    fn base_config() -> RuntimeConfig {
        RuntimeConfig {
            file_config: FileConfig::default(),
            port: 0,
            theme: String::new(),
            open_browser: false,
        }
    }

    #[test]
    fn export_html_omits_disabled_runtime_assets() {
        let mut config = base_config();
        config.file_config.render.math = false;
        config.file_config.render.mermaid = false;
        config.file_config.render.highlight = false;

        let doc = parse_with_render_config("# Title\n\nText\n", &config.file_config.render);
        let out = export_html(&doc, &config, PathBuf::from("demo.md").as_path());

        assert!(!out.contains("katex.min.js"));
        assert!(!out.contains("auto-render.min.js"));
        assert!(!out.contains("mermaid.min.js"));
        assert!(!out.contains("highlight.min.js"));
        assert!(!out.contains("katex.min.css"));
        assert!(!out.contains("highlight.min.css"));
    }

    #[test]
    fn export_html_embeds_local_images_when_enabled() {
        let dir = temp_fixture_dir("embed-images");
        let md_path = dir.join("note.md");
        let img_path = dir.join("tiny.png");

        // 1x1 transparent PNG
        let png_bytes: &[u8] = &[
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
            0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137,
            0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 255, 255, 63,
            0, 5, 254, 2, 254, 167, 53, 129, 132, 0, 0, 0, 0, 73, 69,
            78, 68, 174, 66, 96, 130,
        ];

        fs::write(&img_path, png_bytes).expect("write png fixture");
        fs::write(&md_path, "![Tiny](tiny.png)\n").expect("write markdown fixture");

        let mut config = base_config();
        config.file_config.export.embed_images = true;
        let doc = parse_with_render_config("![Tiny](tiny.png)\n", &config.file_config.render);
        let out = export_html(&doc, &config, &md_path);

        assert!(out.contains("src=\"data:image/png;base64,"));
        assert!(!out.contains("src=\"tiny.png\""));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_html_preserves_image_paths_when_embedding_disabled() {
        let mut config = base_config();
        config.file_config.export.embed_images = false;

        let doc = parse_with_render_config("![Tiny](tiny.png)\n", &config.file_config.render);
        let out = export_html(&doc, &config, PathBuf::from("note.md").as_path());

        assert!(out.contains("src=\"tiny.png\""));
        assert!(!out.contains("src=\"data:image/png;base64,"));
    }

    #[test]
    fn export_html_preserves_remote_and_data_image_urls_when_embedding_enabled() {
        let mut config = base_config();
        config.file_config.export.embed_images = true;

        let doc = parse_with_render_config(
            "![Remote](https://example.com/a.png)\n![Inline](data:image/png;base64,abcd)\n",
            &config.file_config.render,
        );
        let out = export_html(&doc, &config, PathBuf::from("note.md").as_path());

        assert!(out.contains("src=\"https://example.com/a.png\""));
        assert!(out.contains("src=\"data:image/png;base64,abcd\""));
    }

    #[test]
    fn export_html_preserves_missing_and_non_image_local_paths() {
        let dir = temp_fixture_dir("missing-non-image");
        let md_path = dir.join("note.md");
        let txt_path = dir.join("not-image.txt");

        fs::write(&txt_path, "plain text").expect("write non-image fixture");
        fs::write(&md_path, "![Missing](missing.png)\n![Text](not-image.txt)\n")
            .expect("write markdown fixture");

        let mut config = base_config();
        config.file_config.export.embed_images = true;
        let doc = parse_with_render_config(
            "![Missing](missing.png)\n![Text](not-image.txt)\n",
            &config.file_config.render,
        );
        let out = export_html(&doc, &config, &md_path);

        assert!(out.contains("src=\"missing.png\""));
        assert!(out.contains("src=\"not-image.txt\""));
        assert!(!out.contains("data:image/"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn runtime_config_from_file_drives_export_output() {
        let dir = temp_fixture_dir("config-driven-export");
        let md_path = dir.join("note.md");
        let cfg_path = dir.join(".litemark.toml");
        let img_path = dir.join("tiny.png");

        let png_bytes: &[u8] = &[
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
            0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137,
            0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 255, 255, 63,
            0, 5, 254, 2, 254, 167, 53, 129, 132, 0, 0, 0, 0, 73, 69,
            78, 68, 174, 66, 96, 130,
        ];

        fs::write(
            &cfg_path,
            "[preview]\ntheme = \"github-dark\"\n\n[render]\nmath = false\nmermaid = false\nhighlight = false\ncallout = false\nemoji = false\nlightbox = false\n\n[export]\nembed_images = true\n",
        )
        .expect("write config fixture");
        fs::write(&img_path, png_bytes).expect("write png fixture");
        fs::write(&md_path, "# Title\n\n:rocket:\n\n> [!NOTE]\n> Hidden\n\n![Tiny](tiny.png)\n")
            .expect("write markdown fixture");

        let runtime = RuntimeConfig::for_file(&md_path, 0, String::new(), false);
        let markdown = fs::read_to_string(&md_path).expect("read markdown fixture");
        let doc = parse_with_render_config(&markdown, &runtime.file_config.render);
        let out = export_html(&doc, &runtime, &md_path);

        assert!(out.contains("data-theme=\"github-dark\""));
        assert!(out.contains("src=\"data:image/png;base64,"));
        assert!(out.contains(":rocket:"));
        assert!(!out.contains("class=\"callout"));
        assert!(!out.contains("katex.min.js"));
        assert!(!out.contains("mermaid.min.js"));
        assert!(!out.contains("highlight.min.js"));

        let _ = fs::remove_dir_all(&dir);
    }
}
