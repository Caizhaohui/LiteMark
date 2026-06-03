use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use super::utils::html_escape;

/// Generate a table of contents from markdown headings
pub fn generate_toc(input: &str) -> String {
    let options = Options::ENABLE_HEADING_ATTRIBUTES;
    let parser = Parser::new_ext(input, options);

    let mut toc = Vec::new();
    let mut in_heading = false;
    let mut current_level: u8 = 0;
    let mut current_text = String::new();
    let mut current_id: Option<String> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, id, .. }) => {
                in_heading = true;
                current_level = level as u8;
                current_text.clear();
                current_id = id.map(|s| s.to_string());
            }
            Event::End(TagEnd::Heading(_, _, _)) => {
                if in_heading {
                    let id = current_id.clone().unwrap_or_else(|| slugify(&current_text));
                    toc.push(TocEntry {
                        level: current_level,
                        text: current_text.clone(),
                        id,
                    });
                }
                in_heading = false;
            }
            Event::Text(text) if in_heading => {
                current_text.push_str(&text);
            }
            Event::Code(code) if in_heading => {
                current_text.push_str(&code);
            }
            _ => {}
        }
    }

    render_toc_html(&toc)
}

struct TocEntry {
    level: u8,
    text: String,
    id: String,
}

fn render_toc_html(entries: &[TocEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut html = String::from("<nav class=\"toc\">\n<ul>\n");
    let mut prev_level: u8 = 0;

    for entry in entries {
        if entry.level > prev_level {
            for _ in prev_level..entry.level {
                if prev_level > 0 {
                    html.push_str("<ul>\n");
                }
            }
        } else if entry.level < prev_level {
            for _ in entry.level..prev_level {
                html.push_str("</ul>\n");
            }
        }

        html.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>\n",
            html_escape(&entry.id),
            html_escape(&entry.text)
        ));

        prev_level = entry.level;
    }

    // Close remaining lists
    for _ in 0..prev_level {
        html.push_str("</ul>\n");
    }

    html.push_str("</nav>\n");
    html
}

/// Generate a URL-friendly slug from heading text
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_toc_basic() {
        let input = "# Heading 1\n\n## Heading 2\n\n### Heading 3\n";
        let toc = generate_toc(input);
        assert!(toc.contains("Heading 1"));
        assert!(toc.contains("Heading 2"));
        assert!(toc.contains("Heading 3"));
        assert!(toc.contains("<nav"));
    }

    #[test]
    fn test_generate_toc_empty() {
        let input = "No headings here\nJust text\n";
        let toc = generate_toc(input);
        assert!(toc.is_empty());
    }

    #[test]
    fn test_generate_toc_escaping() {
        let input = "# Heading <with> & \"special\"\n";
        let toc = generate_toc(input);
        assert!(toc.contains("&lt;with&gt;"));
        assert!(toc.contains("&amp;"));
        assert!(toc.contains("&quot;"));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Hello  World"), "hello-world");
        assert_eq!(slugify("Test!@#"), "test");
    }
}
