//! Tests that structurally invalid XML is rejected cleanly.
//!
//! Every test must verify that the parser returns `Err(_)` (never panics)
//! or `Ok(_)` for inputs that are valid XML but happen to be edge cases.

use odt_format::Document;

// ── Reject invalid XML ────────────────────────────────────────────────────────

/// Plain text that is not XML at all must return an error.
#[test]
fn test_completely_invalid_xml() {
    let result = Document::from_xml("This is not XML at all");
    assert!(result.is_err(), "Should reject non-XML input");
}

/// An XML document with an unclosed tag is malformed and must be rejected.
#[test]
fn test_malformed_xml_unclosed_tags() {
    let xml = super::fodt("<text:p>Unclosed paragraph");
    // Deliberately strip the closing tags to make it malformed.
    let truncated = &xml[..xml.len() - 80.min(xml.len())];
    let result = Document::from_xml(truncated);
    assert!(result.is_err(), "Should reject XML with unclosed tags");
}

/// Completely empty string is not valid XML.
#[test]
fn test_empty_string() {
    let result = Document::from_xml("");
    assert!(result.is_err(), "Empty string should not parse as XML");
}

/// A tag opened as `<text:p>` but closed as `<text:h>` is a well-formedness
/// error that roxmltree must reject.
#[test]
fn test_mismatched_tags() {
    // Embed directly without the helper so the mismatched tag survives
    let ns_office = super::NS_OFFICE;
    let ns_text = super::NS_TEXT;
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document xmlns:office="{ns_office}" xmlns:text="{ns_text}">
  <office:body>
    <office:text>
      <text:p>Content</text:h>
    </office:text>
  </office:body>
</office:document>"#
    );
    let result = Document::from_xml(&xml);
    assert!(result.is_err(), "Mismatched tags should be rejected");
}

// ── Accept valid edge-case XML ────────────────────────────────────────────────

/// An FODT document with an empty `office:text` body is perfectly valid and
/// should parse to a document with zero blocks.
#[test]
fn test_empty_document_body() {
    let xml = super::fodt("");
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Empty document body should be accepted");
    assert_eq!(
        result.unwrap().blocks.len(),
        0,
        "Empty body should yield zero blocks"
    );
}

/// XML comments must be ignored by the parser.
#[test]
fn test_xml_comments_are_ignored() {
    let body = "<!-- comment --><text:p>Para 1</text:p><!-- another --><text:p>Para 2</text:p>";
    let xml = super::fodt(body);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Comments should not prevent parsing");
    assert_eq!(result.unwrap().blocks.len(), 2, "Two paragraphs expected");
}

/// XML processing instructions other than the `<?xml?>` declaration should
/// not cause a panic.
#[test]
fn test_processing_instruction_in_body() {
    let body = "<?custom instruction?><text:p>Paragraph</text:p>";
    let xml = super::fodt(body);
    // PI inside an element may or may not be valid depending on the parser;
    // verify no panic.
    let _result = Document::from_xml(&xml);
}

/// Bare text inside `office:text` (outside any paragraph) should not panic.
#[test]
fn test_bare_text_in_body() {
    let body = "Bare text without a paragraph element";
    let xml = super::fodt(body);
    let result = Document::from_xml(&xml);
    // Parser silently skips unrecognised text nodes — valid output expected.
    assert!(
        result.is_ok(),
        "Bare text in body should not panic or error"
    );
}

/// XML with only a BOM should fail without panicking.
#[test]
fn test_bom_only() {
    let result = Document::from_xml("\u{FEFF}");
    assert!(result.is_err(), "BOM-only input should return an error");
}
