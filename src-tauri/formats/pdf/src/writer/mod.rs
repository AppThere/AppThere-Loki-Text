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

//! PDF/X writer — converts a validated `VectorDocument` to PDF bytes.
//!
//! The entry point is `write_pdf_x`, which accepts a document that has
//! already passed `conformance::validate`. Calling this with an invalid
//! document will produce a non-conformant PDF.

pub mod colour;
pub mod content;
pub mod image;
pub mod metadata;
pub mod page;
pub mod path_ops;
pub mod resources;
pub(crate) mod text;

use crate::conformance::validate;
use crate::error::PdfError;
use crate::export_settings::PdfExportSettings;
use crate::flatten::{FlattenedItem, RasterRegion};
use content::{build_content_stream, build_flattened_content};
use metadata::build_xmp_packet;
use page::compute_page_geometry;
use pdf_writer::types::OutputIntentSubtype;
use pdf_writer::{Name, Pdf, Rect, Ref, TextStr};
use vector_core::document::VectorDocument;

/// Write a `VectorDocument` to PDF/X-conformant bytes.
///
/// Validation runs first. Auto-fixable violations (colour conversion,
/// transparency flattening) are handled by the pipeline — only genuinely
/// unresolvable issues block export.
pub fn write_pdf_x(
    document: &VectorDocument,
    settings: &PdfExportSettings,
) -> Result<Vec<u8>, PdfError> {
    // 1. Validate — collect all violations.
    let report = validate(document, settings);
    // Hard violations: neither auto-fixable nor the empty-document warning.
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document" && !v.auto_fixable)
        .collect();
    if !hard.is_empty() {
        let msg = hard
            .iter()
            .map(|v| format!("[{}] {}", v.rule, v.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(PdfError::Conformance(msg));
    }

    // 2. Pre-export preparation: expand linked colours + convert colours.
    let prepared = crate::preexport::prepare_for_export(document, settings)?;

    let canvas = &prepared.canvas;
    let geo = compute_page_geometry(canvas.width, canvas.height, canvas.dpi, settings);
    let page_h_pt = geo.trim_box.height();

    let mut pdf = Pdf::new();
    let mut next_ref = 6i32; // 1-5 reserved for catalog/pages/page/content/xmp

    let catalog_ref = Ref::new(1);
    let pages_ref = Ref::new(2);
    let page_ref = Ref::new(3);
    let content_ref = Ref::new(4);
    let xmp_ref = Ref::new(5);

    // ---- Catalog with OutputIntent ----
    {
        let mut catalog = pdf.catalog(catalog_ref);
        catalog.pages(pages_ref);
        catalog.pair(Name(b"Metadata"), xmp_ref);

        let mut intents = catalog.output_intents();
        let mut intent = intents.push();
        intent.subtype(OutputIntentSubtype::PDFX);
        intent.output_condition_identifier(TextStr(settings.output_condition_identifier.as_str()));
        if !settings.output_condition.is_empty() {
            intent.output_condition(TextStr(settings.output_condition.as_str()));
        }
        if !settings.registry_name.is_empty() {
            intent.registry_name(TextStr(settings.registry_name.as_str()));
        }
    }

    // ---- Pages ----
    {
        let mut pages = pdf.pages(pages_ref);
        pages.kids([page_ref]);
        pages.count(1);
    }

    let (content_bytes, image_xobjects) = if settings.standard.requires_cmyk_only() {
        build_x1a_content(&prepared, settings, page_h_pt, &mut pdf, &mut next_ref)?
    } else {
        let mut all = Vec::new();
        for layer in &prepared.layers {
            if layer.visible {
                all.extend(layer.objects.iter().cloned());
            }
        }
        let s = build_content_stream(&all, page_h_pt)?;
        (s.into_bytes(), Vec::new())
    };

    let content_compressed = crate::compress::compress(&content_bytes);
    pdf.stream(content_ref, &content_compressed)
        .filter(pdf_writer::Filter::FlateDecode);

    // ---- XMP metadata ----
    let xmp = build_xmp_packet(prepared.metadata.title.as_deref(), settings);
    {
        let xmp_bytes = xmp.into_bytes();
        let mut xmp_stream = pdf.stream(xmp_ref, &xmp_bytes);
        xmp_stream.pair(Name(b"Type"), Name(b"Metadata"));
        xmp_stream.pair(Name(b"Subtype"), Name(b"XML"));
    }

    // ---- Page ----
    let media = geo.media_box;
    let trim = geo.trim_box;
    let bleed = geo.bleed_box;
    {
        let mut page = pdf.page(page_ref);
        page.parent(pages_ref);
        page.media_box(Rect::new(
            media.x_min as f32,
            media.y_min as f32,
            media.x_max as f32,
            media.y_max as f32,
        ));
        page.trim_box(Rect::new(
            trim.x_min as f32,
            trim.y_min as f32,
            trim.x_max as f32,
            trim.y_max as f32,
        ));
        page.bleed_box(Rect::new(
            bleed.x_min as f32,
            bleed.y_min as f32,
            bleed.x_max as f32,
            bleed.y_max as f32,
        ));
        page.contents(content_ref);
        page.pair(Name(b"Metadata"), xmp_ref);

        // Add XObject resources for any embedded raster images.
        if !image_xobjects.is_empty() {
            let mut resources = page.resources();
            let mut xobjs = resources.x_objects();
            for (name, img_ref) in &image_xobjects {
                xobjs.pair(Name(name.as_bytes()), *img_ref);
            }
        }
    }

    Ok(pdf.finish())
}

/// Write a text document (blocks + styles + metadata) to PDF/X-compliant bytes.
///
/// Performs validation, font loading/embedding, text layout, and PDF structure
/// writing. Single-page output only in Phase 7; multi-page is Phase 8.
pub fn write_text_pdf(
    blocks: &[common_core::Block],
    styles: &std::collections::HashMap<String, common_core::StyleDefinition>,
    metadata: &common_core::Metadata,
    settings: &PdfExportSettings,
    font_resolver: &dyn crate::fonts::resolver::FontResolver,
) -> Result<Vec<u8>, PdfError> {
    use crate::conformance::validate_text;
    use crate::fonts::{create_subset, embed_font};
    use text::{collect_used_glyphs, emit_blocks, FontKey};

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
    let mut font_map: std::collections::HashMap<FontKey, (String, pdf_writer::Ref, crate::fonts::FontSubset)> =
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

        // Find variation coordinates for variable fonts.
        let _coords = crate::fonts::subset::find_variation_coordinates(&font_bytes, *weight > 400, *italic);

        let subset = create_subset(&font_bytes, used_chars)
            .map_err(|e| PdfError::FontLoad(format!("Subset failed for '{}': {}", family, e)))?;

        let (pdf_name, font_ref) = embed_font(&subset, &mut pdf, &mut next_ref)
            .map_err(|e| PdfError::FontLoad(format!("Embed failed for '{}': {}", family, e)))?;

        font_map.insert((family.to_string(), *weight, *italic), (pdf_name, font_ref, subset));
    }

    // Also ensure at least one font is available (for documents with no text styles).
    if font_map.is_empty() {
        let fallback_bytes = font_resolver
            .resolve(font_resolver.fallback_family(), 400, false)
            .ok_or_else(|| PdfError::FontLoad(
                format!("Fallback font '{}' not available", font_resolver.fallback_family())
            ))?;
        let all_chars: crate::fonts::UsedGlyphs = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 ,.".chars().collect();
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
    // We only need the (name, subset) part for emit_blocks
    let emit_map: std::collections::HashMap<FontKey, (String, crate::fonts::FontSubset)> = 
        font_map.iter().map(|(k, v)| (k.clone(), (v.0.clone(), v.2.clone()))).collect();
        
    let layout_result = emit_blocks(blocks, styles, &emit_map, page_width_pt, page_height_pt, margin_pt)?;

    // 6. Write PDF structure.
    let catalog_ref = Ref::new(1);
    let pages_ref = Ref::new(2);
    let xmp_ref = Ref::new(5);
    
    // We need unique refs for each page and each page's content stream.
    let mut page_refs = Vec::new();
    let mut content_refs = Vec::new();
    for _ in 0..layout_result.pages.len() {
        page_refs.push(Ref::new(next_ref));
        next_ref += 1;
        content_refs.push(Ref::new(next_ref));
        next_ref += 1;
    }

    // Catalog with OutputIntent.
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

    // Pages tree.
    {
        let mut pages = pdf.pages(pages_ref);
        pages.kids(page_refs.iter().copied()); // Pass iterator of Refs directly
        pages.count(page_refs.len() as i32);
    }

    // XMP.
    let xmp = metadata::build_xmp_packet(metadata.title.as_deref(), settings);
    {
        let xmp_bytes = xmp.into_bytes();
        let mut xmp_stream = pdf.stream(xmp_ref, &xmp_bytes);
        xmp_stream.pair(Name(b"Type"), Name(b"Metadata"));
        xmp_stream.pair(Name(b"Subtype"), Name(b"XML"));
    }

    // Write each page.
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

        // Content stream for this page.
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

/// Build the content stream for PDF/X-1a: flatten transparency then write.
///
/// Returns `(content_bytes, [(xobject_name, image_ref)])`.
fn build_x1a_content(
    doc: &VectorDocument,
    settings: &PdfExportSettings,
    page_h_pt: f64,
    pdf: &mut Pdf,
    next_ref: &mut i32,
) -> Result<(Vec<u8>, Vec<(String, Ref)>), PdfError> {
    let flattened = crate::flatten::flatten_document(doc, settings.resolution_dpi as f64)?;

    // Assign names and write XObjects for each raster region.
    let mut image_map: Vec<(usize, String)> = Vec::new(); // (addr, name)
    let mut xobject_refs: Vec<(String, Ref)> = Vec::new();
    let mut img_counter = 0u32;

    for flat_layer in &flattened {
        for item in &flat_layer.items {
            if let FlattenedItem::Raster(region) = item {
                let name = format!("Im{}", img_counter);
                img_counter += 1;
                let addr = region as *const RasterRegion as usize;
                let (img_ref, _smask_ref) = image::write_image_xobject(region, pdf, next_ref)?;
                image_map.push((addr, name.clone()));
                xobject_refs.push((name, img_ref));
            }
        }
    }

    let content = build_flattened_content(&flattened, page_h_pt, doc.canvas.dpi, &image_map)?;
    Ok((content.into_bytes(), xobject_refs))
}
