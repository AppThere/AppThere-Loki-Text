//! Tests for the PageBreak-node suppression logic in [`from_lexical`].
//!
//! Separated from `from_lexical_tests` to keep individual test files within
//! the project's 300-line limit.

use std::collections::HashMap;

use common_core::lexical::{LexicalDocument, LexicalNode, LexicalRoot};
use common_core::{Block, Metadata, StyleDefinition};

use super::from_lexical;

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

/// A PageBreak node immediately before a paragraph whose style has
/// `fo:break-before = "page"` must be dropped — the style already provides
/// the break, so writing an explicit PageBreak paragraph as well would
/// produce a double page-break in other ODF consumers.
#[test]
fn page_break_before_break_before_style_is_dropped() {
    let mut styles = HashMap::new();
    let mut attrs = HashMap::new();
    attrs.insert("fo:break-before".to_string(), "page".to_string());
    styles.insert(
        "Heading1".to_string(),
        StyleDefinition {
            name: "Heading1".to_string(),
            family: common_core::StyleFamily::Paragraph,
            parent: None,
            next: None,
            display_name: None,
            attributes: attrs,
            text_transform: None,
            outline_level: Some(1),
            autocomplete: None,
            font_colour: None,
            background_colour: None,
        },
    );

    let lex = make_lex(vec![
        LexicalNode::PageBreak { version: 1 },
        LexicalNode::HeadingStyle {
            tag: "h1".to_string(),
            style_name: Some("Heading1".to_string()),
            children: vec![],
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        },
    ]);

    let doc = from_lexical(lex, styles, Metadata::default());
    // The PageBreak must have been dropped; only the heading should remain.
    assert_eq!(doc.blocks.len(), 1, "expected only the heading block");
    assert!(matches!(doc.blocks[0], Block::Heading { .. }));
}

/// A PageBreak node that is NOT followed by a break-before style must be
/// preserved as an explicit page break block.
#[test]
fn standalone_page_break_is_preserved() {
    let lex = make_lex(vec![
        LexicalNode::PageBreak { version: 1 },
        LexicalNode::ParagraphStyle {
            style_name: Some("Standard".to_string()),
            children: vec![],
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        },
    ]);

    let doc = from_lexical(lex, HashMap::new(), Metadata::default());
    assert_eq!(doc.blocks.len(), 2);
    assert!(matches!(doc.blocks[0], Block::PageBreak));
}

/// A PageBreak node at the end of the document (no following node) must be
/// preserved.
#[test]
fn trailing_page_break_is_preserved() {
    let lex = make_lex(vec![LexicalNode::PageBreak { version: 1 }]);
    let doc = from_lexical(lex, HashMap::new(), Metadata::default());
    assert_eq!(doc.blocks.len(), 1);
    assert!(matches!(doc.blocks[0], Block::PageBreak));
}
