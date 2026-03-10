//! Lexical editor state ↔ ODT [`Document`] conversion.
//!
//! This module provides [`from_lexical`] and [`to_lexical`] which let the
//! Tauri backend exchange the native Lexical JSON format directly with the
//! TypeScript frontend — eliminating the TipTap intermediate layer.
//!
//! # Data flow
//!
//! ```text
//! open:  ODT file → Document → to_lexical() → LexicalDocument → frontend
//! save:  frontend → LexicalDocument → from_lexical() → Document → ODT file
//! ```

mod from_lexical;
mod to_lexical;

pub use from_lexical::from_lexical;
pub use to_lexical::to_lexical;
