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

/// Parses an ODT or FODT XML string into a [`Document`].
///
/// Accepts `office:document` (FODT), `office:document-content`,
/// `office:document-styles`, or `office:document-meta` root elements.
///
/// # Errors
///
/// Returns a `String` error if the XML is invalid or the root element
/// is not a recognized ODT document type.
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

        parse_blocks(office_text, ns.text, ns.table, ns.draw, ns.xlink, &style_map)
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
        let new_styles =
            parse_styles_node(styles_elem, ns.style, ns.fo, ns.text, ns.loki)?;
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
