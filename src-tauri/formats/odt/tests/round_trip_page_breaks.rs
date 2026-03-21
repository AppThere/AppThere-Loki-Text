//! Round-trip tests specifically for style-based page breaks.
//!
//! Separated from `round_trip` to keep individual test files within the
//! project's 300-line limit.

use common_core::Block;
use odt_format::{
    lexical::{from_lexical, to_lexical},
    parser::parse_document,
};

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";

fn fodt(styles: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}" office:version="1.3">
  <office:styles>{styles}</office:styles>
  <office:body>
    <office:text>{body}</office:text>
  </office:body>
</office:document>"#
    )
}

fn round_trip(xml: &str) -> (Vec<Block>, Vec<Block>) {
    let doc1 = parse_document(xml).expect("initial parse failed");
    let lex = to_lexical(&doc1);
    let doc2 = from_lexical(lex, doc1.styles.clone(), doc1.metadata.clone());
    (doc1.blocks, doc2.blocks)
}

/// A paragraph style that has `fo:break-before = "page"` should survive the
/// full round-trip without sprouting an extra explicit PageBreak block.
#[test]
fn style_with_break_before_round_trips_without_extra_page_break() {
    let style_xml = format!(
        r#"<style:style style:name="Chapter" style:family="paragraph"
                xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}">
            <style:paragraph-properties fo:break-before="page"/>
        </style:style>"#
    );
    let body = format!(
        r#"<text:p xmlns:text="{NS_TEXT}" text:style-name="Chapter">Chapter 1</text:p>"#
    );
    let (b1, b2) = round_trip(&fodt(&style_xml, &body));

    // The source and round-tripped documents must both contain exactly one
    // block (the paragraph) — no extra PageBreak blocks.
    assert_eq!(b1.len(), 1, "source must have 1 block");
    assert_eq!(b2.len(), 1, "round-tripped must have 1 block");
    assert!(matches!(b1[0], Block::Paragraph { .. }));
    assert!(matches!(b2[0], Block::Paragraph { .. }));
}

/// The Lexical representation of a document with a break-before style must
/// contain a PageBreak node before the styled paragraph.
#[test]
fn style_with_break_before_produces_page_break_node_in_lexical() {
    use common_core::lexical::LexicalNode;

    let style_xml = format!(
        r#"<style:style style:name="Chapter" style:family="paragraph"
                xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}">
            <style:paragraph-properties fo:break-before="page"/>
        </style:style>"#
    );
    let body = format!(
        r#"<text:p xmlns:text="{NS_TEXT}" text:style-name="Chapter">Intro</text:p>"#
    );
    let doc = parse_document(&fodt(&style_xml, &body)).unwrap();
    let lex = to_lexical(&doc);

    assert_eq!(
        lex.root.children.len(),
        2,
        "expected [PageBreak, ParagraphStyle]"
    );
    assert!(
        matches!(lex.root.children[0], LexicalNode::PageBreak { .. }),
        "first node must be PageBreak"
    );
}
