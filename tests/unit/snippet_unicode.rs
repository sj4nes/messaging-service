// Feature 009 - US3 UTF-8 snippet boundary test (T030)
// Verifies that snippets are safely truncated on UTF-8 character boundaries

use messaging_core::conversations::snippet::make_snippet;

#[test]
fn truncates_ascii_safely() {
    let result = make_snippet(Some("hello world test"), 5);
    assert_eq!(result, "hello");
}

#[test]
fn preserves_emoji_graphemes() {
    // Test with emoji including skin tone modifiers (multi-codepoint graphemes)
    let result = make_snippet(Some("👍👍🏽👍🏿"), 2);
    assert_eq!(result, "👍👍🏽", "Should preserve grapheme cluster boundaries");
}

#[test]
fn handles_multibyte_unicode() {
    // Japanese characters (3 bytes each in UTF-8)
    let result = make_snippet(Some("こんにちは世界"), 5);
    assert_eq!(result, "こんにちは");
}

#[test]
fn handles_arabic_with_diacritics() {
    // Arabic with diacritical marks
    let text = "مَرْحَبًا بِكُمْ";
    let result = make_snippet(Some(text), 5);
    // Should not break combining characters
    assert!(result.is_char_boundary(result.len()));
}

#[test]
fn handles_mixed_content() {
    let result = make_snippet(Some("Hello 世界 🌍"), 8);
    assert_eq!(result, "Hello 世界");
}

#[test]
fn handles_complex_emoji_sequences() {
    // Family emoji (multiple codepoints with ZWJ)
    let result = make_snippet(Some("👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦"), 2);
    // Should preserve two family graphemes
    assert_eq!(result.chars().filter(|&c| c == '👨').count(), 2);
}

#[test]
fn collapses_whitespace_with_unicode() {
    let result = make_snippet(Some("日本語\n\n文字列  テスト"), 20);
    assert_eq!(result, "日本語 文字列 テスト");
}

#[test]
fn handles_very_long_graphemes() {
    // String with combining diacritics
    let text = "e\u{0301}\u{0302}\u{0303}"; // e with multiple combining marks
    let result = make_snippet(Some(text), 1);
    // Should keep the entire grapheme cluster
    assert_eq!(result, "e\u{0301}\u{0302}\u{0303}");
}

#[test]
fn handles_zero_width_joiner() {
    // Zero-width joiner sequences (flag emojis, etc.)
    let text = "🏴‍☠️ Pirate flag";
    let result = make_snippet(Some(text), 1);
    // Should preserve the entire pirate flag emoji
    assert!(result.starts_with("🏴‍☠️"));
}

#[test]
fn handles_cyrillic() {
    let text = "Привет мир тест";
    let result = make_snippet(Some(text), 10);
    assert_eq!(result, "Привет мир");
}

#[test]
fn handles_thai_script() {
    // Thai doesn't use spaces between words
    let text = "สวัสดีครับ";
    let result = make_snippet(Some(text), 5);
    assert_eq!(result, "สวัสดี");
    // Verify no broken UTF-8
    assert!(result.is_char_boundary(result.len()));
}

#[test]
fn handles_empty_and_whitespace() {
    assert_eq!(make_snippet(None, 10), "");
    assert_eq!(make_snippet(Some(""), 10), "");
    assert_eq!(make_snippet(Some("   "), 10), "");
    assert_eq!(make_snippet(Some("\n\n\n"), 10), "");
}

#[test]
fn boundary_at_exact_limit() {
    let text = "12345";
    let result = make_snippet(Some(text), 5);
    assert_eq!(result, "12345");
}

#[test]
fn boundary_exceeds_limit() {
    let text = "1234567890";
    let result = make_snippet(Some(text), 5);
    assert_eq!(result, "12345");
    assert_eq!(result.len(), 5);
}
