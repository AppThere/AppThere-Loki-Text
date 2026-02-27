use crate::*;
use std::collections::HashMap;

#[test]
fn test_to_tiptap_conversion() {
    let mut styles = HashMap::new();
    styles.insert(
        "Standard".to_string(),
        StyleDefinition {
            name: "Standard".to_string(),
            family: StyleFamily::Paragraph,
            parent: None,
            next: None,
            display_name: None,
            attributes: HashMap::new(),
            text_transform: None,
            outline_level: None,
            autocomplete: None,
        },
    );

    let doc = Document {
        blocks: vec![
            Block::Heading {
                level: 1,
                style_name: Some("Heading 1".to_string()),
                attrs: None,
                content: vec![Inline::Text {
                    text: "Hello World".to_string(),
                    style_name: None,
                    marks: vec![],
                }],
            },
            Block::Paragraph {
                style_name: Some("Standard".to_string()),
                attrs: None,
                content: vec![
                    Inline::Text {
                        text: "This is a ".to_string(),
                        style_name: None,
                        marks: vec![],
                    },
                    Inline::Text {
                        text: "bold".to_string(),
                        style_name: None,
                        marks: vec![TiptapMark::Bold],
                    },
                    Inline::Text {
                        text: " statement.".to_string(),
                        style_name: None,
                        marks: vec![],
                    },
                ],
            },
        ],
        styles,
        metadata: Metadata::default(),
        font_face_decls: None,
        automatic_styles: None,
        master_styles: None,
    };

    let tiptap = doc.to_tiptap();

    if let TiptapNode::Doc { content } = tiptap {
        assert_eq!(content.len(), 2);

        // Check Heading
        if let TiptapNode::Heading {
            attrs: Some(attrs),
            content: Some(c),
        } = &content[0]
        {
            assert_eq!(attrs.level, Some(1));
            assert_eq!(c.len(), 1);
            if let TiptapNode::Text { text, .. } = &c[0] {
                assert_eq!(text, "Hello World");
            } else {
                panic!("Expected Text node");
            }
        } else {
            panic!("Expected Heading node, got {:?}", &content[0]);
        }

        // Check Paragraph
        if let TiptapNode::Paragraph {
            content: Some(c), ..
        } = &content[1]
        {
            assert_eq!(c.len(), 3);
            // Check bold mark
            if let TiptapNode::Text { text, marks } = &c[1] {
                assert_eq!(text, "bold");
                assert!(marks.is_some());
                assert!(marks.as_ref().unwrap().contains(&TiptapMark::Bold));
            } else {
                panic!("Expected Text node for bold");
            }
        } else {
            panic!("Expected Paragraph node");
        }
    } else {
        panic!("Expected Doc node");
    }
}

#[test]
fn test_metadata_persistence() {
    let metadata = Metadata {
        identifier: Some("uuid-123".to_string()),
        title: Some("My Document".to_string()),
        language: Some("fr".to_string()),
        description: Some("A test document".to_string()),
        subject: Some("Testing".to_string()),
        creator: Some("John Doe".to_string()),
        creation_date: Some("2023-01-01T00:00:00Z".to_string()),
        generator: Some("Loki Test".to_string()),
    };

    let doc = Document {
        blocks: vec![],
        styles: HashMap::new(),
        metadata,
        font_face_decls: None,
        automatic_styles: None,
        master_styles: None,
    };

    // Test ODT Meta XML
    let meta_xml = doc.to_meta_xml().unwrap();
    let doc2 = Document::from_xml(&meta_xml).unwrap();
    assert_eq!(doc.metadata.title, doc2.metadata.title);
    assert_eq!(doc.metadata.description, doc2.metadata.description);
    assert_eq!(doc.metadata.subject, doc2.metadata.subject);
    assert_eq!(doc.metadata.creator, doc2.metadata.creator);
    assert_eq!(doc.metadata.language, doc2.metadata.language);
    assert_eq!(doc.metadata.identifier, doc2.metadata.identifier);
    assert_eq!(doc.metadata.creation_date, doc2.metadata.creation_date);

    // Test FODT XML
    let fodt_xml = doc.to_xml().unwrap();
    let doc3 = Document::from_xml(&fodt_xml).unwrap();
    assert_eq!(doc.metadata.title, doc3.metadata.title);
    assert_eq!(doc.metadata.description, doc3.metadata.description);
    assert_eq!(doc.metadata.subject, doc3.metadata.subject);
    assert_eq!(doc.metadata.creator, doc3.metadata.creator);
    assert_eq!(doc.metadata.language, doc3.metadata.language);
    assert_eq!(doc.metadata.identifier, doc3.metadata.identifier);
    assert_eq!(doc.metadata.creation_date, doc3.metadata.creation_date);

    // Test update_fodt
    let old_xml = r#"<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0"><office:meta><dc:title>Old Title</dc:title></office:meta><office:body><office:text></office:text></office:body></office:document>"#;

    let updated_xml = doc.update_fodt(old_xml).unwrap();
    let doc4 = Document::from_xml(&updated_xml).unwrap();
    assert_eq!(doc.metadata.title, doc4.metadata.title);
    assert_eq!(doc.metadata.description, doc4.metadata.description);
}
