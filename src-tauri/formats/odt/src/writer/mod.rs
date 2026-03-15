//! ODT document writers.
//!
//! This module provides XML generation for all ODT/FODT file variants:
//!
//! - [`content`]: generates `content.xml` for ZIP-format ODT files
//! - [`fodt`]: generates complete FODT flat XML documents and in-place updates
//! - [`meta`]: generates `meta.xml` for ZIP-format ODT files
//! - [`styles_writer`]: generates `styles.xml` for ZIP-format ODT files
//! - [`blocks`]: shared block XML writers
//! - [`inlines`]: shared inline XML writers
//! - [`namespaces`]: ODF namespace attribute helpers

pub mod blocks;
pub mod content;
pub mod fodt;
pub mod inlines;
pub mod meta;
pub mod namespaces;
pub mod styles_writer;
