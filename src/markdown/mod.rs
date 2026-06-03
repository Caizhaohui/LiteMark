pub mod callout;
pub mod frontmatter;
pub mod parser;
pub mod toc;
pub mod utils;

use std::path::Path;

use crate::config::{RenderConfig, RuntimeConfig};

pub struct ParsedDocument {
    pub html: String,
    pub title: String,
    pub toc_html: String,
    pub front_matter: Option<frontmatter::FrontMatter>,
}

pub fn parse_with_render_config(input: &str, render_config: &RenderConfig) -> ParsedDocument {
    // Strip front matter
    let (front_matter, content) = frontmatter::extract_front_matter(input);

    // Generate TOC
    let toc_html = toc::generate_toc(content);

    // Parse markdown to HTML
    let html = parser::render_html(content, render_config);

    // Extract title from first heading or front matter
    let title = front_matter
        .as_ref()
        .and_then(|fm| fm.title.clone())
        .unwrap_or_else(|| extract_title_from_html(&html));

    ParsedDocument {
        html,
        title,
        toc_html,
        front_matter,
    }
}

/// Parse and render for preview, wrapping with full HTML template
pub fn render_for_preview(
    input: &str,
    config: &RuntimeConfig,
    file_path: &Path,
) -> String {
    let doc = parse_with_render_config(input, &config.file_config.render);
    build_preview_html(&doc, config, file_path)
}

/// Build the full preview HTML page
fn build_preview_html(doc: &ParsedDocument, config: &RuntimeConfig, file_path: &Path) -> String {
    let theme = config.effective_theme();
    let render = &config.file_config.render;
    let title = &doc.title;
    let front_matter_html = doc
        .front_matter
        .as_ref()
        .map(|fm| fm.to_html())
        .unwrap_or_default();
    let katex_css = if render.math {
        "<link rel=\"stylesheet\" href=\"/assets/vendor/katex.min.css\">"
    } else {
        ""
    };
    let highlight_css = if render.highlight {
        "<link rel=\"stylesheet\" href=\"/assets/vendor/highlight.min.css\">"
    } else {
        ""
    };
    let katex_js = if render.math {
        "<script defer src=\"/assets/vendor/katex.min.js\"></script>\n    <script defer src=\"/assets/vendor/auto-render.min.js\"></script>"
    } else {
        ""
    };
    let mermaid_js = if render.mermaid {
        "<script defer src=\"/assets/vendor/mermaid.min.js\"></script>"
    } else {
        ""
    };
    let highlight_js = if render.highlight {
        "<script defer src=\"/assets/vendor/highlight.min.js\"></script>"
    } else {
        ""
    };

    let filename = file_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="{theme}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - LiteMark</title>
    <link rel="stylesheet" href="/assets/themes/{theme}.css">
    {katex_css}
    {highlight_css}
    {katex_js}
    {mermaid_js}
    {highlight_js}
    <script defer src="/assets/app.js"></script>
</head>
<body
    data-render-math="{render_math}"
    data-render-mermaid="{render_mermaid}"
    data-render-highlight="{render_highlight}"
    data-render-lightbox="{render_lightbox}"
    data-scroll-sync="{scroll_sync}"
>
    <div class="litemark-container">
        <div class="litemark-header">
            <span class="litemark-filename">{filename}</span>
        </div>
        <article class="litemark-content" id="preview-content">
            {front_matter_html}
            {html}
        </article>
    </div>
</body>
</html>"#,
        theme = theme,
        title = title,
        filename = filename,
        katex_css = katex_css,
        highlight_css = highlight_css,
        katex_js = katex_js,
        mermaid_js = mermaid_js,
        highlight_js = highlight_js,
        render_math = render.math,
        render_mermaid = render.mermaid,
        render_highlight = render.highlight,
        render_lightbox = render.lightbox,
        scroll_sync = config.file_config.preview.scroll_sync,
        front_matter_html = front_matter_html,
        html = doc.html,
    )
}

fn extract_title_from_html(html: &str) -> String {
    // Simple extraction: find first <h1> tag
    if let Some(start) = html.find("<h1") {
        if let Some(gt) = html[start..].find('>') {
            let after_tag = &html[start + gt + 1..];
            if let Some(end) = after_tag.find("</h1>") {
                let raw = &after_tag[..end];
                // Strip inner HTML tags
                let stripped = strip_tags(raw);
                return stripped.trim().to_string();
            }
        }
    }
    "Untitled".to_string()
}

fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}
