use std::path::Path;

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
        <article class="litemark-content" id="litemark-content">
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
        html = doc.html,
        katex_js = katex_js,
        auto_render_js = auto_render_js,
        mermaid_js = mermaid_js,
        highlight_js = highlight_js,
        export_js = export_js,
    )
}
