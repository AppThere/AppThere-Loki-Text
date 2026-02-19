use super::*;
use std::collections::HashMap;

fn create_mock_tiptap_doc() -> TiptapNode {
    TiptapNode::Doc {
        content: vec![
            TiptapNode::Heading {
                attrs: Some(TiptapAttrs {
                    level: Some(1),
                    ..Default::default()
                }),
                content: Some(vec![TiptapNode::Text {
                    text: "Chapter 1".to_string(),
                    marks: None,
                }]),
            },
            TiptapNode::Paragraph {
                attrs: None,
                content: Some(vec![TiptapNode::Text {
                    text: "Hello World".to_string(),
                    marks: None,
                }]),
            },
        ],
    }
}

#[test]
fn test_epub_from_tiptap() {
    let styles = HashMap::new();
    let metadata = Metadata::default();
    let fonts = Vec::new();

    let epub = EpubDocument::from_tiptap(create_mock_tiptap_doc(), styles, metadata, fonts);

    // Should have at least one section
    assert!(!epub.sections.is_empty());

    // Check first section content
    let section = &epub.sections[0];
    assert_eq!(section.blocks.len(), 2);

    match &section.blocks[0] {
        Block::Heading { level, .. } => assert_eq!(*level, 1),
        _ => panic!("Expected heading"),
    }
}

#[test]
fn test_block_to_html() {
    let doc = EpubDocument {
        sections: vec![],
        styles: HashMap::new(),
        metadata: Metadata::default(),
        fonts: vec![],
    };

    let block = Block::Paragraph {
        style_name: None,
        attrs: None,
        content: vec![Inline::Text {
            text: "test".to_string(),
            style_name: None,
            marks: vec![TiptapMark::Bold],
        }],
    };

    let html = doc.block_to_html(&block);
    assert!(html.contains("<p>"));
    assert!(html.contains("<strong>test</strong>"));
    assert!(html.contains("</p>"));
}

#[test]
fn test_opf_generation() {
    let mut metadata = Metadata::default();
    metadata.title = Some("My Book".to_string());
    metadata.creator = Some("Author".to_string());

    let doc = EpubDocument {
        sections: vec![ContentSection {
            id: "sec1".to_string(),
            title: Some("Ch1".to_string()),
            blocks: vec![],
        }],
        styles: HashMap::new(),
        metadata,
        fonts: vec![],
    };

    let opf = doc.to_package_opf();
    assert!(opf.contains("<dc:title>My Book</dc:title>"));
    assert!(opf.contains("<dc:creator>Author</dc:creator>"));
    assert!(opf.contains("href=\"Text/sec1.xhtml\""));
}

#[test]
fn test_page_break_splitting() {
    let mut styles = HashMap::new();
    let mut style_def = StyleDefinition {
        name: "BreakStyle".to_string(),
        family: odt_logic::StyleFamily::Paragraph,
        parent: None,
        next: None,
        display_name: None,
        attributes: HashMap::new(),
        text_transform: None,
        outline_level: None,
        autocomplete: None,
    };
    style_def
        .attributes
        .insert("fo:break-before".to_string(), "page".to_string());
    styles.insert("BreakStyle".to_string(), style_def);

    let root = TiptapNode::Doc {
        content: vec![
            TiptapNode::Paragraph {
                attrs: None,
                content: Some(vec![TiptapNode::Text {
                    text: "P1".to_string(),
                    marks: None,
                }]),
            },
            TiptapNode::Paragraph {
                attrs: Some(TiptapAttrs {
                    style_name: Some("BreakStyle".to_string()),
                    ..Default::default()
                }),
                content: Some(vec![TiptapNode::Text {
                    text: "P2".to_string(),
                    marks: None,
                }]),
            },
        ],
    };

    let epub = EpubDocument::from_tiptap(root, styles, Metadata::default(), vec![]);
    // Should be split into 2 sections because of break-before
    assert_eq!(epub.sections.len(), 2);
}
