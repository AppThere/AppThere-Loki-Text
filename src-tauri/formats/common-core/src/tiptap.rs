//! Tiptap/Lexical JSON node types.
//!
//! This module defines the bridge types used to communicate document content
//! between the Rust backend and the TypeScript Lexical editor frontend.
//! The [`TiptapNode`] enum mirrors Tiptap's JSON schema.
//!
//! # Examples
//!
//! ```
//! use common_core::tiptap::{TiptapNode, TiptapAttrs};
//!
//! let doc = TiptapNode::Doc {
//!     content: vec![
//!         TiptapNode::Paragraph {
//!             attrs: Some(TiptapAttrs {
//!                 style_name: Some("Standard".to_string()),
//!                 level: None,
//!                 text_align: None,
//!                 indent: None,
//!             }),
//!             content: Some(vec![
//!                 TiptapNode::Text {
//!                     text: "Hello".to_string(),
//!                     marks: None,
//!                 }
//!             ]),
//!         }
//!     ],
//! };
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::block::CellAttrs;
use crate::marks::TiptapMark;
use crate::metadata::Metadata;
use crate::style::StyleDefinition;

/// Shared attribute bag for Tiptap paragraph and heading nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TiptapAttrs {
    /// The ODT paragraph style name.
    #[serde(rename = "styleName")]
    pub style_name: Option<String>,
    /// Heading level (1-6), only applicable to heading nodes.
    pub level: Option<u32>,
    /// Text alignment (e.g. `"left"`, `"center"`, `"right"`).
    #[serde(rename = "textAlign")]
    pub text_align: Option<String>,
    /// Indentation level.
    pub indent: Option<u32>,
}

/// Image node attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageAttrs {
    /// The image source URL or embedded data URI.
    pub src: String,
    /// Alternative text description.
    pub alt: Option<String>,
    /// Image title tooltip.
    pub title: Option<String>,
}

/// A Tiptap/Lexical JSON document node.
///
/// Represents any node type in the editor's document tree.
/// Used as the serialization format exchanged with the TypeScript frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TiptapNode {
    /// The root document node.
    Doc { content: Vec<TiptapNode> },
    /// A paragraph node.
    Paragraph {
        attrs: Option<TiptapAttrs>,
        content: Option<Vec<TiptapNode>>,
    },
    /// A heading node.
    Heading {
        attrs: Option<TiptapAttrs>,
        content: Option<Vec<TiptapNode>>,
    },
    /// A text leaf node.
    Text {
        text: String,
        marks: Option<Vec<TiptapMark>>,
    },
    /// An image node.
    Image { attrs: ImageAttrs },
    /// An unordered list.
    BulletList { content: Vec<TiptapNode> },
    /// An ordered list.
    OrderedList { content: Vec<TiptapNode> },
    /// A list item.
    ListItem { content: Vec<TiptapNode> },
    /// A block quote.
    Blockquote { content: Vec<TiptapNode> },
    /// A table.
    Table { content: Vec<TiptapNode> },
    /// A table row.
    TableRow { content: Vec<TiptapNode> },
    /// A table header cell.
    TableHeader {
        attrs: Option<CellAttrs>,
        content: Vec<TiptapNode>,
    },
    /// A table data cell.
    TableCell {
        attrs: Option<CellAttrs>,
        content: Vec<TiptapNode>,
    },
    /// A horizontal rule.
    HorizontalRule,
    /// A page break.
    PageBreak,
    /// A hard line break within a paragraph.
    HardBreak,
}

/// The response payload sent to the frontend when opening a document.
///
/// Bundles the document tree, named styles, and metadata together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiptapResponse {
    /// The document content tree.
    pub content: TiptapNode,
    /// Named styles keyed by style name.
    pub styles: HashMap<String, StyleDefinition>,
    /// Document metadata.
    pub metadata: Metadata,
}
