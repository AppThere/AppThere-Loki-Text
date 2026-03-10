//! Integration tests for odt-format: parse → tiptap → write round-trips.
//!
//! These tests exercise the full pipeline using minimal but realistic
//! ODT/FODT XML fixtures, verifying that content survives the round-trip
//! without loss.

use std::collections::HashMap;

use common_core::{Block, Inline, Metadata, TiptapNode};
use odt_format::parser::parse_document;
use odt_format::tiptap::from_tiptap::tiptap_to_document;
use odt_format::tiptap::to_tiptap::document_to_tiptap;
use odt_format::writer::fodt::to_xml;
use odt_format::writer::meta::to_meta_xml;

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
const NS_META: &str = "urn:oasis:names:tc:opendocument:xmlns:meta:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";

fn fodt_with_body(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:dc="{NS_DC}" xmlns:meta="{NS_META}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}"
    office:version="1.3">
  <office:body>
    <office:text>{body}</office:text>
  </office:body>
</office:document>"#
    )
}

// ── Parser tests ─────────────────────────────────────────────────────────────

#[test]
fn parse_single_paragraph() {
    let xml = fodt_with_body(r#"<text:p>Simple paragraph</text:p>"#);
    let doc = parse_document(&xml).unwrap();
    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        assert_eq!(content.len(), 1);
        if let Inline::Text { text, .. } = &content[0] {
            assert_eq!(text, "Simple paragraph");
        } else {
            panic!("expected Text inline");
        }
    } else {
        panic!("expected Paragraph block");
    }
}

#[test]
fn parse_multiple_paragraphs() {
    let xml = fodt_with_body(
        r#"<text:p>First</text:p><text:p>Second</text:p><text:p>Third</text:p>"#,
    );
    let doc = parse_document(&xml).unwrap();
    assert_eq!(doc.blocks.len(), 3);
}

#[test]
fn parse_heading() {
    let xml = fodt_with_body(r#"<text:h text:outline-level="2">My Heading</text:h>"#);
    let doc = parse_document(&xml).unwrap();
    assert_eq!(doc.blocks.len(), 1);
    if let Block::Heading { level, content, .. } = &doc.blocks[0] {
        assert_eq!(*level, 2);
        assert_eq!(content.len(), 1);
    } else {
        panic!("expected Heading block");
    }
}

#[test]
fn parse_empty_paragraph() {
    let xml = fodt_with_body(r#"<text:p></text:p>"#);
    let doc = parse_document(&xml).unwrap();
    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        assert!(content.is_empty());
    }
}

#[test]
fn parse_metadata_title_and_creator() {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:dc="{NS_DC}" xmlns:meta="{NS_META}" office:version="1.3">
  <office:meta>
    <dc:title>Integration Test</dc:title>
    <dc:creator>Test Suite</dc:creator>
    <dc:language>en-US</dc:language>
  </office:meta>
  <office:body>
    <office:text/>
  </office:body>
</office:document>"#
    );
    let doc = parse_document(&xml).unwrap();
    assert_eq!(doc.metadata.title.as_deref(), Some("Integration Test"));
    assert_eq!(doc.metadata.creator.as_deref(), Some("Test Suite"));
    assert_eq!(doc.metadata.language.as_deref(), Some("en-US"));
}

// ── Block ↔ Tiptap round-trip tests ─────────────────────────────────────────

