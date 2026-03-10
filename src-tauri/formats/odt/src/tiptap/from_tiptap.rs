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

#[cfg(test)]
mod tests {
    use super::*;
    use common_core::tiptap::{ImageAttrs, TiptapAttrs};

    fn empty_doc() -> TiptapNode {
        TiptapNode::Doc { content: vec![] }
    }

    #[test]
    fn tiptap_to_document_empty_doc() {
        let doc = tiptap_to_document(empty_doc(), HashMap::new(), Metadata::default());
        assert!(doc.blocks.is_empty());
        assert!(doc.styles.is_empty());
    }

    #[test]
    fn tiptap_to_document_non_doc_root_is_empty() {
        // Passing a non-Doc variant as root should yield an empty block list
        let node = TiptapNode::Paragraph { attrs: None, content: None };
        let doc = tiptap_to_document(node, HashMap::new(), Metadata::default());
        assert!(doc.blocks.is_empty());
    }

    #[test]
    fn tiptap_node_to_block_paragraph() {
        let node = TiptapNode::Paragraph {
            attrs: Some(TiptapAttrs {
                style_name: Some("Standard".to_string()),
                ..TiptapAttrs::default()
            }),
            content: Some(vec![TiptapNode::Text {
                text: "hello".to_string(),
                marks: None,
            }]),
        };
        let block = tiptap_node_to_block(node).unwrap();
        if let Block::Paragraph { style_name, content, .. } = block {
            assert_eq!(style_name.as_deref(), Some("Standard"));
            assert_eq!(content.len(), 1);
        } else {
            panic!("expected Paragraph block");
        }
    }

    #[test]
    fn tiptap_node_to_block_heading_level() {
        let node = TiptapNode::Heading {
            attrs: Some(TiptapAttrs { level: Some(3), ..TiptapAttrs::default() }),
            content: None,
        };
        let block = tiptap_node_to_block(node).unwrap();
        if let Block::Heading { level, .. } = block {
            assert_eq!(level, 3);
        } else {
            panic!("expected Heading block");
        }
    }

    #[test]
    fn tiptap_node_to_block_heading_default_level_one() {
        let node = TiptapNode::Heading { attrs: None, content: None };
        let block = tiptap_node_to_block(node).unwrap();
        if let Block::Heading { level, .. } = block {
            assert_eq!(level, 1);
        } else {
            panic!("expected Heading block");
        }
    }

    #[test]
    fn tiptap_node_to_block_image() {
        let node = TiptapNode::Image {
            attrs: ImageAttrs {
                src: "img.png".to_string(),
                alt: Some("alt".to_string()),
                title: None,
            },
        };
        let block = tiptap_node_to_block(node).unwrap();
        if let Block::Image { src, alt, .. } = block {
            assert_eq!(src, "img.png");
            assert_eq!(alt.as_deref(), Some("alt"));
        } else {
            panic!("expected Image block");
        }
    }

    #[test]
    fn tiptap_node_to_block_text_returns_none() {
        let node = TiptapNode::Text { text: "x".to_string(), marks: None };
        assert!(tiptap_node_to_block(node).is_none());
    }

    #[test]
    fn tiptap_node_to_block_hard_break_returns_none() {
        assert!(tiptap_node_to_block(TiptapNode::HardBreak).is_none());
    }

    #[test]
    fn tiptap_node_to_block_horizontal_rule() {
        let block = tiptap_node_to_block(TiptapNode::HorizontalRule).unwrap();
        assert!(matches!(block, Block::HorizontalRule));
    }

    #[test]
    fn tiptap_node_to_block_page_break() {
        let block = tiptap_node_to_block(TiptapNode::PageBreak).unwrap();
        assert!(matches!(block, Block::PageBreak));
    }

    #[test]
    fn tiptap_node_to_block_bullet_list() {
        let node = TiptapNode::BulletList {
            content: vec![TiptapNode::ListItem {
                content: vec![],
            }],
        };
        let block = tiptap_node_to_block(node).unwrap();
        if let Block::BulletList { content } = block {
            assert_eq!(content.len(), 1);
        } else {
            panic!("expected BulletList");
        }
    }

    #[test]
    fn tiptap_content_to_inlines_text_and_break() {
        let nodes = vec![
            TiptapNode::Text { text: "hi".to_string(), marks: None },
            TiptapNode::HardBreak,
            TiptapNode::Text { text: "there".to_string(), marks: None },
        ];
        let inlines = tiptap_content_to_inlines(nodes);
        assert_eq!(inlines.len(), 3);
        assert!(matches!(&inlines[1], Inline::LineBreak));
    }

    #[test]
    fn tiptap_to_document_preserves_metadata() {
        let meta = Metadata { title: Some("My Title".to_string()), ..Metadata::default() };
        let doc = tiptap_to_document(empty_doc(), HashMap::new(), meta);
        assert_eq!(doc.metadata.title.as_deref(), Some("My Title"));
    }
}
