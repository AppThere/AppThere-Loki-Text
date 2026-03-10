//! ODT document format support for AppThere Loki.
//!
//! This crate provides parsing, writing, and Tiptap/Lexical conversion for
//! OpenDocument Text (ODT and FODT) files.
//!
//! # Architecture
//!
//! ```text
//! ODT XML ──► parser ──► Document ──► tiptap::to_tiptap ──► TiptapNode (JSON)
//!                            ▲                                      │
//!                            └─── tiptap::from_tiptap ◄────────────┘
//!                            │
//!                            ▼
//!                         writer ──► content.xml / styles.xml / meta.xml / FODT
//! ```
//!
//! # Examples
//!
//! ## Parsing an FODT file
//!
//! ```no_run
//! use odt_format::parser::parse_document;
//! use odt_format::tiptap::to_tiptap::document_to_tiptap;
//!
//! let xml = std::fs::read_to_string("document.fodt").unwrap();
//! let doc = parse_document(&xml).unwrap();
//! let tiptap = document_to_tiptap(&doc.blocks);
//! ```
//!
//! ## Updating an FODT file from editor state
//!
//! ```no_run
//! use odt_format::tiptap::from_tiptap::tiptap_to_document;
//! use odt_format::writer::fodt::to_xml;
//! use common_core::{TiptapNode, Metadata};
//! use std::collections::HashMap;
//!
//! let root: TiptapNode = serde_json::from_str("{}").unwrap();
//! let doc = tiptap_to_document(root, HashMap::new(), Metadata::default());
//! let xml = to_xml(&doc.blocks, &doc.styles, &doc.metadata,
//!                  &doc.font_face_decls, &doc.automatic_styles, &doc.master_styles).unwrap();
//! ```

pub mod document;
pub mod namespaces;
pub mod parser;
pub mod tiptap;
pub mod writer;

pub use document::Document;
