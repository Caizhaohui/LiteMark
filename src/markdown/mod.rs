pub mod callout;
pub mod frontmatter;
pub mod parser;
pub mod toc;
pub mod utils;

use std::path::Path;

pub struct ParsedDocument {
    pub html: String,
    pub title: String,
    pub toc_html: String,
    pub front_matter: Option<frontmatter::FrontMatter>,
}

/// Parse a markdown string into a full document
pub fn parse(input: &str) -> ParsedDocument {
    // Strip front matter
    let (front_matter, content) = frontmatter::extract_front_matter(input);

    // Generate TOC
    let toc_html = toc::generate_toc(content);

    // Parse markdown to HTML
    let html = parser::render_html(content);

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
    theme: &str,
    file_path: &Path,
) -> String {
    let doc = parse(input);
    build_preview_html(&doc, theme, file_path)
}

/// Build the full preview HTML page
fn build_preview_html(doc: &ParsedDocument, theme: &str, file_path: &Path) -> String {
    let title = &doc.title;
    let front_matter_html = doc
        .front_matter
        .as_ref()
        .map(|fm| fm.to_html())
        .unwrap_or_default();

    let filename = file_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    let toc_html = &doc.toc_html;

    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="{theme}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - LiteMark</title>
    <link rel="stylesheet" href="/assets/themes/{theme}.css">
    <link rel="stylesheet" href="/assets/vendor/katex.min.css">
    <link rel="stylesheet" href="/assets/vendor/highlight.min.css">
    <script defer src="/assets/vendor/katex.min.js"></script>
    <script defer src="/assets/vendor/auto-render.min.js"></script>
    <script defer src="/assets/vendor/mermaid.min.js"></script>
    <script defer src="/assets/vendor/highlight.min.js"></script>
    <script defer src="/assets/app.js"></script>
</head>
<body>
    <div class="litemark-container">
        <div class="litemark-header">
            <span class="litemark-filename">{filename}</span>
        </div>
        <div class="litemark-body">
            <aside class="litemark-toc">{toc_html}</aside>
            <article class="litemark-content" id="litemark-content">
                {front_matter_html}
                {html}
            </article>
        </div>
    </div>
</body>
</html>"#,
        theme = theme,
        title = title,
        filename = filename,
        toc_html = toc_html,
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
