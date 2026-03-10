//! Inline-level document content.
//!
//! This module defines the [`Inline`] enum which represents inline content
//! within block elements such as styled text runs and line breaks.
//!
//! # Examples
//!
//! ```
//! use common_core::inline::Inline;
//! use common_core::marks::TiptapMark;
//!
//! let text = Inline::Text {
//!     text: "Hello, World!".to_string(),
//!     style_name: Some("Emphasis".to_string()),
//!     marks: vec![TiptapMark::Italic],
//! };
//! let line_break = Inline::LineBreak;
//! ```

use serde::{Deserialize, Serialize};

use crate::marks::TiptapMark;

/// An inline content element within a block.
///
/// Inlines are the leaf-level content inside paragraphs, headings, and
/// other block elements. Each inline is either a styled text run or a
/// hard line break.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Inline {
    /// A text run with optional ODT style and formatting marks.
    ///
    /// # Example
    /// ```
    /// # use common_core::inline::Inline;
    /// # use common_core::marks::TiptapMark;
    /// let bold_text = Inline::Text {
    ///     text: "important".to_string(),
    ///     style_name: None,
    ///     marks: vec![TiptapMark::Bold],
    /// };
    /// ```
    Text {
        /// The raw text content.
        text: String,
        /// The ODT character style name, if any.
        #[serde(rename = "styleName")]
        style_name: Option<String>,
        /// Formatting marks applied to this text run.
        #[serde(default)]
        marks: Vec<TiptapMark>,
    },
    /// A hard line break (`text:line-break` in ODT).
    LineBreak,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marks::TiptapMark;

    #[test]
    fn text_inline_serde_roundtrip() {
        let inline = Inline::Text {
            text: "Hello, World!".to_string(),
            style_name: Some("Emphasis".to_string()),
            marks: vec![TiptapMark::Bold, TiptapMark::Italic],
        };
        let json = serde_json::to_string(&inline).unwrap();
        let decoded: Inline = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, inline);
    }

    #[test]
    fn text_inline_no_marks() {
        let inline = Inline::Text {
            text: "plain".to_string(),
            style_name: None,
            marks: vec![],
        };
        if let Inline::Text { text, style_name, marks } = &inline {
            assert_eq!(text, "plain");
            assert!(style_name.is_none());
            assert!(marks.is_empty());
        } else {
            panic!("expected Text variant");
        }
    }

    #[test]
    fn line_break_serde_roundtrip() {
        let inline = Inline::LineBreak;
        let json = serde_json::to_string(&inline).unwrap();
        let decoded: Inline = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, Inline::LineBreak);
    }
}
