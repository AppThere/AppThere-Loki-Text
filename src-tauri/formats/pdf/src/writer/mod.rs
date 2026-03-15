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
pub mod metadata;
pub mod page;
pub mod path_ops;
pub mod resources;

use crate::conformance::validate;
use crate::error::PdfError;
use crate::export_settings::PdfExportSettings;
use content::build_content_stream;
use metadata::build_xmp_packet;
use page::compute_page_geometry;
use pdf_writer::types::OutputIntentSubtype;
use pdf_writer::{Name, Pdf, Rect, Ref, TextStr};
use vector_core::document::VectorDocument;

/// Write a `VectorDocument` to PDF/X-conformant bytes.
///
/// The document is validated against `settings` before any PDF is written.
/// Returns `Err(PdfError::Conformance(...))` if validation fails.
pub fn write_pdf_x(
    document: &VectorDocument,
    settings: &PdfExportSettings,
) -> Result<Vec<u8>, PdfError> {
    // Validate first — this is non-negotiable.
    let report = validate(document, settings);
    // Allow the "empty document" warning but treat all other violations as
    // hard errors.
    let hard_violations: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document")
        .collect();
    if !hard_violations.is_empty() {
        let msg = hard_violations
            .iter()
            .map(|v| format!("[{}] {}", v.rule, v.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(PdfError::Conformance(msg));
    }

    let canvas = &document.canvas;
    let geo = compute_page_geometry(canvas.width, canvas.height, canvas.dpi, settings);
    let page_h_pt = geo.trim_box.height();

    let mut pdf = Pdf::new();

    // Object ref allocation.
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

    // ---- Content stream ----
    let mut all_objects = Vec::new();
    for layer in &document.layers {
        if layer.visible {
            all_objects.extend(layer.objects.iter().cloned());
        }
    }
    let content_str = build_content_stream(&all_objects, page_h_pt)?;
    let content_bytes = content_str.into_bytes();
    pdf.stream(content_ref, &content_bytes);

    // ---- XMP metadata stream ----
    let title = document.metadata.title.as_deref();
    let xmp = build_xmp_packet(title, settings);
    let xmp_bytes = xmp.into_bytes();
    {
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
    }

    Ok(pdf.finish())
}
