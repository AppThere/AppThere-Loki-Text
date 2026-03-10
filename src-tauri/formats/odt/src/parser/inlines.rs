//! ODT inline content parser.
//!
//! Parses `text:span`, `text:a`, `text:line-break`, and plain text nodes
//! from an ODT XML element into [`Inline`] values.

use std::collections::HashMap;

use common_core::marks::LinkAttrs;
use common_core::{Inline, TiptapMark};

/// Parses inline content from an ODT XML node.
///
/// Walks the children of `node` and converts text nodes, spans, line breaks,
/// and hyperlinks into [`Inline`] values.
///
/// # Arguments
///
/// * `node` - The parent XML node whose children will be parsed.
/// * `ns_text` - The `text:` namespace URI.
/// * `ns_xlink` - The `xlink:` namespace URI.
/// * `style_map` - Map from ODT style name to `(family, marks)` pairs.
pub fn parse_inlines(
    node: roxmltree::Node,
    ns_text: &str,
    ns_xlink: &str,
    style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
) -> Vec<Inline> {
    let mut inlines = Vec::new();
    for child in node.children() {
        if child.is_text() {
            inlines.push(Inline::Text {
                text: child.text().unwrap_or("").to_string(),
                style_name: None,
                marks: Vec::new(),
            });
        } else if child.has_tag_name((ns_text, "span")) {
            let s_name = child.attribute((ns_text, "style-name"));
            let marks = s_name
                .and_then(|s| style_map.get(s))
                .map(|(_, m)| m.clone())
                .unwrap_or_default();
            inlines.push(Inline::Text {
                text: child.text().unwrap_or("").to_string(),
                style_name: s_name.map(|s| s.to_string()),
                marks,
            });
        } else if child.has_tag_name((ns_text, "line-break")) {
            inlines.push(Inline::LineBreak);
        } else if child.has_tag_name((ns_text, "a")) {
            parse_hyperlink(child, ns_text, ns_xlink, style_map, &mut inlines);
        }
    }
    inlines
}

/// Parses a `text:a` hyperlink element, attaching a `Link` mark to each inline.
fn parse_hyperlink(
    child: roxmltree::Node,
    ns_text: &str,
    ns_xlink: &str,
    style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
    inlines: &mut Vec<Inline>,
) {
    let href = child
        .attribute((ns_xlink, "href"))
        .unwrap_or("")
        .to_string();
    let inner = parse_inlines(child, ns_text, ns_xlink, style_map);
    for mut inline in inner {
        if let Inline::Text { ref mut marks, .. } = inline {
            marks.push(TiptapMark::Link {
                attrs: LinkAttrs {
                    href: href.clone(),
                    target: Some("_blank".to_string()),
                },
            });
        }
        inlines.push(inline);
    }
}
