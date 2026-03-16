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

//! Integration tests for the PDF/X-1a writer path (transparency flattening,
//! CMYK-only content, and raster image embedding).

mod common;

use common::{cmyk_rect_document, rect_with_opacity, rgb_rect_document, x1a_settings};
use loki_pdf::write_pdf_x;
use vector_core::canvas::Canvas;
use vector_core::document::VectorDocument;

// ---------------------------------------------------------------------------
// Test: X-1a with CMYK-only content (no transparency) round-trips to bytes.
// ---------------------------------------------------------------------------

#[test]
fn x1a_cmyk_no_transparency_produces_bytes() {
    let doc = cmyk_rect_document();
    let settings = x1a_settings();
    let result = write_pdf_x(&doc, &settings);
    assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
    let bytes = result.unwrap();
    assert!(!bytes.is_empty(), "PDF bytes should not be empty");
    assert!(
        bytes.starts_with(b"%PDF-"),
        "output should start with PDF header"
    );
}

// ---------------------------------------------------------------------------
// Test: X-1a with RGB content auto-converts to CMYK and produces bytes.
// ---------------------------------------------------------------------------

#[test]
fn x1a_rgb_content_auto_converts_and_produces_bytes() {
    let doc = rgb_rect_document();
    let settings = x1a_settings();
    // RGB is auto-fixable for X-1a; export should succeed without error.
    let result = write_pdf_x(&doc, &settings);
    assert!(
        result.is_ok(),
        "RGB doc should auto-convert for X-1a, got {:?}",
        result.err()
    );
    let bytes = result.unwrap();
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// Test: X-1a with a transparent object rasterises and embeds image XObject.
// ---------------------------------------------------------------------------

#[test]
fn x1a_transparent_object_rasterises_to_pdf() {
    let mut doc = cmyk_rect_document();
    // Add a semi-transparent rectangle.
    doc.layers[0].objects.push(rect_with_opacity(0.5));
    let settings = x1a_settings();
    let result = write_pdf_x(&doc, &settings);
    assert!(
        result.is_ok(),
        "transparent doc should flatten for X-1a, got {:?}",
        result.err()
    );
    let bytes = result.unwrap();
    // The PDF should contain the XObject resource entry for the raster image.
    let pdf_str = String::from_utf8_lossy(&bytes);
    assert!(
        pdf_str.contains("/XObject"),
        "expected /XObject resource in PDF for rasterised transparency"
    );
}

// ---------------------------------------------------------------------------
// Test: X-1a output for a fully-transparent layer still produces valid PDF.
// ---------------------------------------------------------------------------

#[test]
fn x1a_fully_transparent_layer_produces_valid_pdf() {
    let mut doc = cmyk_rect_document();
    doc.layers[0].objects.push(rect_with_opacity(0.0));
    let settings = x1a_settings();
    let result = write_pdf_x(&doc, &settings);
    assert!(
        result.is_ok(),
        "expected Ok for fully-transparent layer, got {:?}",
        result.err()
    );
    let bytes = result.unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}

// ---------------------------------------------------------------------------
// Test: An empty document still produces a minimal valid PDF for X-1a.
// ---------------------------------------------------------------------------

#[test]
fn x1a_empty_document_produces_valid_pdf() {
    let doc = VectorDocument::blank_a4();
    let settings = x1a_settings();
    let result = write_pdf_x(&doc, &settings);
    assert!(
        result.is_ok(),
        "empty doc should still produce PDF, got {:?}",
        result.err()
    );
    let bytes = result.unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}
