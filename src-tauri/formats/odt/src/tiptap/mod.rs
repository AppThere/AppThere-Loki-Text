//! Tiptap JSON ↔ ODT Document conversion.
//!
//! Provides bidirectional conversion between the internal [`Document`] model
//! and the Tiptap/Lexical JSON format used by the frontend editor.
//!
//! - [`to_tiptap`]: converts [`Document`] → `TiptapNode::Doc`
//! - [`from_tiptap`]: converts `TiptapNode::Doc` → [`Document`]

pub mod from_tiptap;
pub mod to_tiptap;
