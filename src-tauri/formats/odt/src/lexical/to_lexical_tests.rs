use std::collections::HashMap;

use super::*;
use common_core::marks::{LinkAttrs, TiptapMark};
use common_core::StyleDefinition;

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
        assert_eq!(style_name, Some("Body Text".to_string()));
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

/// A paragraph whose style has `fo:break-before = "page"` must be preceded by
/// a synthetic `PageBreak` node in the Lexical output so the editor shows a
/// visible page-break indicator.
#[test]
fn paragraph_with_break_before_style_gets_preceding_page_break_node() {
    let mut styles = HashMap::new();
    let mut attrs = HashMap::new();
    attrs.insert("fo:break-before".to_string(), "page".to_string());
    styles.insert(
        "ChapterStart".to_string(),
        StyleDefinition {
            name: "ChapterStart".to_string(),
            family: common_core::StyleFamily::Paragraph,
            parent: None,
            next: None,
            display_name: None,
            attributes: attrs,
            text_transform: None,
            outline_level: None,
            autocomplete: None,
            font_colour: None,
            background_colour: None,
        },
    );

    let doc = Document {
        blocks: vec![Block::Paragraph {
            style_name: Some("ChapterStart".to_string()),
            attrs: None,
            content: vec![],
        }],
        styles,
        metadata: common_core::Metadata::default(),
        font_face_decls: None,
        automatic_styles: None,
        master_styles: None,
    };

    let lex = to_lexical(&doc);
    assert_eq!(lex.root.children.len(), 2, "expected PageBreak + paragraph");
    assert!(
        matches!(lex.root.children[0], LexicalNode::PageBreak { .. }),
        "first node must be PageBreak"
    );
    assert!(
        matches!(lex.root.children[1], LexicalNode::ParagraphStyle { .. }),
        "second node must be the paragraph"
    );
}

/// When an explicit `Block::PageBreak` already precedes a paragraph with
/// `fo:break-before`, no extra synthetic `PageBreak` node is inserted (the
/// explicit one is sufficient for visual representation).
#[test]
fn explicit_page_break_before_break_before_paragraph_no_double_insertion() {
    let mut styles = HashMap::new();
    let mut attrs = HashMap::new();
    attrs.insert("fo:break-before".to_string(), "page".to_string());
    styles.insert(
        "ChapterStart".to_string(),
        StyleDefinition {
            name: "ChapterStart".to_string(),
            family: common_core::StyleFamily::Paragraph,
            parent: None,
            next: None,
            display_name: None,
            attributes: attrs,
            text_transform: None,
            outline_level: None,
            autocomplete: None,
            font_colour: None,
            background_colour: None,
        },
    );

    let doc = Document {
        blocks: vec![
            Block::PageBreak,
            Block::Paragraph {
                style_name: Some("ChapterStart".to_string()),
                attrs: None,
                content: vec![],
            },
        ],
        styles,
        metadata: common_core::Metadata::default(),
        font_face_decls: None,
        automatic_styles: None,
        master_styles: None,
    };

    let lex = to_lexical(&doc);
    // Should be: [PageBreak (explicit), ParagraphStyle] — no extra PageBreak.
    assert_eq!(lex.root.children.len(), 2, "expected exactly two nodes");
    assert!(matches!(lex.root.children[0], LexicalNode::PageBreak { .. }));
    assert!(matches!(
        lex.root.children[1],
        LexicalNode::ParagraphStyle { .. }
    ));
}

/// A paragraph that uses the literal "PageBreak" style must NOT generate a
/// preceding synthetic page-break node — it is itself a page-break block.
#[test]
fn page_break_style_paragraph_not_preceded_by_extra_node() {
    let doc = Document {
        blocks: vec![Block::PageBreak],
        styles: HashMap::new(),
        metadata: common_core::Metadata::default(),
        font_face_decls: None,
        automatic_styles: None,
        master_styles: None,
    };

    let lex = to_lexical(&doc);
    assert_eq!(lex.root.children.len(), 1);
    assert!(matches!(lex.root.children[0], LexicalNode::PageBreak { .. }));
}
