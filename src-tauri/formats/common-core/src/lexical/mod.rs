//! Lexical editor JSON types.
//!
//! This module defines Rust types that match the JSON serialization format
//! used by the Lexical editor running in the TypeScript frontend. Converters
//! between these types and the internal [`Block`]/[`Inline`] model live in
//! the `odt-format` crate.
//!
//! The type names mirror the custom Lexical node registrations in the
//! frontend (e.g. `paragraph-style`, `heading-style`).
//!
//! # Examples
//!
//! ```
//! use common_core::lexical::{LexicalDocument, LexicalRoot, LexicalNode};
//!
//! let doc = LexicalDocument {
//!     root: LexicalRoot {
//!         children: vec![],
//!         direction: None,
//!         format: String::new(),
//!         indent: 0,
//!         node_type: "root".to_string(),
//!         version: 1,
//!     },
//! };
//! assert_eq!(doc.root.node_type, "root");
//! ```

mod node;

pub use node::LexicalNode;

use serde::{Deserialize, Serialize};

// Text-format bitmask values (matches Lexical's IS_* constants).
pub const FORMAT_BOLD: u32 = 1;
pub const FORMAT_ITALIC: u32 = 2;
pub const FORMAT_STRIKETHROUGH: u32 = 4;
pub const FORMAT_UNDERLINE: u32 = 8;
pub const FORMAT_SUBSCRIPT: u32 = 32;
pub const FORMAT_SUPERSCRIPT: u32 = 64;

/// The top-level Lexical editor state sent over the IPC bridge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LexicalDocument {
    /// The root container node.
    pub root: LexicalRoot,
}

/// The Lexical root node (type `"root"`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LexicalRoot {
    /// All top-level block nodes.
    pub children: Vec<LexicalNode>,
    /// Text direction (`"ltr"`, `"rtl"`, or `null`).
    pub direction: Option<String>,
    /// Block alignment format string.
    pub format: String,
    /// Indent level.
    pub indent: u32,
    /// Always `"root"`.
    #[serde(rename = "type")]
    pub node_type: String,
    /// Serialization version (always `1`).
    pub version: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_doc(children: Vec<LexicalNode>) -> LexicalDocument {
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
    fn empty_document_serde_roundtrip() {
        let doc = make_doc(vec![]);
        let json = serde_json::to_string(&doc).unwrap();
        let decoded: LexicalDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, doc);
    }

    #[test]
    fn paragraph_style_serde_roundtrip() {
        let node = LexicalNode::ParagraphStyle {
            style_name: "Standard".to_string(),
            children: vec![LexicalNode::Text {
                text: "Hello".to_string(),
                format: FORMAT_BOLD,
                style: String::new(),
                mode: "normal".to_string(),
                detail: 0,
                style_name: None,
                version: 1,
            }],
            direction: Some("ltr".to_string()),
            format: String::new(),
            indent: 0,
            version: 1,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"paragraph-style\""));
        let decoded: LexicalNode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, node);
    }

    #[test]
    fn heading_style_serde_roundtrip() {
        let node = LexicalNode::HeadingStyle {
            tag: "h2".to_string(),
            style_name: Some("Heading 2".to_string()),
            children: vec![],
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"heading-style\""));
        let decoded: LexicalNode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, node);
    }

    #[test]
    fn text_format_bitmask_bold_italic() {
        let format = FORMAT_BOLD | FORMAT_ITALIC;
        assert_eq!(format, 3);
        assert!(format & FORMAT_BOLD != 0);
        assert!(format & FORMAT_ITALIC != 0);
        assert!(format & FORMAT_UNDERLINE == 0);
    }

    #[test]
    fn page_break_serde() {
        let node = LexicalNode::PageBreak { version: 1 };
        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"page-break\""));
    }

    #[test]
    fn link_node_serde_roundtrip() {
        let node = LexicalNode::Link {
            url: "https://example.com".to_string(),
            target: Some("_blank".to_string()),
            rel: Some("noopener noreferrer".to_string()),
            children: vec![LexicalNode::Text {
                text: "click here".to_string(),
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
        };
        let json = serde_json::to_string(&node).unwrap();
        let decoded: LexicalNode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, node);
    }
}
