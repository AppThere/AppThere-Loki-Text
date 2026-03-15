//! Tests for Unicode edge cases: emoji, CJK, RTL text, and special codepoints.
//!
//! Verifies that multibyte and special Unicode characters survive parse →
//! write → parse round-trips without corruption or panic.

use common_core::{Block, Inline};
use odt_format::Document;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse a document, write it with `to_content_xml`, parse again, and return
/// the text of the first inline in the first paragraph.
fn round_trip_first_text(input_xml: &str) -> Option<String> {
    let doc1 = Document::from_xml(input_xml).ok()?;
    let out = doc1.to_content_xml().ok()?;
    let doc2 = Document::from_xml(&out).ok()?;
    if let Block::Paragraph { content, .. } = doc2.blocks.first()? {
        if let Inline::Text { text, .. } = content.first()? {
            return Some(text.clone());
        }
    }
    None
}

/// Build a single-paragraph FODT document containing `text`.
fn para(text: &str) -> String {
    super::fodt(&format!("<text:p>{text}</text:p>"))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Emoji (multi-codepoint supplementary characters) must survive round-trip.
#[test]
fn test_emoji_round_trip() {
    let text = "Rust is great! 🦀🎉✨";
    let xml = para(text);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Emoji input should parse");
    let restored = round_trip_first_text(&xml);
    assert_eq!(restored.as_deref(), Some(text), "Emoji should be preserved");
}

/// CJK characters (Chinese, Japanese, Korean) must survive round-trip.
#[test]
fn test_cjk_text_round_trip() {
    let text = "测试中文文本 テストテキスト 테스트";
    let xml = para(text);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "CJK text should parse");
    let restored = round_trip_first_text(&xml);
    assert_eq!(
        restored.as_deref(),
        Some(text),
        "CJK text should be preserved"
    );
}

/// Hebrew (right-to-left) text must survive round-trip.
#[test]
fn test_rtl_hebrew_round_trip() {
    let text = "שלום עולם";
    let xml = para(text);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Hebrew RTL text should parse");
    let restored = round_trip_first_text(&xml);
    assert_eq!(
        restored.as_deref(),
        Some(text),
        "Hebrew text should be preserved"
    );
}

/// Arabic (right-to-left) text must survive round-trip.
#[test]
fn test_rtl_arabic_round_trip() {
    let text = "مرحبا بالعالم";
    let xml = para(text);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Arabic RTL text should parse");
    let restored = round_trip_first_text(&xml);
    assert_eq!(
        restored.as_deref(),
        Some(text),
        "Arabic text should be preserved"
    );
}

/// Zero-width space (U+200B) is a valid XML character and must not panic.
#[test]
fn test_zero_width_space() {
    let text = "Hello\u{200B}World";
    let xml = para(text);
    let result = Document::from_xml(&xml);
    assert!(
        result.is_ok(),
        "Zero-width space should parse without error"
    );
}

/// RTL override character (U+202E) is a valid XML character and must not panic.
#[test]
fn test_rtl_override_character() {
    let text = "Test\u{202E}Reversed".to_string();
    let xml = para(&text);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "RTL override character should parse");
}

/// Null bytes are NOT valid in XML — the parser must return an error.
#[test]
fn test_null_byte_rejected() {
    let xml = super::fodt("<text:p>Text\x00Null</text:p>");
    let result = Document::from_xml(&xml);
    // roxmltree rejects null bytes as invalid XML characters.
    assert!(
        result.is_err(),
        "Null bytes in XML content should be rejected"
    );
}

/// Mixed scripts (Latin, CJK, emoji, Arabic) in the same paragraph must parse.
#[test]
fn test_mixed_scripts() {
    let text = "Hello 🦀 世界 مرحبا Привет";
    let xml = para(text);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Mixed-script paragraph should parse");
}
