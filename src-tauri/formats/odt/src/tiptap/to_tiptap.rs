//! Conversion from the internal [`Document`] model to Tiptap JSON nodes.
//!
//! Provides [`document_to_tiptap`] which transforms the parsed document
//! into a [`TiptapNode::Doc`] tree suitable for sending to the frontend.

use common_core::tiptap::{ImageAttrs, TiptapAttrs};
use common_core::{Block, Inline, TiptapNode};

/// Converts a slice of blocks to a `TiptapNode::Doc`.
///
/// # Examples
///
/// ```
/// use odt_format::tiptap::to_tiptap::document_to_tiptap;
/// use common_core::Block;
///
/// let blocks = vec![Block::PageBreak];
/// let node = document_to_tiptap(&blocks);
/// ```
pub fn document_to_tiptap(blocks: &[Block]) -> TiptapNode {
    let content = blocks.iter().map(block_to_tiptap).collect();
    TiptapNode::Doc { content }
}

/// Converts a single [`Block`] to a [`TiptapNode`].
///
/// Returns `None` for block types that have no direct Tiptap equivalent.
pub fn block_to_tiptap(block: &Block) -> TiptapNode {
    match block {
        Block::Paragraph { style_name, attrs, content } => {
            TiptapNode::Paragraph {
                attrs: Some(TiptapAttrs {
                    style_name: style_name.clone(),
                    text_align: attrs.as_ref().and_then(|a| a.text_align.clone()),
                    indent: attrs.as_ref().and_then(|a| a.indent),
                    level: None,
                }),
                content: Some(inlines_to_tiptap(content)),
            }
        }
        Block::Heading { level, style_name, attrs, content } => {
            TiptapNode::Heading {
                attrs: Some(TiptapAttrs {
                    style_name: style_name.clone(),
                    text_align: attrs.as_ref().and_then(|a| a.text_align.clone()),
                    indent: attrs.as_ref().and_then(|a| a.indent),
                    level: Some(*level),
                }),
                content: Some(inlines_to_tiptap(content)),
            }
        }
        Block::Image { src, alt, title } => TiptapNode::Image {
            attrs: ImageAttrs {
                src: src.clone(),
                alt: alt.clone(),
                title: title.clone(),
            },
        },
        Block::BulletList { content } => TiptapNode::BulletList {
            content: content.iter().map(block_to_tiptap).collect(),
        },
        Block::OrderedList { content } => TiptapNode::OrderedList {
            content: content.iter().map(block_to_tiptap).collect(),
        },
        Block::ListItem { content } => TiptapNode::ListItem {
            content: content.iter().map(block_to_tiptap).collect(),
        },
        Block::Blockquote { content } => TiptapNode::Blockquote {
            content: content.iter().map(block_to_tiptap).collect(),
        },
        Block::Table { content } => TiptapNode::Table {
            content: content.iter().map(block_to_tiptap).collect(),
        },
        Block::TableRow { content } => TiptapNode::TableRow {
            content: content.iter().map(block_to_tiptap).collect(),
        },
        Block::TableHeader { attrs, content } => TiptapNode::TableHeader {
            attrs: attrs.clone(),
            content: content.iter().map(block_to_tiptap).collect(),
        },
        Block::TableCell { attrs, content } => TiptapNode::TableCell {
            attrs: attrs.clone(),
            content: content.iter().map(block_to_tiptap).collect(),
        },
        Block::HorizontalRule => TiptapNode::HorizontalRule,
        Block::PageBreak => TiptapNode::PageBreak,
    }
}

/// Converts a slice of [`Inline`] values to Tiptap text/break nodes.
pub fn inlines_to_tiptap(inlines: &[Inline]) -> Vec<TiptapNode> {
    inlines
        .iter()
        .map(|inline| match inline {
            Inline::Text { text, marks, .. } => TiptapNode::Text {
                text: text.clone(),
                marks: Some(marks.clone()),
            },
            Inline::LineBreak => TiptapNode::HardBreak,
        })
        .collect()
}
