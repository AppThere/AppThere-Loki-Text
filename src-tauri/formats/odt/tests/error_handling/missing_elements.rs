//! Tests that well-formed XML missing required ODT structure is rejected.
//!
//! The parser must return `Err(_)` (not panic) when required elements such as
//! `office:body`, `office:text`, or the document root are absent or wrong.

use odt_format::Document;

const NS_OFFICE: &str = super::NS_OFFICE;
const NS_TEXT: &str = super::NS_TEXT;

// ── Reject wrong / missing structure ─────────────────────────────────────────

/// A root element that is not a recognised ODT type must be rejected with
/// an error mentioning "Invalid ODT XML".
#[test]
fn test_wrong_root_element() {
    let xml = r#"<?xml version="1.0"?>
<wrong:root xmlns:wrong="http://example.com">
  <content>Not an ODT document</content>
</wrong:root>"#;
    let result = Document::from_xml(xml);
    assert!(result.is_err(), "Wrong root element should be rejected");
    let err = result.unwrap_err();
    assert!(
        err.contains("Invalid ODT XML"),
        "Error should mention 'Invalid ODT XML': {err}"
    );
}

/// A document-content wrapper without `office:body` must fail.
#[test]
fn test_document_content_without_body() {
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document-content xmlns:office="{NS_OFFICE}">
  <!-- No office:body here -->
</office:document-content>"#
    );
    let result = Document::from_xml(&xml);
    assert!(result.is_err(), "Missing office:body should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("office:text") || err.contains("Could not find"),
        "Error should mention missing element: {err}"
    );
}

/// `office:body` present but `office:text` absent must fail.
#[test]
fn test_document_content_without_office_text() {
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document-content xmlns:office="{NS_OFFICE}">
  <office:body>
    <!-- No office:text here -->
  </office:body>
</office:document-content>"#
    );
    let result = Document::from_xml(&xml);
    assert!(result.is_err(), "Missing office:text should fail");
    let err = result.unwrap_err();
    assert!(
        err.contains("office:text") || err.contains("Could not find"),
        "Error should mention missing element: {err}"
    );
}

// ── Accept valid but sparse structure ─────────────────────────────────────────

/// An empty `text:list-item` (no child elements) should not cause a panic.
#[test]
fn test_empty_list_item() {
    let body = "<text:list><text:list-item></text:list-item></text:list>";
    let xml = super::fodt(body);
    let result = Document::from_xml(&xml);
    assert!(
        result.is_ok(),
        "Empty list-item should be accepted: {result:?}"
    );
}

/// A `table:table` with no rows should parse without error.
#[test]
fn test_empty_table() {
    let body = r#"<table:table xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0">
          <!-- Empty table -->
        </table:table>"#
        .to_string();
    let xml = super::fodt(&body);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Empty table should be accepted: {result:?}");
}

/// A table row with no cells should not panic.
#[test]
fn test_table_row_without_cells() {
    let body = r#"<table:table xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0">
           <table:table-row></table:table-row>
         </table:table>"#
        .to_string();
    let xml = super::fodt(&body);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Table row without cells should be accepted");
}

/// A list item containing only another list (nested) should not panic.
#[test]
fn test_list_item_containing_only_list() {
    let body = r#"<text:list>
      <text:list-item>
        <text:list>
          <text:list-item><text:p>Inner</text:p></text:list-item>
        </text:list>
      </text:list-item>
    </text:list>"#;
    let xml = super::fodt(body);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Nested list items should be accepted");
}

/// Heading with no content should not panic.
#[test]
fn test_empty_heading() {
    let body = format!(r#"<text:h xmlns:text="{NS_TEXT}" text:outline-level="1"></text:h>"#);
    let xml = super::fodt(&body);
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Empty heading should be accepted");
}
