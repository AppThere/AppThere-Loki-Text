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