#[test]
fn blocks_to_tiptap_and_back_paragraph() {
    let original = vec![Block::Paragraph {
        style_name: Some("Standard".to_string()),
        attrs: None,
        content: vec![Inline::Text {
            text: "Round-trip text".to_string(),
            style_name: None,
            marks: vec![],
        }],
    }];

    let tiptap = document_to_tiptap(&original);
    let doc = tiptap_to_document(tiptap, HashMap::new(), Metadata::default());

    assert_eq!(doc.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &doc.blocks[0] {
        if let Inline::Text { text, .. } = &content[0] {
            assert_eq!(text, "Round-trip text");
        }
    }
}

#[test]
fn blocks_to_tiptap_and_back_heading() {
    let original = vec![Block::Heading {
        level: 1,
        style_name: None,
        attrs: None,
        content: vec![Inline::Text {
            text: "Title".to_string(),
            style_name: None,
            marks: vec![],
        }],
    }];

    let tiptap = document_to_tiptap(&original);
    let doc = tiptap_to_document(tiptap, HashMap::new(), Metadata::default());

    if let Block::Heading { level, content, .. } = &doc.blocks[0] {
        assert_eq!(*level, 1);
        assert_eq!(content.len(), 1);
    } else {
        panic!("expected Heading");
    }
}

#[test]
fn blocks_to_tiptap_and_back_horizontal_rule() {
    let original = vec![Block::HorizontalRule];
    let tiptap = document_to_tiptap(&original);
    let doc = tiptap_to_document(tiptap, HashMap::new(), Metadata::default());
    assert!(matches!(doc.blocks[0], Block::HorizontalRule));
}

#[test]
fn blocks_to_tiptap_doc_node_wraps_content() {
    let original = vec![Block::PageBreak, Block::HorizontalRule];
    let tiptap = document_to_tiptap(&original);
    if let TiptapNode::Doc { content } = tiptap {
        assert_eq!(content.len(), 2);
        assert!(matches!(content[0], TiptapNode::PageBreak));
        assert!(matches!(content[1], TiptapNode::HorizontalRule));
    } else {
        panic!("expected Doc node");
    }
}

// ── Parser → tiptap → back full pipeline ────────────────────────────────────

#[test]
fn full_pipeline_fodt_paragraph() {
    let xml = fodt_with_body(r#"<text:p>Pipeline test</text:p>"#);
    let parsed = parse_document(&xml).unwrap();

    let tiptap = document_to_tiptap(&parsed.blocks);
    let reconstructed =
        tiptap_to_document(tiptap, parsed.styles.clone(), parsed.metadata.clone());

    assert_eq!(reconstructed.blocks.len(), 1);
    if let Block::Paragraph { content, .. } = &reconstructed.blocks[0] {
        if let Inline::Text { text, .. } = &content[0] {
            assert_eq!(text, "Pipeline test");
        } else {
            panic!("expected Text inline");
        }
    } else {
        panic!("expected Paragraph block");
    }
}

// ── Writer tests ─────────────────────────────────────────────────────────────

#[test]
fn to_xml_empty_document_produces_valid_fodt() {
    let xml = to_xml(&[], &HashMap::new(), &Metadata::default(), &None, &None, &None).unwrap();
    assert!(xml.contains("office:document"));
    assert!(xml.contains("office:body"));
}

#[test]
fn to_xml_with_paragraph() {
    let blocks = vec![Block::Paragraph {
        style_name: Some("Standard".to_string()),
        attrs: None,
        content: vec![Inline::Text {
            text: "Written content".to_string(),
            style_name: None,
            marks: vec![],
        }],
    }];
    let xml = to_xml(&blocks, &HashMap::new(), &Metadata::default(), &None, &None, &None)
        .unwrap();
    assert!(xml.contains("Written content"));
    assert!(xml.contains("text:p"));
}

#[test]
fn to_meta_xml_round_trip_title() {
    let meta = Metadata {
        title: Some("Round-trip Title".to_string()),
        creator: Some("Author".to_string()),
        ..Metadata::default()
    };
    let xml = to_meta_xml(&meta).unwrap();
    // Re-parse the generated meta.xml as a document-meta
    let reparsed = parse_document(&xml).unwrap();
    assert_eq!(reparsed.metadata.title.as_deref(), Some("Round-trip Title"));
    assert_eq!(reparsed.metadata.creator.as_deref(), Some("Author"));
}
