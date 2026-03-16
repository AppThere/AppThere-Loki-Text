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

    // 3. Build content — branch on X-1a (flatten) vs X-4 (vector).
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

    pdf.stream(content_ref, &content_bytes);

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
