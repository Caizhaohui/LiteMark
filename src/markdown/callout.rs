use pulldown_cmark::{Event, Tag};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CalloutType {
    Note,
    Tip,
    Warning,
    Caution,
    Important,
    Info,
    Bug,
    Example,
    Quote,
}

/// Detect callouts in blockquotes.
/// A callout is a blockquote whose first line matches `[!TYPE]`
pub fn detect_callouts(
    input: &str,
    events: &[(Event, std::ops::Range<usize>)],
) -> HashMap<usize, CalloutType> {
    let mut map = HashMap::new();
    let mut i = 0;
    let len = events.len();

    while i < len {
        let (ref event, ref range) = events[i];
        if let Event::Start(Tag::BlockQuote(_)) = event {
            let blockquote_start = range.start;

            // Look at the next events to find text content
            let mut text_buf = String::new();
            let mut j = i + 1;
            while j < len {
                let (ref inner_event, _) = events[j];
                match inner_event {
                    Event::Text(t) => {
                        text_buf.push_str(t);
                        // Only check first line of text
                        break;
                    }
                    Event::Start(Tag::Paragraph) => {
                        j += 1;
                        continue;
                    }
                    _ => break,
                }
            }

            // Check for callout pattern: [!TYPE]
            let first_line = text_buf.lines().next().unwrap_or("");
            if let Some(callout_type) = parse_callout_type(first_line.trim()) {
                map.insert(blockquote_start, callout_type);
            }
        }
        i += 1;
    }

    map
}

fn parse_callout_type(text: &str) -> Option<CalloutType> {
    let upper = text.to_uppercase();
    if upper.starts_with("[!NOTE]") {
        Some(CalloutType::Note)
    } else if upper.starts_with("[!TIP]") {
        Some(CalloutType::Tip)
    } else if upper.starts_with("[!WARNING]") {
        Some(CalloutType::Warning)
    } else if upper.starts_with("[!CAUTION]") {
        Some(CalloutType::Caution)
    } else if upper.starts_with("[!IMPORTANT]") {
        Some(CalloutType::Important)
    } else if upper.starts_with("[!INFO]") {
        Some(CalloutType::Info)
    } else if upper.starts_with("[!BUG]") {
        Some(CalloutType::Bug)
    } else if upper.starts_with("[!EXAMPLE]") {
        Some(CalloutType::Example)
    } else if upper.starts_with("[!QUOTE]") {
        Some(CalloutType::Quote)
    } else {
        None
    }
}

/// Get CSS class, icon, and title for a callout type
pub fn callout_meta(callout: &CalloutType) -> (&'static str, &'static str, &'static str) {
    match callout {
        CalloutType::Note => ("note", "📝", "Note"),
        CalloutType::Tip => ("tip", "💡", "Tip"),
        CalloutType::Warning => ("warning", "⚠️", "Warning"),
        CalloutType::Caution => ("caution", "🔥", "Caution"),
        CalloutType::Important => ("important", "❗", "Important"),
        CalloutType::Info => ("info", "ℹ️", "Info"),
        CalloutType::Bug => ("bug", "🐛", "Bug"),
        CalloutType::Example => ("example", "📋", "Example"),
        CalloutType::Quote => ("quote", "💬", "Quote"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Options, Parser};

    fn parse_events(input: &str) -> Vec<(pulldown_cmark::Event, std::ops::Range<usize>)> {
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_HEADING_ATTRIBUTES;
        Parser::new_ext(input, options)
            .into_offset_iter()
            .collect()
    }

    #[test]
    fn test_detect_callouts_note() {
        let input = "> [!NOTE]\n> This is a note\n";
        let events = parse_events(input);
        let map = detect_callouts(input, &events);
        assert!(!map.is_empty());
        assert!(map.values().any(|ct| *ct == CalloutType::Note));
    }

    #[test]
    fn test_detect_callouts_none() {
        let input = "> Normal blockquote\n> No callout here\n";
        let events = parse_events(input);
        let map = detect_callouts(input, &events);
        assert!(map.is_empty());
    }

    #[test]
    fn test_callout_meta() {
        let (css, _, title) = callout_meta(&CalloutType::Warning);
        assert_eq!(css, "warning");
        assert_eq!(title, "Warning");
    }
}
