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

//! Text document layout and PDF content stream generation.

mod collector;
mod layout;
mod measure;
pub mod named_styles;
mod operators;
mod para;
mod renderer;
pub mod style_props;

pub use collector::{collect_used_glyphs, FontKey};
pub use renderer::emit_blocks;

use crate::error::PdfError;
use crate::export_settings::PdfExportSettings;

/// Write a text document (blocks + styles + metadata) to PDF/X-compliant bytes.
pub fn write_text_pdf(
    blocks: &[common_core::Block],
    styles: &std::collections::HashMap<String, common_core::StyleDefinition>,
    metadata: &common_core::Metadata,
    settings: &PdfExportSettings,
    font_resolver: &dyn crate::fonts::resolver::FontResolver,
) -> Result<Vec<u8>, PdfError> {
    use crate::conformance::validate_text;
    use crate::fonts::{create_subset, embed_font};
    use pdf_writer::{Name, Pdf, Ref};

    // 1. Validate — reject hard (non-auto-fixable) violations.
    let violations = validate_text(blocks, styles, metadata, settings);
    let hard_count = violations.iter().filter(|v| !v.auto_fixable).count();
    if hard_count > 0 {
        let msg = violations
            .iter()
            .filter(|v| !v.auto_fixable)
            .map(|v| format!("[{}] {}", v.rule, v.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(PdfError::Conformance(msg));
    }

    // 2. Collect used glyphs per font variant (Pass 1).
    let used_by_font = collect_used_glyphs(blocks, styles);

    let mut pdf = Pdf::new();
    let mut next_ref = 6i32; // 1-5 reserved for catalog/pages/page/content/xmp

    // 3. Subset and embed each required font variant.
    let mut font_map: std::collections::HashMap<FontKey, (String, Ref, crate::fonts::FontSubset)> =
        std::collections::HashMap::new();

    for ((family, weight, italic), used_chars) in &used_by_font {
        let font_bytes = font_resolver
            .resolve(family, *weight, *italic)
            .or_else(|| font_resolver.resolve(font_resolver.fallback_family(), 400, false))
            .ok_or_else(|| {
                PdfError::FontLoad(format!(
                    "Font '{}' not found and fallback '{}' also unavailable",
                    family,
                    font_resolver.fallback_family()
                ))
            })?;

        let _coords =
            crate::fonts::subset::find_variation_coordinates(&font_bytes, *weight > 400, *italic);

        let subset = create_subset(&font_bytes, used_chars)
            .map_err(|e| PdfError::FontLoad(format!("Subset failed for '{}': {}", family, e)))?;

        let (pdf_name, font_ref) = embed_font(&subset, &mut pdf, &mut next_ref)
            .map_err(|e| PdfError::FontLoad(format!("Embed failed for '{}': {}", family, e)))?;

        font_map.insert(
            (family.to_string(), *weight, *italic),
            (pdf_name, font_ref, subset),
        );
    }

    if font_map.is_empty() {
        let fallback_bytes = font_resolver
            .resolve(font_resolver.fallback_family(), 400, false)
            .ok_or_else(|| {
                PdfError::FontLoad(format!(
                    "Fallback font '{}' not available",
                    font_resolver.fallback_family()
                ))
            })?;
        let all_chars: crate::fonts::UsedGlyphs =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 ,."
                .chars()
                .collect();
        let subset = create_subset(&fallback_bytes, &all_chars)?;
        let (pdf_name, font_ref) = embed_font(&subset, &mut pdf, &mut next_ref)?;
        let key: FontKey = (font_resolver.fallback_family().to_lowercase(), 400, false);
        font_map.insert(key, (pdf_name, font_ref, subset));
    }

    // 4. Page geometry (A4 portrait: 595 × 842 pt).
    let page_width_pt = 595.0_f64;
    let page_height_pt = 842.0_f64;
    let margin_pt = 72.0_f64; // 1 inch
    let bleed = settings.bleed_pt;

    // 5. Generate content streams (Pass 2).
    let emit_map: std::collections::HashMap<FontKey, (String, crate::fonts::FontSubset)> = font_map
        .iter()
        .map(|(k, v)| (k.clone(), (v.0.clone(), v.2.clone())))
        .collect();

    let layout_result = emit_blocks(
        blocks,
        styles,
        &emit_map,
        page_width_pt,
        page_height_pt,
        margin_pt,
    )?;

    // 6. Write PDF structure.
    let catalog_ref = Ref::new(1);
    let pages_ref = Ref::new(2);
    let xmp_ref = Ref::new(5);

    let mut page_refs = Vec::new();
    let mut content_refs = Vec::new();
    for _ in 0..layout_result.pages.len() {
        page_refs.push(Ref::new(next_ref));
        next_ref += 1;
        content_refs.push(Ref::new(next_ref));
        next_ref += 1;
    }

    {
        let mut catalog = pdf.catalog(catalog_ref);
        catalog.pages(pages_ref);
        catalog.pair(Name(b"Metadata"), xmp_ref);
        let mut intents = catalog.output_intents();
        let mut intent = intents.push();
        intent.subtype(pdf_writer::types::OutputIntentSubtype::PDFX);
        intent.output_condition_identifier(pdf_writer::TextStr(
            settings.output_condition_identifier.as_str(),
        ));
        if !settings.output_condition.is_empty() {
            intent.output_condition(pdf_writer::TextStr(settings.output_condition.as_str()));
        }
        if !settings.registry_name.is_empty() {
            intent.registry_name(pdf_writer::TextStr(settings.registry_name.as_str()));
        }
    }

    {
        let mut pages = pdf.pages(pages_ref);
        pages.kids(page_refs.iter().copied());
        pages.count(page_refs.len() as i32);
    }

    let xmp = crate::writer::metadata::build_xmp_packet(metadata.title.as_deref(), settings);
    {
        let xmp_bytes = xmp.into_bytes();
        let mut xmp_stream = pdf.stream(xmp_ref, &xmp_bytes);
        xmp_stream.pair(Name(b"Type"), Name(b"Metadata"));
        xmp_stream.pair(Name(b"Subtype"), Name(b"XML"));
    }

    let trim = pdf_writer::Rect::new(0.0, 0.0, page_width_pt as f32, page_height_pt as f32);
    let bleed_rect = pdf_writer::Rect::new(
        -bleed as f32,
        -bleed as f32,
        (page_width_pt + bleed) as f32,
        (page_height_pt + bleed) as f32,
    );

    for (i, page_data) in layout_result.pages.iter().enumerate() {
        let page_ref = page_refs[i];
        let content_ref = content_refs[i];

        let content_compressed = crate::compress::compress(page_data.content_stream.as_bytes());
        pdf.stream(content_ref, &content_compressed)
            .filter(pdf_writer::Filter::FlateDecode);

        let mut page = pdf.page(page_ref);
        page.parent(pages_ref);
        page.media_box(bleed_rect);
        page.trim_box(trim);
        page.bleed_box(bleed_rect);
        page.contents(content_ref);
        page.pair(Name(b"Metadata"), xmp_ref);

        let mut resources = page.resources();
        let mut fonts = resources.fonts();
        for (pdf_name, font_ref, _) in font_map.values() {
            fonts.pair(Name(pdf_name.as_bytes()), *font_ref);
        }
    }

    Ok(pdf.finish())
}
