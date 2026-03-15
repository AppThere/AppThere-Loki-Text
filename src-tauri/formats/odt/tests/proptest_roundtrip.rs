//! Property-based round-trip tests using proptest.
//!
//! Each test exercises the full pipeline with 1 000 randomly-generated inputs:
//!
//!   ODT XML (generated) → parse → to_lexical → from_lexical → blocks/metadata
//!
//! Properties verified:
//! * Paragraph text is preserved verbatim.
//! * Heading level survives the full round-trip.
//! * Block count is stable for multi-paragraph documents.
//! * Metadata (title, creator) passed to `from_lexical` is returned unchanged.
//! * Basic Unicode text (Arabic, CJK, Cyrillic) is not corrupted.

use odt_format::{
    lexical::{from_lexical, to_lexical},
    parser::parse_document,
};
use proptest::prelude::*;

// ── XML namespace constants ───────────────────────────────────────────────────

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Minimal FODT wrapper — only the namespaces actually used in the body.
fn fodt(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}" office:version="1.3">
  <office:body>
    <office:text>{body}</office:text>
  </office:body>
</office:document>"#
    )
}

/// Escape the five XML predefined entities so generated text is always valid.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Run parse → to_lexical → from_lexical and return the resulting blocks.
fn round_trip_blocks(xml: &str) -> Vec<common_core::Block> {
    let doc = parse_document(xml).expect("parse failed");
    let lex = to_lexical(&doc);
    let doc2 = from_lexical(lex, doc.styles.clone(), doc.metadata.clone());
    doc2.blocks
}

// ── Property strategies ───────────────────────────────────────────────────────

/// Printable ASCII text (no XML-special characters) — fast and deterministic.
fn safe_ascii() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 ,.!?;:()\\-]{1,80}".prop_map(|s| s)
}

/// Mixed-script Unicode: Latin + Cyrillic + Arabic + CJK (BMP only).
/// Control chars and XML-reserved chars are excluded.
fn unicode_text() -> impl Strategy<Value = String> {
    proptest::string::string_regex(
        "[a-zA-Z0-9 \u{0400}-\u{04FF}\u{0600}-\u{06FF}\u{4E00}-\u{4E7F}]{1,60}",
    )
    .unwrap()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Any safe-ASCII string placed inside a `<text:p>` must survive the
    /// full round-trip as an unchanged text inline.
    #[test]
    fn prop_paragraph_text_survives_roundtrip(text in safe_ascii()) {
        let escaped = xml_escape(&text);
        let xml = fodt(&format!("<text:p>{escaped}</text:p>"));
        let blocks = round_trip_blocks(&xml);

        prop_assert_eq!(blocks.len(), 1, "block count changed");
        if let common_core::Block::Paragraph { content, .. } = &blocks[0] {
            prop_assert!(!content.is_empty(), "paragraph lost its content");
            // Collect all text from inlines and compare
            let recovered: String = content
                .iter()
                .filter_map(|i| {
                    if let common_core::Inline::Text { text: t, .. } = i {
                        Some(t.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            prop_assert_eq!(recovered, text, "paragraph text changed");
        } else {
            return Err(TestCaseError::fail("expected Paragraph block"));
        }
    }

    /// Heading levels 1–6 must be preserved exactly after round-trip.
    #[test]
    fn prop_heading_level_preserved(
        level in 1u32..=6u32,
        text in safe_ascii(),
    ) {
        let escaped = xml_escape(&text);
        let xml = fodt(&format!(
            r#"<text:h text:outline-level="{level}">{escaped}</text:h>"#
        ));
        let blocks = round_trip_blocks(&xml);

        prop_assert_eq!(blocks.len(), 1, "block count changed");
        if let common_core::Block::Heading { level: recovered, .. } = &blocks[0] {
            prop_assert_eq!(*recovered, level, "heading level changed");
        } else {
            return Err(TestCaseError::fail("expected Heading block"));
        }
    }

    /// A document with N paragraphs must still have exactly N blocks
    /// after round-trip.
    #[test]
    fn prop_paragraph_count_preserved(
        texts in proptest::collection::vec(safe_ascii(), 1..=20),
    ) {
        let body: String = texts
            .iter()
            .map(|t| format!("<text:p>{}</text:p>", xml_escape(t)))
            .collect();
        let xml = fodt(&body);
        let blocks = round_trip_blocks(&xml);

        prop_assert_eq!(
            blocks.len(),
            texts.len(),
            "block count changed (expected {}, got {})",
            texts.len(),
            blocks.len()
        );
    }

    /// Metadata fields are passed directly to `from_lexical`; they must
    /// come back unchanged in the resulting document.
    #[test]
    fn prop_metadata_preserved_through_lexical(
        title in proptest::option::of(safe_ascii()),
        creator in proptest::option::of("[a-zA-Z ]{1,40}"),
    ) {
        use common_core::Metadata;

        let meta = Metadata {
            title: title.clone(),
            creator: creator.clone(),
            ..Metadata::default()
        };
        // Build a minimal document, then inject custom metadata before conversion
        let doc = parse_document(&fodt("<text:p>body</text:p>"))
            .expect("parse failed");
        let lex = to_lexical(&doc);
        let doc2 = from_lexical(lex, doc.styles.clone(), meta.clone());

        prop_assert_eq!(doc2.metadata.title, title, "title changed");
        prop_assert_eq!(doc2.metadata.creator, creator, "creator changed");
    }

    /// Unicode text (Cyrillic, Arabic, CJK) placed in a paragraph must
    /// survive the round-trip without corruption.
    #[test]
    fn prop_unicode_text_preserved(text in unicode_text()) {
        // XML-escape in case the random chars include & < > " '
        let escaped = xml_escape(&text);
        let xml = fodt(&format!("<text:p>{escaped}</text:p>"));
        let blocks = round_trip_blocks(&xml);

        prop_assert_eq!(blocks.len(), 1, "block count changed");
        if let common_core::Block::Paragraph { content, .. } = &blocks[0] {
            let recovered: String = content
                .iter()
                .filter_map(|i| {
                    if let common_core::Inline::Text { text: t, .. } = i {
                        Some(t.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            prop_assert_eq!(recovered, text, "unicode text corrupted");
        } else {
            return Err(TestCaseError::fail("expected Paragraph block"));
        }
    }
}
