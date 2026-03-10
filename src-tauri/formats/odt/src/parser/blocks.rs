//! ODT block content parser.
//!
//! Parses `text:p`, `text:h`, `text:list`, and `table:table` elements from
//! an ODT XML body node into [`Block`] values.

use std::collections::HashMap;

use common_core::{Block, TiptapMark};

use crate::parser::inlines::parse_inlines;

/// Parses block-level content from an ODT `office:text` (or list-item) node.
///
/// # Arguments
///
/// * `node` - The parent XML node (e.g., `office:text` or `text:list-item`).
/// * `ns_text` - The `text:` namespace URI.
/// * `ns_table` - The `table:` namespace URI.
/// * `ns_draw` - The `draw:` namespace URI.
/// * `ns_xlink` - The `xlink:` namespace URI.
/// * `style_map` - Map from style name to `(family, marks)` for inline formatting.
pub fn parse_blocks(
    node: roxmltree::Node,
    ns_text: &str,
    ns_table: &str,
    ns_draw: &str,
    ns_xlink: &str,
    style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    for child in node.children() {
        if child.has_tag_name((ns_text, "p")) {
            parse_paragraph(&child, ns_text, ns_draw, ns_xlink, style_map, &mut blocks);
        } else if child.has_tag_name((ns_text, "h")) {
            parse_heading(&child, ns_text, ns_xlink, style_map, &mut blocks);
        } else if child.has_tag_name((ns_text, "list")) {
            parse_list(&child, ns_text, ns_table, ns_draw, ns_xlink, style_map, &mut blocks);
        } else if child.has_tag_name((ns_table, "table")) {
            parse_table(&child, ns_text, ns_table, ns_draw, ns_xlink, style_map, &mut blocks);
        }
    }
    blocks
}

/// Parses a `text:p` element (paragraph or image or page-break).
fn parse_paragraph(
    child: &roxmltree::Node,
    ns_text: &str,
    ns_draw: &str,
    ns_xlink: &str,
    style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
    blocks: &mut Vec<Block>,
) {
    // Check for embedded image frame
    if let Some(img) = child
        .children()
        .find(|n| n.has_tag_name((ns_draw, "frame")))
        .and_then(|frame| frame.children().find(|n| n.has_tag_name((ns_draw, "image"))))
    {
        let href = img.attribute((ns_xlink, "href")).unwrap_or("").to_string();
        blocks.push(Block::Image { src: href, alt: None, title: None });
        return;
    }

    let style_name = child.attribute((ns_text, "style-name")).map(|s| s.to_string());

    if style_name.as_deref() == Some("PageBreak") {
        blocks.push(Block::PageBreak);
        return;
    }

    let content = parse_inlines(*child, ns_text, ns_xlink, style_map);
    blocks.push(Block::Paragraph { style_name, attrs: None, content });
}

/// Parses a `text:h` element (heading).
fn parse_heading(
    child: &roxmltree::Node,
    ns_text: &str,
    ns_xlink: &str,
    style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
    blocks: &mut Vec<Block>,
) {
    let level = child
        .attribute((ns_text, "outline-level"))
        .and_then(|l| l.parse().ok())
        .unwrap_or(1);
    let style_name = child.attribute((ns_text, "style-name")).map(|s| s.to_string());
    let content = parse_inlines(*child, ns_text, ns_xlink, style_map);
    blocks.push(Block::Heading { level, style_name, attrs: None, content });
}

/// Parses a `text:list` element into a `BulletList`.
fn parse_list(
    child: &roxmltree::Node,
    ns_text: &str,
    ns_table: &str,
    ns_draw: &str,
    ns_xlink: &str,
    style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
    blocks: &mut Vec<Block>,
) {
    let mut items = Vec::new();
    for item in child.children().filter(|n| n.has_tag_name((ns_text, "list-item"))) {
        let content = parse_blocks(item, ns_text, ns_table, ns_draw, ns_xlink, style_map);
        items.push(Block::ListItem { content });
    }
    blocks.push(Block::BulletList { content: items });
}

/// Parses a `table:table` element.
fn parse_table(
    child: &roxmltree::Node,
    ns_text: &str,
    ns_table: &str,
    ns_draw: &str,
    ns_xlink: &str,
    style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
    blocks: &mut Vec<Block>,
) {
    let mut rows = Vec::new();
    for row in child.children().filter(|n| n.has_tag_name((ns_table, "table-row"))) {
        let mut cells = Vec::new();
        for cell in row.children() {
            if cell.has_tag_name((ns_table, "table-cell")) {
                let content = parse_blocks(cell, ns_text, ns_table, ns_draw, ns_xlink, style_map);
                cells.push(Block::TableCell { attrs: None, content });
            }
        }
        rows.push(Block::TableRow { content: cells });
    }
    blocks.push(Block::Table { content: rows });
}
