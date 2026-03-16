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

/*
FINDINGS - Phase 9
1. StyleDefinition layout fields: Uses `attributes: HashMap<String, String>`.
   Does NOT have structured fields for text_align, space_before, etc.
2. Property Resolution: Layout engine currently looks up from `attributes` map.
3. Line Height: Currently `font_size * 1.35` (fixed multiple).
4. Text Measurement: Uses `rustybuzz` glyph advances.
5. GID Usage: Writes subset GIDs from `FontSubset.unicode_map`.
*/

//! Font subsetting using the production-quality `subsetter` crate.

use crate::error::{PdfError, PdfResult};
use std::collections::{HashMap, HashSet};
use subsetter;
use ttf_parser::{Face, GlyphId, Tag};

/// A set of Unicode code points required by a single font variant.
pub type UsedGlyphs = HashSet<char>;

/// Font metrics extracted from an OpenType face.
#[derive(Debug, Clone)]
pub struct FontMetrics {
    pub family_name: String,
    pub units_per_em: u16,
    pub ascender: i16,
    pub descender: i16,
    pub cap_height: i16,
    pub x_height: i16,
    pub italic_angle: f32,
    pub is_bold: bool,
    pub is_italic: bool,
    pub bbox: [i16; 4],
}

/// A font prepared for PDF embedding.
#[derive(Debug, Clone)]
pub struct FontSubset {
    pub bytes: Vec<u8>,
    /// Maps original glyph ID to the ID used in the embedded font.
    /// (Identity mapping with the subsetter crate).
    pub glyph_id_map: HashMap<GlyphId, GlyphId>,
    /// Maps Unicode code point -> glyph ID in the embedded font (subset GID).
    pub unicode_map: HashMap<char, GlyphId>,
    pub metrics: FontMetrics,
    pub advance_widths: HashMap<GlyphId, u16>,
}

/// A variation axis coordinate for variable fonts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VariationCoordinate {
    pub axis: ttf_parser::Tag,
    pub value: f32,
}

/// Prepare a font for PDF embedding by stripping unused glyphs.
pub fn create_subset(font_bytes: &[u8], used_chars: &UsedGlyphs) -> PdfResult<FontSubset> {
    // 1. Parse the original face to collect glyph IDs and metrics
    let face = Face::parse(font_bytes, 0)
        .map_err(|e| PdfError::FontLoad(format!("Failed to parse font: {:?}", e)))?;

    // 2. Map each used character to its original GID
    let mut char_to_original_gid: HashMap<char, GlyphId> = HashMap::new();
    for &ch in used_chars {
        if let Some(gid) = face.glyph_index(ch) {
            char_to_original_gid.insert(ch, gid);
        }
    }

    // 3. Collect original GIDs for subsetter (plus .notdef = 0)
    let mut glyph_ids: Vec<u16> = vec![0]; // always include .notdef
    for gid in char_to_original_gid.values() {
        if gid.0 != 0 {
            glyph_ids.push(gid.0);
        }
    }
    glyph_ids.sort_unstable();
    glyph_ids.dedup();

    // 4. Run the subsetter
    let mut remapper = subsetter::GlyphRemapper::new();
    let mut original_to_new: HashMap<u16, u16> = HashMap::new();

    // CID fonts in PDF often expect .notdef at GID 0.
    // Ensure 0 is remapped first if it exists.
    let new_notdef = remapper.remap(0);
    original_to_new.insert(0, new_notdef);

    for &gid_u16 in &glyph_ids {
        if gid_u16 != 0 {
            let new_gid = remapper.remap(gid_u16);
            original_to_new.insert(gid_u16, new_gid);
        }
    }

    let subsetted_bytes = subsetter::subset(font_bytes, 0, &remapper)
        .map_err(|e| PdfError::FontSubset(format!("Subsetting failed: {:?}", e)))?;

    // 5. Build lookup maps using the NEW glyph IDs
    let mut unicode_map: HashMap<char, GlyphId> = HashMap::new();
    for (ch, &gid) in &char_to_original_gid {
        if let Some(&new_gid) = original_to_new.get(&gid.0) {
            unicode_map.insert(*ch, GlyphId(new_gid));
        } else {
            unicode_map.insert(*ch, GlyphId(0));
        }
    }

    let glyph_id_map: HashMap<GlyphId, GlyphId> = original_to_new
        .iter()
        .map(|(&orig, &new)| (GlyphId(orig), GlyphId(new)))
        .collect();

    // 6. Extract metrics and advance widths
    let mut advance_widths = HashMap::new();
    for (&orig, &new) in &original_to_new {
        let adv = face
            .glyph_hor_advance(GlyphId(orig))
            .unwrap_or(face.units_per_em());
        advance_widths.insert(GlyphId(new), adv);
    }

    let metrics = extract_metrics(&face)?;

    Ok(FontSubset {
        bytes: subsetted_bytes,
        glyph_id_map,
        unicode_map,
        metrics,
        advance_widths,
    })
}

fn extract_metrics(face: &Face) -> PdfResult<FontMetrics> {
    let family_name = face
        .names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::FULL_NAME && n.is_unicode())
        .or_else(|| {
            face.names()
                .into_iter()
                .find(|n| n.name_id == ttf_parser::name_id::FAMILY && n.is_unicode())
        })
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let bbox = face.global_bounding_box();

    Ok(FontMetrics {
        family_name,
        units_per_em: face.units_per_em(),
        ascender: face.ascender(),
        descender: face.descender(),
        cap_height: face.capital_height().unwrap_or(face.ascender()),
        x_height: face.x_height().unwrap_or(face.ascender() / 2),
        italic_angle: face.italic_angle(),
        is_bold: face.is_bold(),
        is_italic: face.is_italic(),
        bbox: [bbox.x_min, bbox.y_min, bbox.x_max, bbox.y_max],
    })
}

/// For a variable font, find the axis coordinates for the requested style.
pub fn find_variation_coordinates(
    font_bytes: &[u8],
    bold: bool,
    italic: bool,
) -> Option<Vec<(Tag, f32)>> {
    let face = Face::parse(font_bytes, 0).ok()?;
    if !face.is_variable() {
        return None;
    }

    let mut coords = Vec::new();

    for axis in face.variation_axes() {
        let tag = axis.tag;
        let value = if tag == Tag::from_bytes(b"wght") {
            if bold {
                700.0
            } else {
                400.0
            }
        } else if tag == Tag::from_bytes(b"ital") {
            if italic {
                1.0
            } else {
                0.0
            }
        } else if tag == Tag::from_bytes(b"slnt") {
            if italic {
                -12.0
            } else {
                0.0
            }
        } else {
            axis.def_value
        };
        coords.push((tag, value));
    }

    if coords.is_empty() {
        None
    } else {
        Some(coords)
    }
}
