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
        Block::Paragraph {
            style_name,
            attrs,
            content,
        } => TiptapNode::Paragraph {
            attrs: Some(TiptapAttrs {
                style_name: style_name.clone(),
                text_align: attrs.as_ref().and_then(|a| a.text_align.clone()),
                indent: attrs.as_ref().and_then(|a| a.indent),
                level: None,
            }),
            content: Some(inlines_to_tiptap(content)),
        },
        Block::Heading {
            level,
            style_name,
            attrs,
            content,
        } => TiptapNode::Heading {
            attrs: Some(TiptapAttrs {
                style_name: style_name.clone(),
                text_align: attrs.as_ref().and_then(|a| a.text_align.clone()),
                indent: attrs.as_ref().and_then(|a| a.indent),
                level: Some(*level),
            }),
            content: Some(inlines_to_tiptap(content)),
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use common_core::block::BlockAttrs;
    use common_core::marks::TiptapMark;

    #[test]
    fn document_to_tiptap_empty() {
        let node = document_to_tiptap(&[]);
        if let TiptapNode::Doc { content } = node {
            assert!(content.is_empty());
        } else {
            panic!("expected Doc");
        }
    }

    #[test]
    fn document_to_tiptap_page_break() {
        let node = document_to_tiptap(&[Block::PageBreak]);
        if let TiptapNode::Doc { content } = node {
            assert!(matches!(content[0], TiptapNode::PageBreak));
        } else {
            panic!("expected Doc");
        }
    }

    #[test]
    fn block_to_tiptap_paragraph_preserves_style() {
        let block = Block::Paragraph {
            style_name: Some("Body Text".to_string()),
            attrs: Some(BlockAttrs {
                text_align: Some("right".to_string()),
                indent: None,
            }),
            content: vec![],
        };
        let node = block_to_tiptap(&block);
        if let TiptapNode::Paragraph { attrs, .. } = node {
            let a = attrs.unwrap();
            assert_eq!(a.style_name.as_deref(), Some("Body Text"));
            assert_eq!(a.text_align.as_deref(), Some("right"));
        } else {
            panic!("expected Paragraph");
        }
    }

    #[test]
    fn block_to_tiptap_heading_carries_level() {
        let block = Block::Heading {
            level: 2,
            style_name: None,
            attrs: None,
            content: vec![],
        };
        let node = block_to_tiptap(&block);
        if let TiptapNode::Heading { attrs, .. } = node {
            assert_eq!(attrs.unwrap().level, Some(2));
        } else {
            panic!("expected Heading");
        }
    }

    #[test]
    fn block_to_tiptap_image() {
        let block = Block::Image {
            src: "hero.jpg".to_string(),
            alt: None,
            title: Some("Hero".to_string()),
        };
        let node = block_to_tiptap(&block);
        if let TiptapNode::Image { attrs } = node {
            assert_eq!(attrs.src, "hero.jpg");
            assert_eq!(attrs.title.as_deref(), Some("Hero"));
        } else {
            panic!("expected Image");
        }
    }

    #[test]
    fn block_to_tiptap_horizontal_rule() {
        assert!(matches!(
            block_to_tiptap(&Block::HorizontalRule),
            TiptapNode::HorizontalRule
        ));
    }

    #[test]
    fn inlines_to_tiptap_text_node() {
        let inlines = vec![Inline::Text {
            text: "world".to_string(),
            style_name: None,
            marks: vec![TiptapMark::Bold],
        }];
        let nodes = inlines_to_tiptap(&inlines);
        assert_eq!(nodes.len(), 1);
        if let TiptapNode::Text { text, marks } = &nodes[0] {
            assert_eq!(text, "world");
            assert_eq!(marks.as_ref().unwrap().len(), 1);
        } else {
            panic!("expected Text node");
        }
    }

    #[test]
    fn inlines_to_tiptap_line_break() {
        let inlines = vec![Inline::LineBreak];
        let nodes = inlines_to_tiptap(&inlines);
        assert!(matches!(nodes[0], TiptapNode::HardBreak));
    }

    #[test]
    fn inlines_to_tiptap_empty() {
        assert!(inlines_to_tiptap(&[]).is_empty());
    }

    #[test]
    fn block_to_tiptap_bullet_list() {
        let block = Block::BulletList {
            content: vec![Block::ListItem { content: vec![] }],
        };
        let node = block_to_tiptap(&block);
        if let TiptapNode::BulletList { content } = node {
            assert_eq!(content.len(), 1);
        } else {
            panic!("expected BulletList");
        }
    }
}
