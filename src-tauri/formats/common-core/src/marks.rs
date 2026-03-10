//! Text formatting marks (bold, italic, underline, etc.).
//!
//! This module defines the [`TiptapMark`] enum which represents inline
//! text formatting applied to text runs, compatible with the Tiptap/Lexical
//! JSON editor format.
//!
//! # Examples
//!
//! ```
//! use common_core::marks::{TiptapMark, LinkAttrs};
//!
//! let bold = TiptapMark::Bold;
//! let link = TiptapMark::Link {
//!     attrs: LinkAttrs {
//!         href: "https://example.com".to_string(),
//!         target: Some("_blank".to_string()),
//!     },
//! };
//! ```

use serde::{Deserialize, Serialize};

/// A hyperlink attribute set for link marks.
///
/// Contains the destination URL and optional target window specifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LinkAttrs {
    /// The destination URL.
    pub href: String,
    /// The link target (e.g., `"_blank"` for a new tab).
    pub target: Option<String>,
}

/// Inline text formatting marks compatible with Tiptap/Lexical JSON.
///
/// Marks represent formatting applied to text runs such as bold, italic,
/// underline, or named character styles from ODT documents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TiptapMark {
    /// A named ODT character style.
    NamedSpanStyle { attrs: TiptapAttrsInline },
    /// Bold text (`fo:font-weight: bold`).
    Bold,
    /// Italic text (`fo:font-style: italic`).
    Italic,
    /// Underlined text (`style:text-underline-style`).
    Underline,
    /// Strikethrough text.
    Strike,
    /// Superscript text.
    Superscript,
    /// Subscript text.
    Subscript,
    /// A hyperlink.
    Link { attrs: LinkAttrs },
}

/// Minimal attrs struct used within `NamedSpanStyle` marks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TiptapAttrsInline {
    /// The ODT style name for this span.
    #[serde(rename = "styleName")]
    pub style_name: Option<String>,
}
