//! ODT document parser.
//!
//! Provides [`parse_document`] and [`add_styles_from_xml`] as the primary
//! entry points for loading ODT XML content into a [`Document`].
//!
//! # Supported Formats
//!
//! - **FODT** (`office:document`): Single-file flat XML ODT
//! - **ODT content.xml** (`office:document-content`): ZIP-extracted content
//! - **ODT styles.xml** (`office:document-styles`): ZIP-extracted styles
//! - **ODT meta.xml** (`office:document-meta`): ZIP-extracted metadata

pub mod blocks;
pub mod inlines;
pub mod metadata;
pub mod styles;

use crate::document::Document;
use crate::namespaces::Ns;
use crate::parser::blocks::parse_blocks;
use crate::parser::metadata::parse_metadata;
use crate::parser::styles::{parse_styles, parse_styles_node};

/// Maximum XML element nesting depth accepted before parsing is aborted.
///
/// This guards against adversarial inputs that would cause a stack overflow
/// inside roxmltree or the recursive block parser.  No legitimate ODT
/// document approaches this depth.
const MAX_XML_NESTING_DEPTH: usize = 300;

/// Scans `xml` bytes and returns `Err` if the element nesting depth exceeds
/// `max`.  This is a lightweight pre-check that runs before the full XML
/// tree is built, protecting both roxmltree and our recursive parser.
fn check_nesting_depth(xml: &str, max: usize) -> Result<(), String> {
    let b = xml.as_bytes();
    let mut depth: usize = 0;
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'<' {
            i += 1;
            continue;
        }
        i += 1;
        if i >= b.len() {
            break;
        }
        if b[i] == b'/' {
            // Closing tag — consume to '>'
            depth = depth.saturating_sub(1);
            while i < b.len() && b[i] != b'>' {
                i += 1;
            }
        } else if b[i] == b'!' || b[i] == b'?' {
            // Comment, DTD, or PI — consume to '>'
            while i < b.len() && b[i] != b'>' {
                i += 1;
            }
        } else {
            // Opening tag: scan to '>' tracking quoted attribute values
            depth += 1;
            if depth > max {
                return Err(format!("XML nesting depth exceeds maximum of {max}"));
            }
            let mut in_quote = false;
            let mut qchar = b'"';
            while i < b.len() {
                if in_quote {
                    if b[i] == qchar {
                        in_quote = false;
                    }
                } else if b[i] == b'"' || b[i] == b'\'' {
                    in_quote = true;
                    qchar = b[i];
                } else if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'>' {
                    // Self-closing: undo the increment
                    depth -= 1;
                    i += 1; // skip '>'
                    break;
                } else if b[i] == b'>' {
                    break;
                }
                i += 1;
            }
        }
        i += 1; // skip past '>'
    }
    Ok(())
}

/// Parses an ODT or FODT XML string into a [`Document`].
///
/// Accepts `office:document` (FODT), `office:document-content`,
/// `office:document-styles`, or `office:document-meta` root elements.
///
/// # Errors
///
/// Returns a `String` error if the XML is invalid, the nesting depth
/// exceeds the safety limit, or the root element is not a recognized
/// ODT document type.
///
/// # Examples
///
/// ```no_run
/// use odt_format::parser::parse_document;
///
/// let xml = std::fs::read_to_string("document.fodt").unwrap();
/// let doc = parse_document(&xml).unwrap();
/// println!("Blocks: {}", doc.blocks.len());
/// ```
pub fn parse_document(xml: &str) -> Result<Document, String> {
    check_nesting_depth(xml, MAX_XML_NESTING_DEPTH)?;
    let ns = Ns::default();
    let doc = roxmltree::Document::parse(xml).map_err(|e| e.to_string())?;
    let root = doc.root_element();

    validate_root(&root, ns.office)?;

    let metadata = parse_metadata(root, ns.office, ns.dc, ns.meta);
    let (style_definitions, style_map) =
        parse_styles(root, ns.office, ns.style, ns.fo, ns.text, ns.loki);

    let is_meta_only = root.has_tag_name((ns.office, "document-meta"));
    let blocks = if is_meta_only {
        Vec::new()
    } else {
        let office_text = root
            .children()
            .find(|n| n.has_tag_name((ns.office, "body")))
            .and_then(|n| n.children().find(|c| c.has_tag_name((ns.office, "text"))))
            .ok_or("Could not find office:text")?;

        parse_blocks(
            office_text,
            ns.text,
            ns.table,
            ns.draw,
            ns.xlink,
            &style_map,
        )
    };

    Ok(Document {
        blocks,
        styles: style_definitions,
        metadata,
        font_face_decls: None,
        automatic_styles: None,
        master_styles: None,
    })
}

