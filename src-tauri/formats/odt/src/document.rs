//! The core ODT [`Document`] struct.
//!
//! This module defines the [`Document`] type which is the central data model
//! for an ODT document, holding parsed blocks, named styles, metadata, and
//! optional raw XML sections for round-trip preservation.
//!
//! # Examples
//!
//! ```
//! use odt_format::Document;
//!
//! let doc = Document::new();
//! assert!(doc.blocks.is_empty());
//! assert!(doc.styles.is_empty());
//! ```

use std::collections::HashMap;

use common_core::{Block, Metadata, StyleDefinition};

/// The top-level ODT document model.
///
/// Holds the parsed document content (blocks and styles) together with
/// raw XML sections that must be preserved verbatim for round-trip fidelity
/// when updating existing `.odt` or `.fodt` files.
#[derive(Debug, Clone)]
pub struct Document {
    /// The document's block-level content.
    pub blocks: Vec<Block>,
    /// Named styles keyed by style name.
    pub styles: HashMap<String, StyleDefinition>,
    /// Document metadata (title, author, language, etc.).
    pub metadata: Metadata,
    /// Preserved `<office:font-face-decls>` XML for round-trip fidelity.
    pub font_face_decls: Option<String>,
    /// Preserved `<office:automatic-styles>` XML for round-trip fidelity.
    pub automatic_styles: Option<String>,
    /// Preserved `<office:master-styles>` XML for round-trip fidelity.
    pub master_styles: Option<String>,
}

impl Default for Document {
    /// Creates an empty document.
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    /// Creates a new empty document with no content or styles.
    ///
    /// # Examples
    ///
    /// ```
    /// use odt_format::Document;
    ///
    /// let doc = Document::new();
    /// assert!(doc.blocks.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            styles: HashMap::new(),
            metadata: Metadata::default(),
            font_face_decls: None,
            automatic_styles: None,
            master_styles: None,
        }
    }
}
