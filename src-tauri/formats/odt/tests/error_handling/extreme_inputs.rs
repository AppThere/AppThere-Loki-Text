//! Tests with extreme inputs: huge files, deep nesting, and many styles.
//!
//! These tests verify that the parser and writer do not panic on abnormal
//! but syntactically valid inputs, and that they complete in bounded time.

use odt_format::Document;

const NS_STYLE: &str = super::NS_STYLE;
const NS_FO: &str = super::NS_FO;

// ── Large content ─────────────────────────────────────────────────────────────

/// A single paragraph containing 1 MB of text must parse and round-trip.
#[test]
fn test_very_long_text_in_paragraph() {
    let long_text = "a".repeat(1_024 * 1_024); // 1 MB
    let body = format!("<text:p>{long_text}</text:p>");
    let xml = super::fodt(&body);

    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Should handle very long paragraph text");

    let doc = result.unwrap();
    let out = doc.to_content_xml().unwrap();
    assert!(out.len() >= long_text.len(), "Long text must be preserved");
}

/// 10 000 paragraphs should parse without error or excessive memory usage.
#[test]
fn test_many_paragraphs() {
    let body: String = (0..10_000)
        .map(|i| format!("<text:p>Paragraph {i}</text:p>"))
        .collect();
    let xml = super::fodt(&body);

    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "10 000 paragraphs should parse");
    assert_eq!(result.unwrap().blocks.len(), 10_000);
}

/// 100-level nested lists must not trigger a stack overflow.
///
/// roxmltree is a non-recursive parser so deep nesting is safe; the test
/// verifies the output is either parsed or rejected gracefully.
#[test]
fn test_deeply_nested_lists() {
    let depth = 100_usize;
    let opening: String = (0..depth)
        .map(|_| "<text:list><text:list-item>".to_string())
        .collect();
    let closing: String = (0..depth)
        .map(|_| "</text:list-item></text:list>".to_string())
        .collect();
    let body = format!("{opening}<text:p>Deep</text:p>{closing}");
    let xml = super::fodt(&body);

    // Must not panic; may return Ok or Err.
    match Document::from_xml(&xml) {
        Ok(doc) => assert!(!doc.blocks.is_empty(), "Should parse at least one block"),
        Err(e) => assert!(!e.contains("panic"), "Error must not mention panic: {e}"),
    }
}

/// 10 000 named styles must all be parsed and stored.
#[test]
fn test_huge_number_of_styles() {
    let style_defs: String = (0..10_000)
        .map(|i| {
            format!(
                r#"<style:style style:name="S{i}" style:family="paragraph">
                     <style:paragraph-properties fo:text-align="left"/>
                   </style:style>"#
            )
        })
        .collect();

    let ns_office = super::NS_OFFICE;
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{ns_office}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}"
    xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
    office:version="1.3">
  <office:automatic-styles>
    {style_defs}
  </office:automatic-styles>
  <office:body>
    <office:text><text:p text:style-name="S0">Test</text:p></office:text>
  </office:body>
</office:document>"#
    );

    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "10 000 styles should parse");
    assert_eq!(result.unwrap().styles.len(), 10_000);
}

/// 100K paragraphs — skipped in CI but runnable manually.
///
/// Run with: `cargo test test_very_many_paragraphs -- --ignored --nocapture`
#[test]
#[ignore]
fn test_very_many_paragraphs() {
    let body: String = (0..100_000)
        .map(|i| format!("<text:p>Paragraph {i}</text:p>"))
        .collect();
    let xml = super::fodt(&body);

    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "100 000 paragraphs should parse");
    assert_eq!(result.unwrap().blocks.len(), 100_000);
}
