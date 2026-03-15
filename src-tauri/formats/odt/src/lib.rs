//! ODT document format support for AppThere Loki.
//!
//! This crate provides parsing, writing, and Lexical conversion for
//! OpenDocument Text (ODT and FODT) files.
//!
//! # Architecture
//!
//! ```text
//! ODT XML ──► parser ──► Document ──► lexical::to_lexical ──► LexicalDocument (JSON)
//!                            ▲                                        │
//!                            └──── lexical::from_lexical ◄───────────┘
//!                            │
//!                            ▼
//!                         writer ──► content.xml / styles.xml / meta.xml / FODT
//! ```
//!
//! # Examples
//!
//! ## Parsing an FODT file and converting to Lexical JSON
//!
//! ```no_run
//! use odt_format::parser::parse_document;
//! use odt_format::lexical::to_lexical;
//!
//! let xml = std::fs::read_to_string("document.fodt").unwrap();
//! let doc = parse_document(&xml).unwrap();
//! let lex = to_lexical(&doc);
//! let json = serde_json::to_string(&lex).unwrap();
//! ```
//!
//! ## Saving a document from Lexical editor state
//!
//! ```no_run
//! use odt_format::lexical::from_lexical;
//! use odt_format::writer::fodt::to_xml;
//! use common_core::{LexicalDocument, Metadata};
//! use std::collections::HashMap;
//!
//! let lex: LexicalDocument = serde_json::from_str("{}").unwrap();
//! let doc = from_lexical(lex, HashMap::new(), Metadata::default());
//! let xml = to_xml(&doc.blocks, &doc.styles, &doc.metadata,
//!                  &doc.font_face_decls, &doc.automatic_styles, &doc.master_styles).unwrap();
//! ```

pub mod document;
pub mod lexical;
pub mod namespaces;
pub mod parser;
pub mod tiptap;
pub mod writer;

pub use document::Document;
