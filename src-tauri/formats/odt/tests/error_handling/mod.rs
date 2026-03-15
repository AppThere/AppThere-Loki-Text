//! Error handling test suite.
//!
//! Tests that the parser and writer handle bad inputs gracefully — returning
//! `Err(_)` rather than panicking, and producing useful error messages.

pub mod extreme_inputs;
pub mod invalid_xml;
pub mod missing_elements;
pub mod unicode_edge_cases;

// ── Shared helpers ────────────────────────────────────────────────────────────

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
const NS_TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";

/// Wraps `body` in a minimal FODT document string with all common namespaces.
pub fn fodt(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}"
    xmlns:table="{NS_TABLE}" office:version="1.3">
  <office:body>
    <office:text>{body}</office:text>
  </office:body>
</office:document>"#
    )
}
