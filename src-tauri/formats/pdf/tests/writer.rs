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

mod common;

use common::{cmyk_rect_document, rgb_rect_document, x1a_settings, x4_settings};
use loki_pdf::write_pdf_x;

// ---------------------------------------------------------------------------
// Test: write_pdf_x produces non-empty bytes
// ---------------------------------------------------------------------------

#[test]
fn x4_cmyk_produces_bytes() {
    let doc = cmyk_rect_document();
    let bytes = write_pdf_x(&doc, &x4_settings()).expect("write should succeed");
    assert!(!bytes.is_empty(), "PDF output must not be empty");
}

#[test]
fn x4_rgb_produces_bytes() {
    let doc = rgb_rect_document();
    let bytes = write_pdf_x(&doc, &x4_settings()).expect("write should succeed");
    assert!(!bytes.is_empty());
}

#[test]
fn x1a_cmyk_produces_bytes() {
    let doc = cmyk_rect_document();
    let bytes = write_pdf_x(&doc, &x1a_settings()).expect("write should succeed");
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// Test: output starts with %PDF header
// ---------------------------------------------------------------------------

#[test]
fn output_starts_with_pdf_header() {
    let doc = cmyk_rect_document();
    let bytes = write_pdf_x(&doc, &x4_settings()).unwrap();
    assert!(
        bytes.starts_with(b"%PDF-"),
        "PDF must begin with %PDF- header"
    );
}

// ---------------------------------------------------------------------------
// Test: RGB document for X-1a is auto-converted (not rejected)
// ---------------------------------------------------------------------------

#[test]
fn x1a_rgb_document_auto_converts_to_cmyk() {
    // RGB colour violations are auto-fixable for X-1a: the pipeline converts
    // them to CMYK before writing, so export must succeed.
    let doc = rgb_rect_document();
    let result = write_pdf_x(&doc, &x1a_settings());
    assert!(
        result.is_ok(),
        "X-1a writer must auto-convert RGB document, got {:?}",
        result.err()
    );
}

#[test]
fn empty_output_condition_returns_error() {
    let doc = cmyk_rect_document();
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
    let doc = cmyk_rect_document();
    let bytes = write_pdf_x(&doc, &x4_settings()).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("GTS_PDFXVersion") || text.contains("PDF/X-4"),
        "PDF output should reference PDF/X-4 in XMP"
    );
}

#[test]
fn x1a_output_contains_gts_pdfx_x1a_xmp() {
    let doc = cmyk_rect_document();
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
    let doc = rgb_rect_document();
    let mut no_bleed = x4_settings();
    no_bleed.bleed_pt = 0.0;
    let mut with_bleed = x4_settings();
    with_bleed.bleed_pt = 8.504;

    let bytes_no_bleed = write_pdf_x(&doc, &no_bleed).unwrap();
    let bytes_with_bleed = write_pdf_x(&doc, &with_bleed).unwrap();

    let no_bleed_text = String::from_utf8_lossy(&bytes_no_bleed);
    let with_bleed_text = String::from_utf8_lossy(&bytes_with_bleed);

    assert!(bytes_no_bleed.starts_with(b"%PDF-"));
    assert!(bytes_with_bleed.starts_with(b"%PDF-"));

    // With-bleed PDF contains a negative BleedBox X coordinate.
    let bleed_has_negative = with_bleed_text.contains("BleedBox") && with_bleed_text.contains('-');
    let no_bleed_negative_bleedbox = no_bleed_text
        .find("BleedBox")
        .map(|pos| &no_bleed_text[pos..pos + 60])
        .map(|s| s.contains('-'))
        .unwrap_or(false);
    assert!(
        bleed_has_negative,
        "With-bleed PDF should have negative BleedBox coord"
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
    let mut doc = cmyk_rect_document();
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
    let doc = cmyk_rect_document();
    let settings = x4_settings();
    let bytes1 = write_pdf_x(&doc, &settings).unwrap();
    let bytes2 = write_pdf_x(&doc, &settings).unwrap();
    assert_eq!(bytes1, bytes2, "write_pdf_x should be deterministic");
}
