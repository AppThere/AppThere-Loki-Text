//! Document metadata.
//!
//! This module defines the [`Metadata`] struct which holds Dublin Core and
//! ODF meta fields for documents (title, creator, language, etc.).
//!
//! # Examples
//!
//! ```
//! use common_core::metadata::Metadata;
//!
//! let meta = Metadata {
//!     title: Some("My Document".to_string()),
//!     creator: Some("Alice".to_string()),
//!     language: Some("en".to_string()),
//!     ..Metadata::default()
//! };
//! assert_eq!(meta.title.as_deref(), Some("My Document"));
//! ```

use serde::{Deserialize, Serialize};

/// Document-level metadata fields following Dublin Core conventions.
///
/// All fields are optional; absent fields are omitted during serialization.
/// Corresponds to the `<office:meta>` section of an ODT document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Metadata {
    /// A unique document identifier (dc:identifier).
    pub identifier: Option<String>,
    /// The document title (dc:title).
    pub title: Option<String>,
    /// The primary language of the document (dc:language), e.g. `"en-US"`.
    pub language: Option<String>,
    /// A brief description of the document (dc:description).
    pub description: Option<String>,
    /// The document subject (dc:subject).
    pub subject: Option<String>,
    /// The document creator/author (dc:creator).
    pub creator: Option<String>,
    /// ISO 8601 creation timestamp (meta:creation-date).
    #[serde(rename = "creationDate")]
    pub creation_date: Option<String>,
    /// The application that created this document (meta:generator).
    pub generator: Option<String>,
}
