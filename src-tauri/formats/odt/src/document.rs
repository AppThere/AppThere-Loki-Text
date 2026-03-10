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

use crate::{
    parser,
    writer::{content, fodt, meta, styles_writer},
};

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

    /// Parses an ODT XML string (FODT or component XML) into a [`Document`].
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        parser::parse_document(xml)
    }

    /// Merges named styles from a `styles.xml` string into this document.
    pub fn add_styles_from_xml(&mut self, xml: &str) -> Result<(), String> {
        parser::add_styles_from_xml(self, xml)
    }

    /// Serializes this document to a complete FODT XML string.
    pub fn to_xml(&self) -> Result<String, String> {
        fodt::to_xml(
            &self.blocks,
            &self.styles,
            &self.metadata,
            &self.font_face_decls,
            &self.automatic_styles,
            &self.master_styles,
        )
    }

    /// Updates an existing FODT XML string with this document's content.
    pub fn update_fodt(&self, old_xml: &str) -> Result<String, String> {
        let content_xml = self.to_content_xml()?;
        let styles_xml = self.styles_to_xml()?;
        let meta_xml = self.to_meta_xml()?;
        fodt::update_fodt(
            old_xml,
            &self.blocks,
            &self.styles,
            &self.metadata,
            &content_xml,
            &styles_xml,
            &meta_xml,
        )
    }

    /// Generates a `content.xml` string for use in an ODT ZIP archive.
    pub fn to_content_xml(&self) -> Result<String, String> {
        content::to_content_xml(&self.blocks)
    }

    /// Generates a `styles.xml` string for use in an ODT ZIP archive.
    pub fn styles_to_xml(&self) -> Result<String, String> {
        styles_writer::styles_to_xml(
            &self.styles,
            &self.font_face_decls,
            &self.automatic_styles,
            &self.master_styles,
        )
    }

    /// Generates a `meta.xml` string for use in an ODT ZIP archive.
    pub fn to_meta_xml(&self) -> Result<String, String> {
        meta::to_meta_xml(&self.metadata)
    }
}
