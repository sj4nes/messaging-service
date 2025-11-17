use unicode_segmentation::UnicodeSegmentation;

/// Build a snippet from message body without breaking grapheme clusters and
/// collapsing excessive whitespace. Returns empty string when body missing.
pub(crate) fn make_snippet(body: Option<&str>, max_graphemes: usize) -> String {
    if max_graphemes == 0 {
        return String::new();
    }
    let raw = match body {
        Some(b) => b.trim(),
        None => return String::new(),
    };
    if raw.is_empty() {
        return String::new();
    }
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return String::new();
    }
    normalized.graphemes(true).take(max_graphemes).collect()
}

#[cfg(test)]
mod tests {
    use super::make_snippet;

    #[test]
    fn truncates_ascii() {
        let s = make_snippet(Some("hello world"), 5);
        assert_eq!(s, "hello");
    }

    #[test]
    fn preserves_graphemes() {
        let s = make_snippet(Some("👍👍🏽👍🏿"), 2);
        assert_eq!(s, "👍👍🏽");
    }

    #[test]
    fn collapses_whitespace() {
        let s = make_snippet(Some("line1\n\nline2"), 16);
        assert_eq!(s, "line1 line2");
    }
}
