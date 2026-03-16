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

//! Text width measurement using rustybuzz for OpenType shaping.

use rustybuzz::{Face, UnicodeBuffer};

/// Measure the advance width of `text` when rendered at `font_size_pt` points.
///
/// Uses `rustybuzz` for OpenType-aware shaping. Returns the width in points.
/// Falls back to a character-count approximation if the font cannot be parsed.
pub fn measure_text(text: &str, font_bytes: &[u8], font_size_pt: f64) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    if let Some(face) = Face::from_slice(font_bytes, 0) {
        let upm = face.units_per_em() as f64;
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        let output = rustybuzz::shape(&face, &[], buffer);
        let total: i32 = output.glyph_positions().iter().map(|p| p.x_advance).sum();
        total as f64 / upm * font_size_pt
    } else {
        // Fallback approximation: 0.5 em per ASCII char, 0.6 em per non-ASCII.
        text.chars()
            .map(|c| if c.is_ascii() { 0.5 } else { 0.6 })
            .sum::<f64>()
            * font_size_pt
    }
}

/// Measure the width of a single space character in points.
pub fn space_width(font_bytes: &[u8], font_size_pt: f64) -> f64 {
    measure_text(" ", font_bytes, font_size_pt)
}
