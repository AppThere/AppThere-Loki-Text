//! Core document types for AppThere Loki format support.
//!
//! This crate provides format-agnostic types that represent document
//! structure, including blocks, inline content, metadata, and styles.
//! It also provides Tiptap/Lexical-compatible bridge types for
//! frontend communication.
//!
//! # Architecture
//!
//! Types in this crate are:
//! - **Format-agnostic**: usable across ODT, DOCX, and future formats
//! - **Serde-compatible**: serialize to/from Lexical JSON
//! - **No `unsafe` code**: strict safety guarantees
//!
//! # Examples
//!
//! ```
//! use common_core::{Block, Inline, Metadata, StyleDefinition};
//! use common_core::tiptap::{TiptapNode, TiptapAttrs};
//!
//! let para = Block::Paragraph {
//!     style_name: Some("Standard".to_string()),
//!     attrs: None,
//!     content: vec![Inline::Text {
//!         text: "Hello, World!".to_string(),
//!         style_name: None,
//!         marks: vec![],
//!     }],
//! };
//! ```

pub mod block;
pub mod inline;
pub mod marks;
pub mod metadata;
pub mod style;
pub mod tiptap;

pub use block::{Block, BlockAttrs, CellAttrs};
pub use inline::Inline;
pub use marks::{LinkAttrs, TiptapAttrsInline, TiptapMark};
pub use metadata::Metadata;
pub use style::{StyleDefinition, StyleFamily};
pub use tiptap::{ImageAttrs, TiptapAttrs, TiptapNode, TiptapResponse};
