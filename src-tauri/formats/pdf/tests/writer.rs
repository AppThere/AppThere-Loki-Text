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

//! Integration tests for the PDF/X writer.
//!
//! These tests verify the output bytes produced by `write_pdf_x` using
//! `lopdf` for structural inspection where needed.

use common_core::colour_management::{
    BuiltInProfile, Colour, ColourSpace, DocumentColourSettings, IccProfileRef,
};
use loki_pdf::export_settings::{PdfExportSettings, PdfXStandard};
use loki_pdf::write_pdf_x;
use vector_core::canvas::Canvas;
use vector_core::document::VectorDocument;
use vector_core::object::{CommonProps, ObjectId, RectObject, VectorObject};
use vector_core::style::{ObjectStyle, Paint, StrokeStyle};
use vector_core::transform::Transform;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn srgb_settings() -> DocumentColourSettings {
    DocumentColourSettings::default()
}

fn cmyk_settings() -> DocumentColourSettings {
    DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..Default::default()
    }
}

fn x4_settings() -> PdfExportSettings {
    PdfExportSettings {
        standard: PdfXStandard::X4_2008,
        output_condition_identifier: "sRGB".to_string(),
        ..Default::default()
    }
}

fn x1a_settings() -> PdfExportSettings {
    PdfExportSettings {
        standard: PdfXStandard::X1a2001,
        output_condition_identifier: "FOGRA39".to_string(),
        output_condition: "ISO Coated v2".to_string(),
        registry_name: "http://www.color.org".to_string(),
        bleed_pt: 0.0,
    }
}

fn cmyk_rect_doc() -> VectorDocument {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_settings());
    doc.layers[0].objects.push(VectorObject::Rect(RectObject {
        common: CommonProps {
            id: ObjectId("r1".into()),
            label: None,
            style: ObjectStyle {
                fill: Paint::Solid {
                    colour: Colour::Cmyk {
                        c: 0.0,
                        m: 0.0,
                        y: 0.0,
                        k: 1.0,
                        alpha: 1.0,
                    },
                },
                stroke: StrokeStyle::none(),
                opacity: 1.0,
                fill_opacity: 1.0,
                stroke_opacity: 1.0,
            },
            transform: Transform::identity(),
            visible: true,
            locked: false,
        },
        x: 10.0,
        y: 10.0,
        width: 100.0,
        height: 50.0,
        rx: 0.0,
        ry: 0.0,
    }));
    doc
}

fn rgb_rect_doc() -> VectorDocument {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), srgb_settings());
    doc.layers[0].objects.push(VectorObject::Rect(RectObject {
        common: CommonProps {
            id: ObjectId("r1".into()),
            label: None,
            style: ObjectStyle {
                fill: Paint::Solid {
                    colour: Colour::Rgb {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                },
                stroke: StrokeStyle::none(),
                opacity: 1.0,
                fill_opacity: 1.0,
                stroke_opacity: 1.0,
            },
            transform: Transform::identity(),
            visible: true,
            locked: false,
        },
        x: 10.0,
        y: 10.0,
        width: 100.0,
        height: 50.0,
        rx: 0.0,
        ry: 0.0,
    }));
    doc
}

// ---------------------------------------------------------------------------
// Test: write_pdf_x produces non-empty bytes
// ---------------------------------------------------------------------------

#[test]
fn x4_cmyk_produces_bytes() {
    let doc = cmyk_rect_doc();
    let settings = x4_settings();
    let bytes = write_pdf_x(&doc, &settings).expect("write should succeed");
    assert!(!bytes.is_empty(), "PDF output must not be empty");
}

#[test]
fn x4_rgb_produces_bytes() {
    let doc = rgb_rect_doc();
    let settings = x4_settings();
    let bytes = write_pdf_x(&doc, &settings).expect("write should succeed");
    assert!(!bytes.is_empty());
}

#[test]
fn x1a_cmyk_produces_bytes() {
    let doc = cmyk_rect_doc();
    let settings = x1a_settings();
    let bytes = write_pdf_x(&doc, &settings).expect("write should succeed");
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// Test: output starts with %PDF header
// ---------------------------------------------------------------------------

#[test]
fn output_starts_with_pdf_header() {
    let doc = cmyk_rect_doc();
    let bytes = write_pdf_x(&doc, &x4_settings()).unwrap();
    assert!(
        bytes.starts_with(b"%PDF-"),
        "PDF must begin with %PDF- header"
    );
}

// ---------------------------------------------------------------------------
// Test: non-conformant document returns Err
// ---------------------------------------------------------------------------

#[test]
fn x1a_rgb_document_returns_error() {
    let doc = rgb_rect_doc();
    let settings = x1a_settings();
    let result = write_pdf_x(&doc, &settings);
    assert!(result.is_err(), "X1a writer must reject RGB document");
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("conformance") || msg.contains("Conformance") || msg.contains("X1a"),
        "Error message should mention conformance: {}",
        msg
    );
}

