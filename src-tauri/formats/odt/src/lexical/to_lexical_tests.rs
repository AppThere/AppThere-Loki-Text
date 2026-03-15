use super::*;
use common_core::marks::{LinkAttrs, TiptapMark};

#[test]
fn empty_document_produces_root() {
    let doc = Document::new();
    let lex = to_lexical(&doc);
    assert_eq!(lex.root.node_type, "root");
    assert!(lex.root.children.is_empty());
}

#[test]
fn paragraph_becomes_paragraph_style() {
    let block = Block::Paragraph {
        style_name: Some("Body Text".to_string()),
        attrs: None,
        content: vec![],
    };
    let node = block_to_node(&block);
    if let LexicalNode::ParagraphStyle { style_name, .. } = node {
        assert_eq!(style_name, "Body Text");
    } else {
        panic!("expected ParagraphStyle");
    }
}

#[test]
fn heading_becomes_heading_style_with_tag() {
    let block = Block::Heading {
        level: 3,
        style_name: None,
        attrs: None,
        content: vec![],
    };
    let node = block_to_node(&block);
    if let LexicalNode::HeadingStyle { tag, .. } = node {
        assert_eq!(tag, "h3");
    } else {
        panic!("expected HeadingStyle");
    }
}

#[test]
fn text_with_bold_italic_sets_format() {
    let inlines = vec![Inline::Text {
        text: "hi".to_string(),
        style_name: None,
        marks: vec![TiptapMark::Bold, TiptapMark::Italic],
    }];
    let nodes = inlines_to_nodes(&inlines);
    if let LexicalNode::Text { format, .. } = &nodes[0] {
        assert_eq!(*format, FORMAT_BOLD | FORMAT_ITALIC);
    } else {
        panic!("expected Text");
    }
}

#[test]
fn link_mark_produces_link_wrapper() {
    let inlines = vec![Inline::Text {
        text: "click".to_string(),
        style_name: None,
        marks: vec![TiptapMark::Link {
            attrs: LinkAttrs {
                href: "https://example.com".to_string(),
                target: Some("_blank".to_string()),
            },
        }],
    }];
    let nodes = inlines_to_nodes(&inlines);
    if let LexicalNode::Link { url, children, .. } = &nodes[0] {
        assert_eq!(url, "https://example.com");
        assert_eq!(children.len(), 1);
    } else {
        panic!("expected Link");
    }
}

#[test]
fn line_break_becomes_linebreak_node() {
    let nodes = inlines_to_nodes(&[Inline::LineBreak]);
    assert!(matches!(nodes[0], LexicalNode::LineBreak { .. }));
}

#[test]
fn page_break_becomes_page_break_node() {
    let node = block_to_node(&Block::PageBreak);
    assert!(matches!(node, LexicalNode::PageBreak { .. }));
}
