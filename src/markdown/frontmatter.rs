use serde::Deserialize;
use std::collections::HashMap;

use super::utils::html_escape;

#[derive(Debug, Clone, Deserialize)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

impl FrontMatter {
    pub fn to_html(&self) -> String {
        let mut html = String::from("<div class=\"front-matter\">\n");

        if let Some(ref title) = self.title {
            html.push_str(&format!(
                "  <div class=\"fm-row\"><span class=\"fm-key\">Title</span><span class=\"fm-val\">{}</span></div>\n",
                html_escape(title)
            ));
        }
        if let Some(ref author) = self.author {
            html.push_str(&format!(
                "  <div class=\"fm-row\"><span class=\"fm-key\">Author</span><span class=\"fm-val\">{}</span></div>\n",
                html_escape(author)
            ));
        }
        if let Some(ref date) = self.date {
            html.push_str(&format!(
                "  <div class=\"fm-row\"><span class=\"fm-key\">Date</span><span class=\"fm-val\">{}</span></div>\n",
                html_escape(date)
            ));
        }
        if let Some(ref desc) = self.description {
            html.push_str(&format!(
                "  <div class=\"fm-row\"><span class=\"fm-key\">Description</span><span class=\"fm-val\">{}</span></div>\n",
                html_escape(desc)
            ));
        }
        if let Some(ref tags) = self.tags {
            let tag_html: Vec<String> = tags
                .iter()
                .map(|t| format!("<span class=\"fm-tag\">{}</span>", html_escape(t)))
                .collect();
            html.push_str(&format!(
                "  <div class=\"fm-row\"><span class=\"fm-key\">Tags</span><span class=\"fm-val\">{}</span></div>\n",
                tag_html.join(" ")
            ));
        }

        html.push_str("</div>\n");
        html
    }
}

/// Extract YAML front matter from markdown.
/// Returns (front_matter, remaining_content).
pub fn extract_front_matter(input: &str) -> (Option<FrontMatter>, &str) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return (None, input);
    }

    // Find the closing ---
    let after_open = &trimmed[3..];
    // Skip the rest of the opening line (after ---)
    let after_newline = after_open.find('\n').map(|i| &after_open[i + 1..]);

    if let Some(content_start) = after_newline {
        if let Some(end_pos) = content_start.find("\n---") {
            let yaml_str = &content_start[..end_pos];
            let remaining = &content_start[end_pos + 4..];
            // Skip newline after closing ---
            let remaining = remaining.strip_prefix('\n').unwrap_or(remaining);

            match serde_yaml::from_str::<FrontMatter>(yaml_str) {
                Ok(fm) => (Some(fm), remaining),
                Err(e) => {
                    eprintln!("Warning: failed to parse front matter: {}", e);
                    (None, input)
                }
            }
        } else {
            (None, input)
        }
    } else {
        (None, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_front_matter_basic() {
        let input = "---\ntitle: Hello\nauthor: Test\n---\nContent here\n";
        let (fm, remaining) = extract_front_matter(input);
        let fm = fm.unwrap();
        assert_eq!(fm.title.as_deref(), Some("Hello"));
        assert_eq!(fm.author.as_deref(), Some("Test"));
        assert_eq!(remaining, "Content here\n");
    }

    #[test]
    fn test_extract_front_matter_none() {
        let input = "# Hello\nNo front matter here\n";
        let (fm, remaining) = extract_front_matter(input);
        assert!(fm.is_none());
        assert_eq!(remaining, input);
    }

    #[test]
    fn test_front_matter_to_html() {
        let fm = FrontMatter {
            title: Some("Test Title".to_string()),
            author: Some("Author <&>".to_string()),
            date: None,
            description: None,
            tags: Some(vec!["rust".to_string(), "md".to_string()]),
            extra: HashMap::new(),
        };
        let html = fm.to_html();
        assert!(html.contains("Test Title"));
        assert!(html.contains("Author &lt;&amp;&gt;"));
        assert!(html.contains("rust"));
        assert!(html.contains("md"));
    }

    #[test]
    fn test_extract_front_matter_unclosed() {
        let input = "---\ntitle: Hello\nNo closing\n";
        let (fm, _) = extract_front_matter(input);
        assert!(fm.is_none());
    }
}
