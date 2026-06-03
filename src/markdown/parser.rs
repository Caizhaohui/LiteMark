use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;

use super::callout;
use super::utils::html_escape;

/// Render markdown to HTML with source-line tracking
pub fn render_html(input: &str) -> String {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;

    let parser = Parser::new_ext(input, options);

    // Build byte-offset to line-number mapping
    let line_starts = compute_line_starts(input);
    let total_lines = line_starts.len();

    // Collect all events with their byte ranges
    let events_with_ranges: Vec<(Event, std::ops::Range<usize>)> = parser
        .into_offset_iter()
        .map(|(event, range)| (event, range))
        .collect();

    // Pre-process to detect callouts: scan for blockquote patterns
    let callout_map = callout::detect_callouts(input, &events_with_ranges);

    // Render HTML
    let mut html = String::with_capacity(input.len() * 2);
    let mut i = 0;
    let len = events_with_ranges.len();

    while i < len {
        let (ref event, ref range) = events_with_ranges[i];
        let line = offset_to_line(&line_starts, range.start, total_lines);

        match event {
            Event::Start(tag) => {
                let tag_html = start_tag_html(tag, line, &callout_map, range.start, input);
                html.push_str(&tag_html);

                // Special handling for code blocks
                if let Tag::CodeBlock(kind) = tag {
                    let lang = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        CodeBlockKind::Indented => String::new(),
                    };

                    // Collect code text
                    let mut code = String::new();
                    i += 1;
                    while i < len {
                        let (ref inner_event, _) = events_with_ranges[i];
                        match inner_event {
                            Event::Text(text) => {
                                code.push_str(text);
                                i += 1;
                            }
                            Event::End(TagEnd::CodeBlock) => {
                                break;
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    }

                    // Render code block
                    if lang == "mermaid" {
                        html.push_str(&format!(
                            "<pre class=\"mermaid\" data-source-line=\"{}\">{}</pre>",
                            line,
                            html_escape(&code)
                        ));
                    } else {
                        let lang_class = if lang.is_empty() {
                            String::new()
                        } else {
                            format!(" class=\"language-{}\"", html_escape(&lang))
                        };
                        html.push_str(&format!(
                            "<pre data-source-line=\"{}\"><code{}>{}</code></pre>",
                            line,
                            lang_class,
                            html_escape(&code)
                        ));
                    }

                    // Skip the End event
                    if i < len {
                        i += 1; // skip End(CodeBlock)
                    }
                    continue;
                }

                // Special handling for math: inline $ and display $$
                // These are handled as inline code by pulldown-cmark when not using math plugin
            }
            Event::End(tag_end) => {
                let close = match tag_end {
                    TagEnd::Paragraph => "</p>",
                    TagEnd::Heading(level, _, _) => &format!("</h{}>", *level as u8),
                    TagEnd::BlockQuote(_) => "</blockquote>",
                    TagEnd::CodeBlock => "", // handled above
                    TagEnd::List(ordered) => {
                        if *ordered {
                            "</ol>"
                        } else {
                            "</ul>"
                        }
                    }
                    TagEnd::Item => "</li>",
                    TagEnd::Emphasis => "</em>",
                    TagEnd::Strong => "</strong>",
                    TagEnd::Strikethrough => "</del>",
                    TagEnd::Link => "</a>",
                    TagEnd::Image => "</span>",
                    TagEnd::Table => "</table>",
                    TagEnd::TableHead => "</thead>",
                    TagEnd::TableRow => "</tr>",
                    TagEnd::TableCell => "</td>",
                    TagEnd::FootnoteDefinition(_) => "</div>",
                    TagEnd::HtmlBlock => "",
                    TagEnd::MetadataBlock(_) => "",
                    TagEnd::DefinitionList => "</dl>",
                    TagEnd::DefinitionListTitle => "</dt>",
                    TagEnd::DefinitionListDefinition => "</dd>",
                    _ => "",
                };
                html.push_str(close);
                html.push('\n');
            }
            Event::Text(text) => {
                html.push_str(&html_escape(text));
            }
            Event::Code(code) => {
                // Check if this is a math delimiter pattern
                let text = code.as_ref();
                html.push_str(&format!("<code>{}</code>", html_escape(text)));
            }
            Event::Html(html_content) => {
                html.push_str(html_content);
            }
            Event::InlineHtml(html_content) => {
                html.push_str(html_content);
            }
            Event::SoftBreak => {
                html.push('\n');
            }
            Event::HardBreak => {
                html.push_str("<br />\n");
            }
            Event::Rule => {
                html.push_str(&format!(
                    "<hr data-source-line=\"{}\" />\n",
                    line
                ));
            }
            Event::FootnoteReference(name) => {
                html.push_str(&format!(
                    "<sup class=\"footnote-ref\"><a href=\"#fn-{name}\" id=\"fnref-{name}\">{name}</a></sup>",
                    name = html_escape(name)
                ));
            }
            Event::TaskListMarker(checked) => {
                let checkbox = if *checked {
                    "<input type=\"checkbox\" checked disabled data-task-line=\"{}\">"
                } else {
                    "<input type=\"checkbox\" disabled data-task-line=\"{}\">"
                };
                html.push_str(&checkbox.replace("{}", &line.to_string()));
            }
        }
        i += 1;
    }

    html
}

/// Generate the HTML for a start tag, including data-source-line
fn start_tag_html(
    tag: &Tag,
    line: usize,
    callout_map: &HashMap<usize, callout::CalloutType>,
    byte_offset: usize,
    _input: &str,
) -> String {
    let line_attr = format!(" data-source-line=\"{}\"", line);

    match tag {
        Tag::Paragraph => {
            format!("<p{}>\n", line_attr)
        }
        Tag::Heading { level, id, .. } => {
            let id_attr = id
                .as_ref()
                .map(|id| format!(" id=\"{}\"", html_escape(id.as_ref())))
                .unwrap_or_default();
            format!("<h{}{}{}>", *level as u8, id_attr, line_attr)
        }
        Tag::BlockQuote(_) => {
            // Check if this is a callout
            if let Some(callout_type) = callout_map.get(&byte_offset) {
                let (css_class, icon, title) = callout::callout_meta(callout_type);
                format!(
                    "<blockquote{} class=\"callout callout-{}\">\n<div class=\"callout-title\"><span class=\"callout-icon\">{}</span> {}</div>\n<div class=\"callout-content\">\n",
                    line_attr, css_class, icon, title
                )
            } else {
                format!("<blockquote{}>\n", line_attr)
            }
        }
        Tag::CodeBlock(_) => {
            // Handled separately in the main loop
            String::new()
        }
        Tag::List(first_item) => {
            let tag_name = if first_item.is_some() { "ol" } else { "ul" };
            let start_attr = first_item
                .filter(|&n| *n != 1)
                .map(|n| format!(" start=\"{}\"", n))
                .unwrap_or_default();
            format!(
                "<{}{}{}>\n",
                tag_name, start_attr, line_attr
            )
        }
        Tag::Item => {
            format!("<li{}>", line_attr)
        }
        Tag::Emphasis => "<em>".to_string(),
        Tag::Strong => "<strong>".to_string(),
        Tag::Strikethrough => "<del>".to_string(),
        Tag::Link { dest_url, title, .. } => {
            let title_attr = if title.is_empty() {
                String::new()
            } else {
                format!(" title=\"{}\"", html_escape(title))
            };
            format!(
                "<a href=\"{}\"{}>",
                html_escape(dest_url),
                title_attr
            )
        }
        Tag::Image { dest_url, title, .. } => {
            let title_attr = if title.is_empty() {
                String::new()
            } else {
                format!(" title=\"{}\"", html_escape(title))
            };
            // Render image as <img> tag immediately (self-closing)
            format!(
                "<span{}><img src=\"{}\"{} alt=\"",
                line_attr,
                html_escape(dest_url),
                title_attr
            )
        }
        Tag::Table(alignments) => {
            // Store alignments for cells
            format!("<table{}>\n", line_attr)
        }
        Tag::TableHead => {
            format!("<thead{}>\n<tr>\n", line_attr)
        }
        Tag::TableRow => {
            format!("<tr{}>\n", line_attr)
        }
        Tag::TableCell => {
            format!("<td{}>", line_attr)
        }
        Tag::FootnoteDefinition(name) => {
            format!(
                "<div class=\"footnote\" id=\"fn-{}\"{}>\n<a href=\"#fnref-{}\" class=\"footnote-backref\">↩</a> ",
                html_escape(name),
                line_attr,
                html_escape(name)
            )
        }
        Tag::HtmlBlock => String::new(),
        Tag::MetadataBlock(_) => String::new(),
        Tag::DefinitionList => format!("<dl{}>\n", line_attr),
        Tag::DefinitionListTitle => format!("<dt{}>", line_attr),
        Tag::DefinitionListDefinition => format!("<dd{}>", line_attr),
        _ => String::new(),
    }
}

/// Compute line start byte offsets
fn compute_line_starts(input: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, byte) in input.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// Convert byte offset to 1-based line number
fn offset_to_line(line_starts: &[usize], offset: usize, _total_lines: usize) -> usize {
    match line_starts.binary_search(&offset) {
        Ok(idx) => idx + 1,
        Err(idx) => idx, // idx is the line after, so this is correct for 1-based
    }
}

