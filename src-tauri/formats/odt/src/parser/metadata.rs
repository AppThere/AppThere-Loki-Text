//! ODT metadata parser.
//!
//! Parses the `<office:meta>` section of an ODT XML document into a
//! [`Metadata`] struct containing Dublin Core and ODF meta fields.

use common_core::Metadata;

/// Parses the `<office:meta>` element from the document root.
///
/// Extracts Dublin Core elements (title, description, creator, etc.) and
/// ODF meta elements (generator, creation-date) into a [`Metadata`] struct.
///
/// # Arguments
///
/// * `root` - The document root element.
/// * `ns_office` - The `office:` namespace URI.
/// * `ns_dc` - The Dublin Core namespace URI.
/// * `ns_meta` - The ODF meta namespace URI.
pub fn parse_metadata(
    root: roxmltree::Node,
    ns_office: &str,
    ns_dc: &str,
    ns_meta: &str,
) -> Metadata {
    let mut metadata = Metadata::default();

    let meta_node = root
        .children()
        .find(|n| n.has_tag_name((ns_office, "meta")));

    if let Some(meta_node) = meta_node {
        for child in meta_node.children() {
            if child.has_tag_name((ns_dc, "title")) {
                metadata.title = child.text().map(|s| s.to_string());
            } else if child.has_tag_name((ns_dc, "description")) {
                metadata.description = child.text().map(|s| s.to_string());
            } else if child.has_tag_name((ns_dc, "subject")) {
                metadata.subject = child.text().map(|s| s.to_string());
            } else if child.has_tag_name((ns_dc, "creator")) {
                metadata.creator = child.text().map(|s| s.to_string());
            } else if child.has_tag_name((ns_meta, "creation-date")) {
                metadata.creation_date = child.text().map(|s| s.to_string());
            } else if child.has_tag_name((ns_meta, "generator")) {
                metadata.generator = child.text().map(|s| s.to_string());
            } else if child.has_tag_name((ns_dc, "identifier")) {
                metadata.identifier = child.text().map(|s| s.to_string());
            } else if child.has_tag_name((ns_dc, "language")) {
                metadata.language = child.text().map(|s| s.to_string());
            }
        }
    }

    metadata
}
