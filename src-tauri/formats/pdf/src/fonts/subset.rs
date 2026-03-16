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

//! Font subsetting for PDF embedding.
//!
//! # Phase 7 note
//!
//! Full OpenType table surgery (subsetting) is deferred to Phase 8.
//! This implementation embeds the complete font file with an identity
//! glyph-ID map and emits a visible warning. The resulting PDFs are
//! valid PDF/X-4 but have larger file sizes than optimally subsetted PDFs.

use std::collections::{HashMap, HashSet};

use ttf_parser::{Face, GlyphId};

use crate::error::PdfError;

/// A set of Unicode code points required by a single font variant.
pub type UsedGlyphs = HashSet<char>;

/// Font metrics extracted from an OpenType face.
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// PostScript / family name of the font.
    pub family_name: String,
    /// Number of font design units per EM square.
    pub units_per_em: u16,
    /// Typographic ascender in font units.
    pub ascender: i16,
    /// Typographic descender in font units (usually negative).
    pub descender: i16,
    /// Cap-height in font units.
    pub cap_height: i16,
    /// x-height in font units.
    pub x_height: i16,
    /// Italic angle in degrees (negative = forward slant).
    pub italic_angle: f32,
    /// Whether the face carries a bold style flag.
    pub is_bold: bool,
    /// Whether the face carries an italic style flag.
    pub is_italic: bool,
    /// Bounding box [xMin, yMin, xMax, yMax] in font units.
    pub bbox: [i16; 4],
}

/// A font prepared for PDF embedding.
///
/// In Phase 7, `bytes` is the full original font file.
/// In Phase 8, `bytes` will be a properly subsetted font.
#[derive(Debug, Clone)]
pub struct FontSubset {
    /// The font bytes to embed in the PDF.
    pub bytes: Vec<u8>,
    /// Maps original glyph ID to the ID used in the embedded font.
    /// Identity map in Phase 7.
    pub glyph_id_map: HashMap<GlyphId, GlyphId>,
    /// Maps Unicode code point → glyph ID in the embedded font.
    pub unicode_map: HashMap<char, GlyphId>,
    /// Metrics for building the PDF FontDescriptor.
    pub metrics: FontMetrics,
    /// Advance widths in font units, keyed by glyph ID.
    pub advance_widths: HashMap<GlyphId, u16>,
}

/// Prepare a font for PDF embedding.
///
/// In Phase 7 this is a "subset" in name only — the full font is used.
/// What this function *does* properly:
/// - Extracts metrics for the FontDescriptor
/// - Builds the unicode → glyph-ID map for used characters
/// - Collects advance widths for the /W array
///
/// Phase 8 will replace the body with proper table surgery.
pub fn create_subset(font_bytes: &[u8], used_chars: &UsedGlyphs) -> Result<FontSubset, PdfError> {
    let face = Face::parse(font_bytes, 0)
        .map_err(|e| PdfError::FontLoad(format!("Failed to parse font: {:?}", e)))?;

    // Extract metrics -------------------------------------------------------
    let units_per_em = face.units_per_em();

    let family_name = face
        .names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && n.is_unicode())
        .and_then(|n| n.to_string())
        .or_else(|| {
            face.names()
                .into_iter()
                .find(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
                .and_then(|n| n.to_string())
        })
        .or_else(|| {
            face.names()
                .into_iter()
                .find(|n| n.name_id == ttf_parser::name_id::FAMILY)
                .and_then(|n| n.to_string())
        })
        .unwrap_or_else(|| "Unknown".to_string());

    let ascender = face.ascender();
    let descender = face.descender();
    let cap_height = face.capital_height().unwrap_or(ascender);
    let x_height = face.x_height().unwrap_or(ascender / 2);
    let italic_angle = face.italic_angle();
    let is_bold = face.is_bold();
    let is_italic = face.is_italic();

    let bbox_raw = face.global_bounding_box();
    let bbox = [
        bbox_raw.x_min,
        bbox_raw.y_min,
        bbox_raw.x_max,
        bbox_raw.y_max,
    ];

    let metrics = FontMetrics {
        family_name: family_name.clone(),
        units_per_em,
        ascender,
        descender,
        cap_height,
        x_height,
        italic_angle,
        is_bold,
        is_italic,
        bbox,
    };

    // Build unicode map and collect advance widths --------------------------
    let mut unicode_map: HashMap<char, GlyphId> = HashMap::new();
    let mut glyph_id_map: HashMap<GlyphId, GlyphId> = HashMap::new();
    let mut advance_widths: HashMap<GlyphId, u16> = HashMap::new();

    for &ch in used_chars {
        if let Some(gid) = face.glyph_index(ch) {
            unicode_map.insert(ch, gid);
            glyph_id_map.insert(gid, gid); // identity in Phase 7
            let adv = face.glyph_hor_advance(gid).unwrap_or(units_per_em);
            advance_widths.insert(gid, adv);
        }
    }

    // Phase 7: full embedding with warning ----------------------------------
    eprintln!(
        "[loki-pdf] WARNING: Full font embedding (not subsetted) for '{}'. \
         PDF file size will be larger than optimal. \
         Implement proper subsetting in Phase 8.",
        family_name
    );

    Ok(FontSubset {
        bytes: font_bytes.to_vec(),
        glyph_id_map,
        unicode_map,
        metrics,
        advance_widths,
    })
}