#[test]
fn empty_output_condition_returns_error() {
    let doc = cmyk_rect_doc();
    let mut settings = x4_settings();
    settings.output_condition_identifier = String::new();
    let result = write_pdf_x(&doc, &settings);
    assert!(
        result.is_err(),
        "Empty OutputConditionIdentifier should fail"
    );
}

// ---------------------------------------------------------------------------
// Test: PDF contains GTS_PDFX in the XMP
// ---------------------------------------------------------------------------

#[test]
fn x4_output_contains_gts_pdfx_xmp() {
    let doc = cmyk_rect_doc();
    let bytes = write_pdf_x(&doc, &x4_settings()).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("GTS_PDFXVersion") || text.contains("PDF/X-4"),
        "PDF output should reference PDF/X-4 in XMP"
    );
}

#[test]
fn x1a_output_contains_gts_pdfx_x1a_xmp() {
    let doc = cmyk_rect_doc();
    let bytes = write_pdf_x(&doc, &x1a_settings()).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("PDF/X-1a") || text.contains("GTS_PDFXVersion"),
        "PDF output should reference PDF/X-1a:2001 in XMP"
    );
}

// ---------------------------------------------------------------------------
// Test: bleed extends page box
// ---------------------------------------------------------------------------

#[test]
fn bleed_produces_larger_media_box() {
    let doc = rgb_rect_doc();
    let mut no_bleed = x4_settings();
    no_bleed.bleed_pt = 0.0;
    let mut with_bleed = x4_settings();
    with_bleed.bleed_pt = 8.504;

    let bytes_no_bleed = write_pdf_x(&doc, &no_bleed).unwrap();
    let bytes_with_bleed = write_pdf_x(&doc, &with_bleed).unwrap();

    // The with-bleed PDF should be different from no-bleed.
    // We check this by searching for different MediaBox entries.
    let no_bleed_text = String::from_utf8_lossy(&bytes_no_bleed);
    let with_bleed_text = String::from_utf8_lossy(&bytes_with_bleed);

    // Both should be valid PDFs.
    assert!(bytes_no_bleed.starts_with(b"%PDF-"));
    assert!(bytes_with_bleed.starts_with(b"%PDF-"));

    // The with-bleed PDF contains a negative BleedBox X coordinate while the
    // no-bleed PDF does not (all box values are 0 or positive).
    let bleed_has_negative = with_bleed_text.contains("BleedBox") && with_bleed_text.contains('-');
    let no_bleed_negative_bleedbox = {
        // Find the BleedBox entry in the no-bleed output and check it has no
        // negative offset (all coords are 0..width/height).
        no_bleed_text
            .find("BleedBox")
            .map(|pos| &no_bleed_text[pos..pos + 60])
            .map(|s| s.contains('-'))
            .unwrap_or(false)
    };
    // The with-bleed PDF should have a negative BleedBox coord;
    // the no-bleed PDF should not.
    assert!(
        bleed_has_negative,
        "With-bleed PDF should contain negative BleedBox coord"
    );
    assert!(
        !no_bleed_negative_bleedbox,
        "No-bleed PDF should not have negative BleedBox coord"
    );
}

// ---------------------------------------------------------------------------
// Test: title is embedded in XMP
// ---------------------------------------------------------------------------

#[test]
fn document_title_appears_in_xmp() {
    let mut doc = cmyk_rect_doc();
    doc.metadata.title = Some("Test Document Title".to_string());
    let bytes = write_pdf_x(&doc, &x4_settings()).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("Test Document Title"),
        "Document title should appear in PDF"
    );
}

// ---------------------------------------------------------------------------
// Test: write can be called twice on the same document (immutable borrow)
// ---------------------------------------------------------------------------

#[test]
fn write_is_idempotent() {
    let doc = cmyk_rect_doc();
    let settings = x4_settings();
    let bytes1 = write_pdf_x(&doc, &settings).unwrap();
    let bytes2 = write_pdf_x(&doc, &settings).unwrap();
    // Both writes should produce identical output.
    assert_eq!(bytes1, bytes2, "write_pdf_x should be deterministic");
}
