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

//! PDF content stream operator emission for text runs.

use ttf_parser::GlyphId;

use crate::fonts::subset::FontSubset;

/// Write a BT...ET block for a single text run to `out`.
///
/// `x` and `y` are in PDF coordinate space (Y=0 at bottom of page, increases upward).
pub fn write_text_run(
    text: &str,
    subset: &FontSubset,
    font_pdf_name: &str,
    font_size: f64,
    x: f64,
    y: f64,
    colour_r: f32,
    colour_g: f32,
    colour_b: f32,
    out: &mut String,
) {
    if text.is_empty() {
        return;
    }

    // Build glyph hex string for TJ operator.
    let mut hex_glyphs = String::new();
    for ch in text.chars() {
        let gid: u16 = subset.unicode_map.get(&ch).map(|GlyphId(g)| *g).unwrap_or(0);
        hex_glyphs.push_str(&format!("{:04X}", gid));
    }

    out.push_str(&format!(
        "BT\n\
         /{font} {size:.2} Tf\n\
         {r:.4} {g:.4} {b:.4} rg\n\
         {x:.4} {y:.4} Td\n\
         <{glyphs}> Tj\n\
         ET\n",
        font = font_pdf_name,
        size = font_size,
        r = colour_r,
        g = colour_g,
        b = colour_b,
        x = x,
        y = y,
        glyphs = hex_glyphs,
    ));
}

/// Write a horizontal rule at the given Y position.
pub fn write_horizontal_rule(x: f64, y: f64, width: f64, out: &mut String) {
    out.push_str(&format!(
        "{x:.4} {y:.4} m\n\
         {x2:.4} {y:.4} l\n\
         S\n",
        x = x,
        y = y,
        x2 = x + width,
    ));
}

/// Write a filled rectangle (used for table cell borders).
pub fn write_rect_stroke(x: f64, y: f64, w: f64, h: f64, out: &mut String) {
    out.push_str(&format!(
        "{x:.4} {y:.4} {w:.4} {h:.4} re\nS\n",
        x = x,
        y = y,
        w = w,
        h = h,
    ));
}
