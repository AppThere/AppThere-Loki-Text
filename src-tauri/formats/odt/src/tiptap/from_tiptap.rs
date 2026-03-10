//! Conversion from Tiptap JSON nodes to the internal [`Document`] model.
//!
//! Provides [`tiptap_to_document`] which constructs a [`Document`] from
//! a `TiptapNode::Doc` received from the frontend editor.

use std::collections::HashMap;

use common_core::block::BlockAttrs;
use common_core::{Block, Inline, Metadata, StyleDefinition, TiptapNode};

use crate::document::Document;

/// Constructs a [`Document`] from a Tiptap `Doc` node plus styles and metadata.
///
/// # Arguments
///
/// * `root` - The root `TiptapNode::Doc` node from the frontend.
/// * `styles` - Named style definitions (unchanged from the original document).
/// * `metadata` - Document metadata.
///
/// # Examples
///
/// ```
/// use odt_format::tiptap::from_tiptap::tiptap_to_document;
/// use common_core::{TiptapNode, Metadata};
/// use std::collections::HashMap;
///
/// let root = TiptapNode::Doc { content: vec![] };
/// let doc = tiptap_to_document(root, HashMap::new(), Metadata::default());
/// assert!(doc.blocks.is_empty());
/// ```
pub fn tiptap_to_document(
    root: TiptapNode,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
) -> Document {
    let blocks = if let TiptapNode::Doc { content } = root {
        content.into_iter().filter_map(tiptap_node_to_block).collect()
    } else {
        Vec::new()
    };

    Document {
        blocks,
        styles,
        metadata,
        font_face_decls: None,
        automatic_styles: None,
        master_styles: None,
    }
}

/// Converts a single [`TiptapNode`] to a [`Block`].
///
/// Returns `None` for node types that are inline-only (e.g., `Text`, `HardBreak`).
pub fn tiptap_node_to_block(node: TiptapNode) -> Option<Block> {
    match node {
        TiptapNode::Paragraph { attrs, content } => {
            let style_name = attrs.as_ref().and_then(|a| a.style_name.clone());
            let block_attrs = attrs.map(|a| BlockAttrs {
                text_align: a.text_align,
                indent: a.indent,
            });
            let inlines = tiptap_content_to_inlines(content.unwrap_or_default());
            Some(Block::Paragraph { style_name, attrs: block_attrs, content: inlines })
        }
        TiptapNode::Heading { attrs, content } => {
            let style_name = attrs.as_ref().and_then(|a| a.style_name.clone());
            let level = attrs.as_ref().and_then(|a| a.level).unwrap_or(1);
            let block_attrs = attrs.map(|a| BlockAttrs {
                text_align: a.text_align,
                indent: a.indent,
            });
            let inlines = tiptap_content_to_inlines(content.unwrap_or_default());
            Some(Block::Heading { level, style_name, attrs: block_attrs, content: inlines })
        }
        TiptapNode::Image { attrs } => Some(Block::Image {
            src: attrs.src,
            alt: attrs.alt,
            title: attrs.title,
        }),
        TiptapNode::BulletList { content } => Some(Block::BulletList {
            content: content.into_iter().filter_map(tiptap_node_to_block).collect(),
        }),
        TiptapNode::OrderedList { content } => Some(Block::OrderedList {
            content: content.into_iter().filter_map(tiptap_node_to_block).collect(),
        }),
        TiptapNode::ListItem { content } => Some(Block::ListItem {
            content: content.into_iter().filter_map(tiptap_node_to_block).collect(),
        }),
        TiptapNode::Blockquote { content } => Some(Block::Blockquote {
            content: content.into_iter().filter_map(tiptap_node_to_block).collect(),
        }),
        TiptapNode::Table { content } => Some(Block::Table {
            content: content.into_iter().filter_map(tiptap_node_to_block).collect(),
        }),
        TiptapNode::TableRow { content } => Some(Block::TableRow {
            content: content.into_iter().filter_map(tiptap_node_to_block).collect(),
        }),
        TiptapNode::TableHeader { attrs, content } => Some(Block::TableHeader {
            attrs,
            content: content.into_iter().filter_map(tiptap_node_to_block).collect(),
        }),
        TiptapNode::TableCell { attrs, content } => Some(Block::TableCell {
            attrs,
            content: content.into_iter().filter_map(tiptap_node_to_block).collect(),
        }),
        TiptapNode::HorizontalRule => Some(Block::HorizontalRule),
        TiptapNode::PageBreak => Some(Block::PageBreak),
        _ => None,
    }
}

/// Converts a list of Tiptap inline nodes to [`Inline`] values.
fn tiptap_content_to_inlines(nodes: Vec<TiptapNode>) -> Vec<Inline> {
    nodes
        .into_iter()
        .filter_map(|node| match node {
            TiptapNode::Text { text, marks } => Some(Inline::Text {
                text,
                style_name: None,
                marks: marks.unwrap_or_default(),
            }),
            TiptapNode::HardBreak => Some(Inline::LineBreak),
            _ => None,
        })
        .collect()
}
