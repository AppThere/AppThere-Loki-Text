use std::collections::HashMap;

use common_core::lexical::{LexicalDocument, LexicalNode, LexicalRoot, FORMAT_BOLD, FORMAT_ITALIC};
use common_core::marks::TiptapMark;
use common_core::{Block, Inline, Metadata};

use super::{from_lexical, node_to_block, node_to_inlines, decode_format};

fn make_lex(children: Vec<LexicalNode>) -> LexicalDocument {
    LexicalDocument {
        root: LexicalRoot {
            children,
            direction: None,
            format: String::new(),
            indent: 0,
            node_type: "root".to_string(),
            version: 1,
        },
    }
}

#[test]
fn empty_document() {
    let doc = from_lexical(make_lex(vec![]), HashMap::new(), Metadata::default());
    assert!(doc.blocks.is_empty());
}

#[test]
fn paragraph_style_to_paragraph_block() {
    let lex = make_lex(vec![LexicalNode::ParagraphStyle {
        style_name: "Standard".to_string(),
        children: vec![LexicalNode::Text {
            text: "Hello".to_string(),
            format: 0,
            style: String::new(),
            mode: "normal".to_string(),
            detail: 0,
            style_name: None,
            version: 1,
        }],
        direction: None,
        format: String::new(),
        indent: 0,
        version: 1,
    }]);
    let doc = from_lexical(lex, HashMap::new(), Metadata::default());
    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { style_name, content, .. } = &doc.blocks[0] {
        assert_eq!(style_name.as_deref(), Some("Standard"));
        assert_eq!(content.len(), 1);
    } else {
        panic!("expected Paragraph");
    }
}

#[test]
fn heading_style_level_extracted_from_tag() {
    let node = LexicalNode::HeadingStyle {
        tag: "h2".to_string(),
        style_name: None,
        children: vec![],
        direction: None,
        format: String::new(),
        indent: 0,
        version: 1,
    };
    if let Some(Block::Heading { level, .. }) = node_to_block(node) {
        assert_eq!(level, 2);
    } else {
        panic!("expected Heading");
    }
}

#[test]
fn bold_italic_format_decodes_to_marks() {
    let format = FORMAT_BOLD | FORMAT_ITALIC;
    let marks = decode_format(format, None);
    assert!(marks.contains(&TiptapMark::Bold));
    assert!(marks.contains(&TiptapMark::Italic));
    assert!(!marks.contains(&TiptapMark::Underline));
}

#[test]
fn link_node_becomes_link_mark_on_text() {
    let link_node = LexicalNode::Link {
        url: "https://example.com".to_string(),
        target: Some("_blank".to_string()),
        rel: None,
        children: vec![LexicalNode::Text {
            text: "visit".to_string(),
            format: FORMAT_BOLD,
            style: String::new(),
            mode: "normal".to_string(),
            detail: 0,
            style_name: None,
            version: 1,
        }],
        direction: None,
        format: String::new(),
        indent: 0,
        version: 1,
    };
    let inlines = node_to_inlines(link_node);
    assert_eq!(inlines.len(), 1);
    if let Inline::Text { text, marks, .. } = &inlines[0] {
        assert_eq!(text, "visit");
        let has_link = marks.iter().any(|m| matches!(m, TiptapMark::Link { .. }));
        let has_bold = marks.contains(&TiptapMark::Bold);
        assert!(has_link, "expected Link mark");
        assert!(has_bold, "expected Bold mark");
    }
}

#[test]
fn page_break_node_maps_to_page_break_block() {
    let node = LexicalNode::PageBreak { version: 1 };
    assert!(matches!(node_to_block(node), Some(Block::PageBreak)));
}

#[test]
fn list_type_number_produces_ordered_list() {
    let node = LexicalNode::List {
        list_type: "number".to_string(),
        start: 1,
        tag: "ol".to_string(),
        children: vec![],
        direction: None,
        format: String::new(),
        indent: 0,
        version: 1,
    };
    assert!(matches!(node_to_block(node), Some(Block::OrderedList { .. })));
}

#[test]
fn table_cell_header_state_selects_variant() {
    let header = LexicalNode::TableCell {
        col_span: 1,
        row_span: 1,
        header_state: 1,
        children: vec![],
        direction: None,
        format: String::new(),
        indent: 0,
        version: 1,
    };
    assert!(matches!(node_to_block(header), Some(Block::TableHeader { .. })));

    let cell = LexicalNode::TableCell {
        col_span: 2,
        row_span: 1,
        header_state: 0,
        children: vec![],
        direction: None,
        format: String::new(),
        indent: 0,
        version: 1,
    };
    if let Some(Block::TableCell { attrs, .. }) = node_to_block(cell) {
        assert_eq!(attrs.unwrap().colspan, Some(2));
    } else {
        panic!("expected TableCell");
    }
}
