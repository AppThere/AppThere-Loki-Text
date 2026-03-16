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
//!     #[cfg(feature = "colour-management")]
//!     font_colour: None,
//!     #[cfg(feature = "colour-management")]
//!     background_colour: None,
//! };
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[cfg(feature = "colour-management")]
use crate::colour_management::Colour;

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
    /// The typed font colour, if set. Populated from `fo:color` / `loki:colour`.
    #[cfg(feature = "colour-management")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_colour: Option<Colour>,
    /// The typed background colour, if set. Populated from `fo:background-color`.
    #[cfg(feature = "colour-management")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_colour: Option<Colour>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_family_paragraph_odf_str() {
        assert_eq!(StyleFamily::Paragraph.to_odf_str(), "paragraph");
    }

    #[test]
    fn style_family_text_odf_str() {
        assert_eq!(StyleFamily::Text.to_odf_str(), "text");
    }

    #[test]
    fn style_definition_serde_roundtrip() {
        let style = StyleDefinition {
            name: "Heading 1".to_string(),
            family: StyleFamily::Paragraph,
            parent: Some("Standard".to_string()),
            next: Some("Text Body".to_string()),
            display_name: Some("Heading 1".to_string()),
            attributes: HashMap::from([("fo:font-size".to_string(), "14pt".to_string())]),
            text_transform: None,
            outline_level: Some(1),
            autocomplete: Some(true),
            #[cfg(feature = "colour-management")]
            font_colour: None,
            #[cfg(feature = "colour-management")]
            background_colour: None,
        };
        let json = serde_json::to_string(&style).unwrap();
        let decoded: StyleDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, style);
    }

    #[test]
    fn style_definition_minimal() {
        let style = StyleDefinition {
            name: "Default".to_string(),
            family: StyleFamily::Text,
            parent: None,
            next: None,
            display_name: None,
            attributes: HashMap::new(),
            text_transform: None,
            outline_level: None,
            autocomplete: None,
            #[cfg(feature = "colour-management")]
            font_colour: None,
            #[cfg(feature = "colour-management")]
            background_colour: None,
        };
        assert_eq!(style.name, "Default");
        assert_eq!(style.family, StyleFamily::Text);
        assert!(style.attributes.is_empty());
    }
}