/// Adds styles from a standalone `styles.xml` into an existing [`Document`].
///
/// Used when loading a ZIP-format `.odt` file where styles are stored
/// separately. Also extracts raw XML sections for round-trip preservation.
///
/// # Errors
///
/// Returns a `String` error if the XML is invalid or required elements
/// are missing.
pub fn add_styles_from_xml(doc: &mut Document, xml: &str) -> Result<(), String> {
    check_nesting_depth(xml, MAX_XML_NESTING_DEPTH)?;
    let ns = Ns::default();
    let parsed = roxmltree::Document::parse(xml).map_err(|e| e.to_string())?;

    // Preserve raw XML sections for round-trip fidelity
    doc.font_face_decls = parsed
        .descendants()
        .find(|n| n.has_tag_name((ns.office, "font-face-decls")))
        .map(|n| xml[n.range()].to_string());

    doc.automatic_styles = parsed
        .descendants()
        .find(|n| n.has_tag_name((ns.office, "automatic-styles")))
        .map(|n| xml[n.range()].to_string());

    doc.master_styles = parsed
        .descendants()
        .find(|n| n.has_tag_name((ns.office, "master-styles")))
        .map(|n| xml[n.range()].to_string());

    // Parse styles from the office:styles section
    if let Some(styles_elem) = parsed
        .descendants()
        .find(|n| n.has_tag_name((ns.office, "styles")))
    {
        let new_styles = parse_styles_node(styles_elem, ns.style, ns.fo, ns.text, ns.loki)?;
        doc.styles.extend(new_styles);
    }

    Ok(())
}

/// Validates that the document root is a recognized ODT element type.
fn validate_root(root: &roxmltree::Node, ns_office: &str) -> Result<(), String> {
    let valid = root.has_tag_name((ns_office, "document"))
        || root.has_tag_name((ns_office, "document-content"))
        || root.has_tag_name((ns_office, "document-styles"))
        || root.has_tag_name((ns_office, "document-meta"));

    if valid {
        Ok(())
    } else {
        Err(
            "Invalid ODT XML: root must be office:document, office:document-content, \
             office:document-styles, or office:document-meta"
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
    const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
    const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
    const NS_META: &str = "urn:oasis:names:tc:opendocument:xmlns:meta:1.0";

    fn minimal_fodt(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:dc="{NS_DC}" xmlns:meta="{NS_META}" office:version="1.3">
  <office:body>
    <office:text>{body}</office:text>
  </office:body>
</office:document>"#
        )
    }

    #[test]
    fn parse_invalid_xml_returns_error() {
        assert!(parse_document("<unclosed").is_err());
    }

    #[test]
    fn parse_non_odt_root_returns_error() {
        let xml = r#"<?xml version="1.0"?><root><child/></root>"#;
        let err = parse_document(xml).unwrap_err();
        assert!(err.contains("Invalid ODT XML"));
    }

    #[test]
    fn parse_empty_fodt_yields_no_blocks() {
        let xml = minimal_fodt("");
        let doc = parse_document(&xml).unwrap();
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn parse_fodt_with_paragraph() {
        let body = format!(r#"<text:p xmlns:text="{NS_TEXT}">Hello</text:p>"#);
        let xml = minimal_fodt(&body);
        let doc = parse_document(&xml).unwrap();
        assert_eq!(doc.blocks.len(), 1);
        if let common_core::Block::Paragraph { content, .. } = &doc.blocks[0] {
            assert_eq!(content.len(), 1);
        } else {
            panic!("expected Paragraph");
        }
    }

    #[test]
    fn parse_meta_only_document_yields_no_blocks() {
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="{NS_OFFICE}" xmlns:dc="{NS_DC}"
    xmlns:meta="{NS_META}" office:version="1.3">
  <office:meta>
    <dc:title>My Doc</dc:title>
  </office:meta>
</office:document-meta>"#
        );
        let doc = parse_document(&xml).unwrap();
        assert!(doc.blocks.is_empty());
        assert_eq!(doc.metadata.title.as_deref(), Some("My Doc"));
    }

    #[test]
    fn add_styles_from_xml_invalid_returns_error() {
        let mut doc = Document {
            blocks: vec![],
            styles: std::collections::HashMap::new(),
            metadata: common_core::Metadata::default(),
            font_face_decls: None,
            automatic_styles: None,
            master_styles: None,
        };
        assert!(add_styles_from_xml(&mut doc, "<bad").is_err());
    }
}
