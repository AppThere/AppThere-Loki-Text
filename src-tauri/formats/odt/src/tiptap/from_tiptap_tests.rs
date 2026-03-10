use std::collections::HashMap;

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
    let node = TiptapNode::Paragraph {
        attrs: None,
        content: None,
    };
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
    if let Block::Paragraph {
        style_name,
        content,
        ..
    } = block
    {
        assert_eq!(style_name.as_deref(), Some("Standard"));
        assert_eq!(content.len(), 1);
    } else {
        panic!("expected Paragraph block");
    }
}

#[test]
fn tiptap_node_to_block_heading_level() {
    let node = TiptapNode::Heading {
        attrs: Some(TiptapAttrs {
            level: Some(3),
            ..TiptapAttrs::default()
        }),
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
    let node = TiptapNode::Heading {
        attrs: None,
        content: None,
    };
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
    let node = TiptapNode::Text {
        text: "x".to_string(),
        marks: None,
    };
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
        content: vec![TiptapNode::ListItem { content: vec![] }],
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
        TiptapNode::Text {
            text: "hi".to_string(),
            marks: None,
        },
        TiptapNode::HardBreak,
        TiptapNode::Text {
            text: "there".to_string(),
            marks: None,
        },
    ];
    let inlines = tiptap_content_to_inlines(nodes);
    assert_eq!(inlines.len(), 3);
    assert!(matches!(&inlines[1], Inline::LineBreak));
}

#[test]
fn tiptap_to_document_preserves_metadata() {
    let meta = Metadata {
        title: Some("My Title".to_string()),
        ..Metadata::default()
    };
    let doc = tiptap_to_document(empty_doc(), HashMap::new(), meta);
    assert_eq!(doc.metadata.title.as_deref(), Some("My Title"));
}
