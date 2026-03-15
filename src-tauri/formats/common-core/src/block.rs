//! Block-level document elements.
//!
//! This module defines the [`Block`] enum which represents all block-level
//! structural elements in a document: paragraphs, headings, lists, tables,
//! images, and special elements like page breaks.
//!
//! # Examples
//!
//! ```
//! use common_core::block::{Block, BlockAttrs};
//! use common_core::inline::Inline;
//!
//! let heading = Block::Heading {
//!     level: 1,
//!     style_name: Some("Heading 1".to_string()),
//!     attrs: None,
//!     content: vec![Inline::Text {
//!         text: "Introduction".to_string(),
//!         style_name: None,
//!         marks: vec![],
//!     }],
//! };
//! ```

use serde::{Deserialize, Serialize};

use crate::inline::Inline;

/// Paragraph and block alignment / indentation attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BlockAttrs {
    /// Horizontal text alignment (e.g. `"left"`, `"center"`, `"right"`).
    #[serde(rename = "textAlign")]
    pub text_align: Option<String>,
    /// Indentation level.
    pub indent: Option<u32>,
}

/// Table cell spanning attributes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CellAttrs {
    /// Number of columns this cell spans.
    pub colspan: Option<u32>,
    /// Number of rows this cell spans.
    pub rowspan: Option<u32>,
    /// Column widths for merged cells.
    pub colwidth: Option<Vec<u32>>,
}

/// A block-level element in a document.
///
/// Blocks are structural elements that contain inline content or nested blocks.
/// This type maps to Tiptap/Lexical node types for frontend compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Block {
    /// A paragraph containing inline content.
    Paragraph {
        /// The ODT paragraph style name.
        #[serde(rename = "styleName")]
        style_name: Option<String>,
        /// Optional block-level formatting attributes.
        #[serde(default)]
        attrs: Option<BlockAttrs>,
        /// The inline content of this paragraph.
        content: Vec<Inline>,
    },
    /// A heading with a specified level (1-6).
    Heading {
        /// The heading level (1 = most important).
        level: u32,
        /// The ODT paragraph style name.
        #[serde(rename = "styleName")]
        style_name: Option<String>,
        /// Optional block-level formatting attributes.
        #[serde(default)]
        attrs: Option<BlockAttrs>,
        /// The inline content of this heading.
        content: Vec<Inline>,
    },
    /// An embedded image.
    Image {
        /// The image source URL or path.
        src: String,
        /// Alternative text description.
        alt: Option<String>,
        /// Image title tooltip.
        title: Option<String>,
    },
    /// An unordered (bullet) list containing list items.
    BulletList {
        /// The list items.
        content: Vec<Block>,
    },
    /// An ordered (numbered) list containing list items.
    OrderedList {
        /// The list items.
        content: Vec<Block>,
    },
    /// A single list item.
    ListItem {
        /// The content blocks inside this list item.
        content: Vec<Block>,
    },
    /// A block quotation.
    Blockquote {
        /// The quoted content blocks.
        content: Vec<Block>,
    },
    /// A table containing rows.
    Table {
        /// The table rows.
        content: Vec<Block>,
    },
    /// A table row containing cells.
    TableRow {
        /// The cells in this row.
        content: Vec<Block>,
    },
    /// A table header cell.
    TableHeader {
        /// Optional cell spanning attributes.
        #[serde(default)]
        attrs: Option<CellAttrs>,
        /// The content blocks inside this header cell.
        content: Vec<Block>,
    },
    /// A table data cell.
    TableCell {
        /// Optional cell spanning attributes.
        #[serde(default)]
        attrs: Option<CellAttrs>,
        /// The content blocks inside this cell.
        content: Vec<Block>,
    },
    /// A horizontal rule separator.
    HorizontalRule,
    /// A page break.
    PageBreak,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inline::Inline;

    #[test]
    fn block_attrs_default() {
        let attrs = BlockAttrs::default();
        assert!(attrs.text_align.is_none());
        assert!(attrs.indent.is_none());
    }

    #[test]
    fn cell_attrs_default() {
        let attrs = CellAttrs::default();
        assert!(attrs.colspan.is_none());
        assert!(attrs.rowspan.is_none());
        assert!(attrs.colwidth.is_none());
    }

    #[test]
    fn paragraph_block_construction() {
        let block = Block::Paragraph {
            style_name: Some("Standard".to_string()),
            attrs: None,
            content: vec![Inline::Text {
                text: "Hello".to_string(),
                style_name: None,
                marks: vec![],
            }],
        };
        if let Block::Paragraph {
            style_name,
            content,
            ..
        } = &block
        {
            assert_eq!(style_name.as_deref(), Some("Standard"));
            assert_eq!(content.len(), 1);
        } else {
            panic!("expected Paragraph");
        }
    }

    #[test]
    fn heading_block_level() {
        let block = Block::Heading {
            level: 2,
            style_name: None,
            attrs: None,
            content: vec![],
        };
        if let Block::Heading { level, .. } = &block {
            assert_eq!(*level, 2);
        } else {
            panic!("expected Heading");
        }
    }

    #[test]
    fn image_block_fields() {
        let block = Block::Image {
            src: "image.png".to_string(),
            alt: Some("A picture".to_string()),
            title: None,
        };
        if let Block::Image { src, alt, title } = &block {
            assert_eq!(src, "image.png");
            assert_eq!(alt.as_deref(), Some("A picture"));
            assert!(title.is_none());
        } else {
            panic!("expected Image");
        }
    }

    #[test]
    fn horizontal_rule_and_page_break() {
        assert_eq!(Block::HorizontalRule, Block::HorizontalRule);
        assert_eq!(Block::PageBreak, Block::PageBreak);
    }

    #[test]
    fn bullet_list_nesting() {
        let inner = Block::ListItem {
            content: vec![Block::Paragraph {
                style_name: None,
                attrs: None,
                content: vec![],
            }],
        };
        let list = Block::BulletList {
            content: vec![inner],
        };
        if let Block::BulletList { content } = &list {
            assert_eq!(content.len(), 1);
        } else {
            panic!("expected BulletList");
        }
    }

    #[test]
    fn block_serde_roundtrip() {
        let block = Block::Heading {
            level: 1,
            style_name: Some("Heading 1".to_string()),
            attrs: Some(BlockAttrs {
                text_align: Some("center".to_string()),
                indent: None,
            }),
            content: vec![],
        };
        let json = serde_json::to_string(&block).unwrap();
        let decoded: Block = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, block);
    }
}
