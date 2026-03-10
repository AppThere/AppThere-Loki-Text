//! Document style definitions.
//!
//! This module defines [`StyleDefinition`] and [`StyleFamily`] which represent
//! named styles parsed from ODT/ODS documents, including their ODF properties.
//!
//! # Examples
//!
//! ```
//! use common_core::style::{StyleDefinition, StyleFamily};
//! use std::collections::HashMap;
//!
//! let style = StyleDefinition {
//!     name: "Heading 1".to_string(),
//!     family: StyleFamily::Paragraph,
//!     parent: Some("Standard".to_string()),
//!     next: Some("Text Body".to_string()),
//!     display_name: Some("Heading 1".to_string()),
//!     attributes: HashMap::new(),
//!     text_transform: None,
//!     outline_level: Some(1),
//!     autocomplete: Some(false),
//! };
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// The family (scope) of a style definition.
///
/// Determines whether the style applies to paragraphs or character spans.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StyleFamily {
    /// A paragraph style (`style:family="paragraph"`).
    Paragraph,
    /// A character/text style (`style:family="text"`).
    Text,
}

impl StyleFamily {
    /// Returns the ODF `style:family` attribute value string.
    ///
    /// # Examples
    ///
    /// ```
    /// use common_core::style::StyleFamily;
    /// assert_eq!(StyleFamily::Paragraph.to_odf_str(), "paragraph");
    /// assert_eq!(StyleFamily::Text.to_odf_str(), "text");
    /// ```
    #[must_use]
    pub fn to_odf_str(&self) -> &'static str {
        match self {
            StyleFamily::Paragraph => "paragraph",
            StyleFamily::Text => "text",
        }
    }
}

/// A named style definition from an ODT document.
///
/// Stores all ODF paragraph and text properties as a flat key-value map
/// using prefixed attribute names (e.g. `"fo:font-size"`, `"style:font-name"`).
/// This enables round-trip preservation of style data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StyleDefinition {
    /// The internal style name (used in `text:style-name` attributes).
    pub name: String,
    /// Whether this is a paragraph or character style.
    pub family: StyleFamily,
    /// The parent style name for inheritance.
    pub parent: Option<String>,
    /// The style to use for the next paragraph after this one.
    pub next: Option<String>,
    /// A human-readable display name shown in style pickers.
    pub display_name: Option<String>,
    /// All ODF CSS-compatible attributes as `"prefix:name"` → `"value"`.
    pub attributes: HashMap<String, String>,
    /// The `fo:text-transform` value if present (e.g. `"uppercase"`).
    #[serde(rename = "textTransform")]
    pub text_transform: Option<String>,
    /// The outline level for heading styles (1–9).
    #[serde(rename = "outlineLevel")]
    pub outline_level: Option<u32>,
    /// Loki-specific: whether this style participates in autocomplete.
    #[serde(default)]
    pub autocomplete: Option<bool>,
}
