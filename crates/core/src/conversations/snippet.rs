use unicode_segmentation::UnicodeSegmentation;

/// Build a snippet from message body without breaking grapheme clusters (Unicode-safe).
/// Collapses excessive whitespace and trims the result.
///
/// # Arguments
/// * `body` - Optional message body text
/// * `max_chars` - Maximum number of characters (grapheme clusters) to include
///
/// # Returns
/// * Truncated string with proper UTF-8 boundaries, or empty string if body is None/empty
pub fn make_snippet(body: Option<&str>, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let raw = match body {
        Some(b) => b.trim(),
        None => return String::new(),
    };
    if raw.is_empty() {
        return String::new();
    }
    // Normalize whitespace: collapse multiple spaces/newlines into single space
    let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return String::new();
    }
    // Truncate on grapheme cluster boundaries (Unicode-safe)
    normalized.graphemes(true).take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_ascii() {
        let s = make_snippet(Some("hello world test"), 5);
        assert_eq!(s, "hello");
    }

    #[test]
    fn preserves_graphemes() {
        // Test with emoji and skin tone modifiers
        let s = make_snippet(Some("👍👍🏽👍🏿"), 2);
        assert_eq!(s, "👍👍🏽");
    }

    #[test]
    fn collapses_whitespace() {
        let s = make_snippet(Some("line1\n\nline2  line3"), 20);
        assert_eq!(s, "line1 line2 line3");
    }

    #[test]
    fn handles_empty() {
        assert_eq!(make_snippet(None, 10), "");
        assert_eq!(make_snippet(Some(""), 10), "");
        assert_eq!(make_snippet(Some("   "), 10), "");
    }

    #[test]
    fn handles_zero_max() {
        assert_eq!(make_snippet(Some("hello"), 0), "");
    }

    #[test]
    fn handles_multibyte_chars() {
        // Japanese characters
        let s = make_snippet(Some("こんにちは世界"), 5);
        assert_eq!(s, "こんにちは");
    }

    #[test]
    fn handles_mixed_content() {
        let s = make_snippet(Some("Hello 世界 🌍"), 8);
        assert_eq!(s, "Hello 世界");
    }
}
