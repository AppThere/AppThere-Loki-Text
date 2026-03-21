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

use std::collections::HashMap;
use common_core::StyleDefinition;

/// Returns `true` if the named style (or any ancestor via `parent`) has
/// `fo:break-before = "page"` or `style:break-before = "page"`.
pub(super) fn style_has_break_before(
    style_name: &str,
    styles: &HashMap<String, StyleDefinition>,
) -> bool {
    let mut current = style_name.to_string();
    let mut visited = std::collections::HashSet::new();
    loop {
        if !visited.insert(current.clone()) {
            break;
        }
        if let Some(style) = styles.get(&current) {
            let attrs = &style.attributes;
            if attrs.get("fo:break-before").map(|s| s.as_str()) == Some("page")
                || attrs.get("style:break-before").map(|s| s.as_str()) == Some("page")
            {
                return true;
            }
            match &style.parent {
                Some(parent) => current = parent.clone(),
                None => break,
            }
        } else {
            break;
        }
    }
    false
}
