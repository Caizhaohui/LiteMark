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

    for (event, range) in events {
        if let Event::Start(Tag::BlockQuote(_)) = event {
            let first_line = input
                .get(range.start..)
                .and_then(|rest| rest.lines().next())
                .unwrap_or("")
                .trim_start_matches('>')
                .trim();

            if let Some(callout_type) = parse_callout_type(first_line) {
                map.insert(range.start, callout_type);
            }
        }
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

    fn parse_events(input: &str) -> Vec<(pulldown_cmark::Event<'_>, std::ops::Range<usize>)> {
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
