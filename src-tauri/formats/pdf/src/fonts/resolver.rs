// Copyright 2024 AppThere
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Font resolver trait and Map-based implementation.
//!
//! The [`FontResolver`] trait abstracts font loading from the PDF writer,
//! allowing the Tauri host to supply embedded font bytes without the writer
//! depending on any filesystem layout.

use std::collections::HashMap;

/// Resolves font family names to font file bytes.
///
/// Implemented by the Tauri command layer which has access to bundled font
/// assets embedded in the binary. The PDF writer uses this trait to load
/// fonts without depending on the filesystem layout.
pub trait FontResolver: Send + Sync {
    /// Load the font bytes for the given family name and style.
    ///
    /// Returns `None` if the font is not available, in which case the
    /// writer will try the fallback family.
    fn resolve(&self, family: &str, bold: bool, italic: bool) -> Option<Vec<u8>>;

    /// The family name of the fallback font. Must always be resolvable.
    fn fallback_family(&self) -> &str;
}

/// A [`FontResolver`] backed by a `HashMap`.
///
/// Used by the Tauri command layer (via `build_font_resolver()`) and in
/// integration tests.
pub struct MapFontResolver {
    /// Key: (family_name_lowercase, bold, italic) → font bytes.
    fonts: HashMap<(String, bool, bool), Vec<u8>>,
    fallback: String,
}

impl MapFontResolver {
    /// Creates a new resolver with the given fallback family name.
    ///
    /// The fallback must be registered via [`add_font`] before the resolver
    /// is used, or font loading will always fail.
    pub fn new(fallback_family: impl Into<String>) -> Self {
        Self {
            fonts: HashMap::new(),
            fallback: fallback_family.into(),
        }
    }

    /// Register a font variant.
    ///
    /// `family` is matched case-insensitively. For variable fonts, register
    /// the same bytes for multiple `(bold, italic)` combinations.
    pub fn add_font(&mut self, family: impl Into<String>, bold: bool, italic: bool, bytes: Vec<u8>) {
        self.fonts.insert((family.into().to_lowercase(), bold, italic), bytes);
    }
}

impl FontResolver for MapFontResolver {
    fn resolve(&self, family: &str, bold: bool, italic: bool) -> Option<Vec<u8>> {
        // Try exact (bold, italic) match first, then fall back to (false, false).
        let key = (family.to_lowercase(), bold, italic);
        if let Some(b) = self.fonts.get(&key) {
            return Some(b.clone());
        }
        // For missing bold/italic variants try the regular face.
        if bold || italic {
            let fallback_key = (family.to_lowercase(), false, false);
            if let Some(b) = self.fonts.get(&fallback_key) {
                return Some(b.clone());
            }
        }
        None
    }

    fn fallback_family(&self) -> &str {
        &self.fallback
    }
}
