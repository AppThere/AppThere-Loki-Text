//! The [`LexicalNode`] enum and its serde helpers.

use serde::{Deserialize, Serialize};

fn default_mode() -> String {
    "normal".to_string()
}

fn default_one() -> u32 {
    1
}

/// A Lexical node – either a block container or an inline leaf.
///
/// The `type` field in JSON drives deserialization via serde's internal tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum LexicalNode {
    /// A custom paragraph node (`"paragraph-style"`).
    #[serde(rename = "paragraph-style")]
    ParagraphStyle {
        /// ODT style name (e.g. `"Standard"`).
        #[serde(rename = "styleName")]
        style_name: String,
        /// Inline or nested block children.
        children: Vec<LexicalNode>,
        /// Text direction.
        direction: Option<String>,
        /// Alignment format string (e.g. `""`, `"center"`).
        format: String,
        /// Indentation level.
        indent: u32,
        /// Always `1`.
        version: u32,
    },
    /// A custom heading node (`"heading-style"`).
    #[serde(rename = "heading-style")]
    HeadingStyle {
        /// HTML heading tag: `"h1"` … `"h6"`.
        tag: String,
        /// Optional ODT style name.
        #[serde(rename = "styleName", skip_serializing_if = "Option::is_none")]
        style_name: Option<String>,
        /// Inline children.
        children: Vec<LexicalNode>,
        /// Text direction.
        direction: Option<String>,
        /// Alignment format string.
        format: String,
        /// Indentation level.
        indent: u32,
        /// Always `1`.
        version: u32,
    },
    /// A text leaf node (`"text"`).
    #[serde(rename = "text")]
    Text {
        /// The raw text content.
        text: String,
        /// Bitmask of formatting flags (`FORMAT_*` constants).
        format: u32,
        /// Inline CSS style string (usually empty).
        #[serde(default)]
        style: String,
        /// Lexical rendering mode (usually `"normal"`).
        #[serde(default = "default_mode")]
        mode: String,
        /// Lexical detail flags (usually `0`).
        #[serde(default)]
        detail: u32,
        /// Named character style name, if any.
        #[serde(rename = "styleName", skip_serializing_if = "Option::is_none")]
        style_name: Option<String>,
        /// Always `1`.
        version: u32,
    },
    /// A hyperlink wrapper (`"link"`).
    #[serde(rename = "link")]
    Link {
        /// The destination URL.
        url: String,
        /// Optional link target (`"_blank"`, etc.).
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
        /// Optional `rel` attribute.
        #[serde(skip_serializing_if = "Option::is_none")]
        rel: Option<String>,
        /// Text children.
        children: Vec<LexicalNode>,
        /// Text direction.
        direction: Option<String>,
        /// Format string.
        format: String,
        /// Indentation level.
        indent: u32,
        /// Always `1`.
        version: u32,
    },
    /// A page break block (`"page-break"`).
    #[serde(rename = "page-break")]
    PageBreak {
        /// Always `1`.
        version: u32,
    },
    /// A hard line-break inline (`"linebreak"`).
    #[serde(rename = "linebreak")]
    LineBreak {
        /// Always `1`.
        version: u32,
    },
    /// An image block (`"image"`).
    #[serde(rename = "image")]
    Image {
        /// Image source URL.
        src: String,
        /// Alternative text.
        #[serde(rename = "altText")]
        alt_text: String,
        /// Always `1`.
        version: u32,
    },
    /// A table block (`"table"`).
    #[serde(rename = "table")]
    Table {
        children: Vec<LexicalNode>,
        direction: Option<String>,
        format: String,
        indent: u32,
        version: u32,
    },
    /// A table row (`"tablerow"`).
    #[serde(rename = "tablerow")]
    TableRow {
        children: Vec<LexicalNode>,
        direction: Option<String>,
        format: String,
        indent: u32,
        version: u32,
    },
    /// A table cell (`"tablecell"`). `header_state = 1` indicates a header.
    #[serde(rename = "tablecell")]
    TableCell {
        #[serde(rename = "colSpan", default = "default_one")]
        col_span: u32,
        #[serde(rename = "rowSpan", default = "default_one")]
        row_span: u32,
        #[serde(rename = "headerState", default)]
        header_state: u32,
        children: Vec<LexicalNode>,
        direction: Option<String>,
        format: String,
        indent: u32,
        version: u32,
    },
    /// A list block (`"list"`).
    #[serde(rename = "list")]
    List {
        /// `"bullet"` or `"number"`.
        #[serde(rename = "listType")]
        list_type: String,
        /// Starting number for ordered lists.
        start: u32,
        /// HTML tag: `"ul"` or `"ol"`.
        tag: String,
        children: Vec<LexicalNode>,
        direction: Option<String>,
        format: String,
        indent: u32,
        version: u32,
    },
    /// A list item (`"listitem"`).
    #[serde(rename = "listitem")]
    ListItem {
        /// Item value (position in ordered list).
        value: u32,
        children: Vec<LexicalNode>,
        direction: Option<String>,
        format: String,
        indent: u32,
        version: u32,
    },
    /// A block-quote (`"quote"`).
    #[serde(rename = "quote")]
    Quote {
        children: Vec<LexicalNode>,
        direction: Option<String>,
        format: String,
        indent: u32,
        version: u32,
    },
}
