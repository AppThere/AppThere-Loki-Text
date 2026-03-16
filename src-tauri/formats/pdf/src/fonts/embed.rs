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

//! Embeds a [`FontSubset`] into a PDF as a Type0/CIDFont composite font.
//!
//! The resulting font supports arbitrary Unicode via a ToUnicode CMap,
//! enabling text search and copy-paste in PDF readers.

use std::collections::BTreeMap;

use pdf_writer::types::{CidFontType, FontFlags, SystemInfo};
use pdf_writer::{Name, Pdf, Ref, Str};
use ttf_parser::GlyphId;

use crate::error::PdfError;
use crate::fonts::subset::{FontMetrics, FontSubset};

/// Embed a font subset into the PDF.
///
/// Writes:
/// - `FontFile2` stream (raw font bytes)
/// - `FontDescriptor` dictionary
/// - `CIDFont` dictionary (Type2, Identity-H)
/// - `ToUnicode` CMap stream
/// - `Type0` composite font dictionary
///
/// Returns `(resource_name, font_dict_ref)` where `resource_name` is the
/// string to use in the content stream's `Tf` operator (e.g. `"F0"`).
pub fn embed_font(
    subset: &FontSubset,
    pdf: &mut Pdf,
    next_ref: &mut i32,
) -> Result<(String, Ref), PdfError> {
    let font_ref = alloc(next_ref);
    let cid_ref = alloc(next_ref);
    let descriptor_ref = alloc(next_ref);
    let font_file_ref = alloc(next_ref);
    let cmap_ref = alloc(next_ref);

    // 1. Font file stream ---------------------------------------------------
    pdf.stream(font_file_ref, &subset.bytes);
        // .filter(pdf_writer::Filter::FlateDecode); // Missing actual compression in Phase 7

    // 2. Font descriptor ----------------------------------------------------
    write_font_descriptor(pdf, descriptor_ref, font_file_ref, &subset.metrics, subset)?;

    // 3. CID font (descendant) ----------------------------------------------
    write_cid_font(pdf, cid_ref, descriptor_ref, &subset.metrics, subset)?;

    // 4. ToUnicode CMap -----------------------------------------------------
    write_to_unicode_cmap(pdf, cmap_ref, subset);

    // 5. Type0 composite font -----------------------------------------------
    let base_name = sanitise_font_name(&subset.metrics.family_name);
    // PDF/X-4 and most readers prefer PSName in Type0 BaseFont
    let mut font = pdf.type0_font(font_ref);
    font.base_font(Name(base_name.as_bytes()));
    font.encoding_predefined(Name(b"Identity-H"));
    font.descendant_font(cid_ref);
    font.to_unicode(cmap_ref);
    drop(font);

    Ok((base_name, font_ref))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn alloc(next_ref: &mut i32) -> Ref {
    let r = Ref::new(*next_ref);
    *next_ref += 1;
    r
}

fn sanitise_font_name(name: &str) -> String {
    // PDF font names must be valid Name strings: ASCII, no whitespace or
    // special chars.
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn font_flags(metrics: &FontMetrics) -> FontFlags {
    let mut flags = FontFlags::NON_SYMBOLIC;
    if metrics.is_italic {
        flags |= FontFlags::ITALIC;
    }
    if metrics.is_bold {
        flags |= FontFlags::FORCE_BOLD;
    }
    flags
}

fn stem_v(metrics: &FontMetrics) -> i32 {
    // Standard approximation: 10 + 220 * (weight - 400) / 600
    // We don't have weight info, so use 80 for regular, 120 for bold.
    if metrics.is_bold { 120 } else { 80 }
}

fn write_font_descriptor(
    pdf: &mut Pdf,
    descriptor_ref: Ref,
    font_file_ref: Ref,
    metrics: &FontMetrics,
    subset: &FontSubset,
) -> Result<(), PdfError> {
    let base_name = sanitise_font_name(&metrics.family_name);
    let upm = metrics.units_per_em as f32;

    // Convert font-unit bbox to PDF units (1000 / upm scaling).
    let scale = 1000.0 / upm;
    let bbox_pdf = [
        (metrics.bbox[0] as f32 * scale) as i32,
        (metrics.bbox[1] as f32 * scale) as i32,
        (metrics.bbox[2] as f32 * scale) as i32,
        (metrics.bbox[3] as f32 * scale) as i32,
    ];

    let asc = (metrics.ascender as f32 * scale) as i32;
    let desc = (metrics.descender as f32 * scale) as i32;
    let cap = (metrics.cap_height as f32 * scale) as i32;

    let _ = subset; // reserved for future glyph-count info

    let mut desc_dict = pdf.font_descriptor(descriptor_ref);
    desc_dict.name(Name(base_name.as_bytes()));
    desc_dict.flags(font_flags(metrics));
    desc_dict.bbox(pdf_writer::Rect::new(
        bbox_pdf[0] as f32,
        bbox_pdf[1] as f32,
        bbox_pdf[2] as f32,
        bbox_pdf[3] as f32,
    ));
    desc_dict.italic_angle(metrics.italic_angle);
    desc_dict.ascent(asc as f32);
    desc_dict.descent(desc as f32);
    desc_dict.cap_height(cap as f32);
    desc_dict.stem_v(stem_v(metrics) as f32);
    desc_dict.font_file2(font_file_ref);

    Ok(())
}

fn write_cid_font(
    pdf: &mut Pdf,
    cid_ref: Ref,
    descriptor_ref: Ref,
    metrics: &FontMetrics,
    subset: &FontSubset,
) -> Result<(), PdfError> {
    let base_name = sanitise_font_name(&metrics.family_name);
    let upm = metrics.units_per_em as f32;

    // Collect advance widths sorted by glyph ID for the /W array.
    // /W format: [first_gid [width ...]] for contiguous runs.
    // We use the simple per-glyph format: [gid [width]] for each entry.
    let sorted_widths: BTreeMap<u16, u16> = subset
        .advance_widths
        .iter()
        .map(|(GlyphId(g), &w)| (*g, w))
        .collect();

    let dw: f32 = 1000.0; // default width (em-wide fallback)

    let mut cid = pdf.cid_font(cid_ref);
    cid.subtype(CidFontType::Type2);
    cid.base_font(Name(base_name.as_bytes()));
    cid.system_info(SystemInfo {
        registry: Str(b"Adobe"),
        ordering: Str(b"Identity"),
        supplement: 0,
    });
    cid.font_descriptor(descriptor_ref);
    cid.default_width(dw);
    cid.cid_to_gid_map_predefined(Name(b"Identity"));

    // Build the /W array: sparse per-glyph format [gid [width_in_1000ths]]
    if !sorted_widths.is_empty() {
        let mut w_arr = cid.widths();
        for (gid, adv) in &sorted_widths {
            let pdf_width = (*adv as f32 * 1000.0 / upm);
            w_arr.consecutive(*gid, [pdf_width]);
        }
    }

    Ok(())
}

/// Write a ToUnicode CMap stream mapping glyph IDs to Unicode code points.
///
/// This is what enables text search and copy-paste in PDF readers.
fn write_to_unicode_cmap(pdf: &mut Pdf, cmap_ref: Ref, subset: &FontSubset) {
    // Build sorted list of (glyph_id → char) pairs.
    let mut pairs: Vec<(u16, char)> = subset
        .unicode_map
        .iter()
        .map(|(&ch, &GlyphId(gid))| (gid, ch))
        .collect();
    pairs.sort_by_key(|&(gid, _)| gid);

    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n\
         12 dict begin\n\
         begincmap\n\
         /CIDSystemInfo\n\
         << /Registry (Adobe)\n\
            /Ordering (UCS)\n\
            /Supplement 0\n\
         >> def\n\
         /CMapName /Adobe-Identity-UCS def\n\
         /CMapType 2 def\n\
         1 begincodespacerange\n\
         <0000> <FFFF>\n\
         endcodespacerange\n",
    );

    // Emit in chunks of 100 (PDF spec limit per beginbfchar block).
    for chunk in pairs.chunks(100) {
        cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
        for &(gid, ch) in chunk {
            let code_point = ch as u32;
            if code_point <= 0xFFFF {
                cmap.push_str(&format!("<{:04X}> <{:04X}>\n", gid, code_point));
            } else {
                // Surrogate-pair encoding for code points > U+FFFF.
                let cp = code_point - 0x10000;
                let high = 0xD800 + (cp >> 10);
                let low = 0xDC00 + (cp & 0x3FF);
                cmap.push_str(&format!("<{:04X}> <{:04X}{:04X}>\n", gid, high, low));
            }
        }
        cmap.push_str("endbfchar\n");
    }

    cmap.push_str(
        "endcmap\n\
         CMapName currentdict /CMap defineresource pop\n\
         end\n\
         end\n",
    );

    let cmap_bytes = cmap.into_bytes();
    pdf.stream(cmap_ref, &cmap_bytes);
    // .filter(pdf_writer::Filter::FlateDecode); // Missing actual compression in Phase 7
}
