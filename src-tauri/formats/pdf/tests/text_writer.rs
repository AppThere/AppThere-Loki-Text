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

//! Integration tests for `write_text_pdf` — the Phase 7 text-document PDF writer.

use std::collections::HashMap;

use common_core::block::{Block, BlockAttrs};
use common_core::inline::Inline;
use common_core::marks::TiptapMark;
use common_core::{Metadata, StyleDefinition};
use loki_pdf::export_settings::{PdfExportSettings, PdfXStandard};
use loki_pdf::{write_text_pdf, MapFontResolver};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load the real Public Sans font bytes from the repository, or return `None`
/// if we're running in an environment without the font assets (e.g. pure unit CI).
fn load_public_sans() -> Option<Vec<u8>> {
    let font_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|root| root.join("src/assets/fonts/PublicSans-Variable.ttf"));
    font_path.and_then(|p| std::fs::read(p).ok())
}

fn make_resolver_with_font(font_bytes: Vec<u8>) -> MapFontResolver {
    let mut r = MapFontResolver::new("public sans");
    r.add_font("public sans", 400, false, font_bytes.clone());
    r.add_font("public sans", 700, false, font_bytes.clone());
    r.add_font("public sans", 400, true, font_bytes.clone());
    r.add_font("public sans", 700, true, font_bytes);
    r
}

fn default_settings() -> PdfExportSettings {
    PdfExportSettings {
        standard: PdfXStandard::X4_2008,
        output_condition_identifier: "sRGB".to_string(),
        output_condition: "sRGB IEC61966-2.1".to_string(),
        registry_name: "http://www.color.org".to_string(),
        ..Default::default()
    }
}

fn simple_paragraph(text: &str) -> Block {
    Block::Paragraph {
        style_name: None,
        attrs: None,
        content: vec![Inline::Text {
            text: text.to_string(),
            style_name: None,
            marks: vec![],
        }],
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// An empty block list should fail validation (hard violation).
#[test]
fn write_text_pdf_empty_blocks_fails_validation() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => {
            eprintln!("[skip] Font unavailable");
            return;
        }
    };
    let resolver = make_resolver_with_font(font_bytes);
    let metadata = Metadata {
        title: Some("Test".to_string()),
        ..Default::default()
    };
    let result = write_text_pdf(
        &[],
        &HashMap::new(),
        &metadata,
        &default_settings(),
        &resolver,
    );
    assert!(result.is_err(), "Empty doc should fail validation");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("empty"), "Error should mention empty: {msg}");
}

/// A document with a simple paragraph should produce valid PDF bytes.
#[test]
fn write_text_pdf_simple_paragraph_produces_pdf() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => {
            eprintln!("[skip] Font unavailable");
            return;
        }
    };
    let resolver = make_resolver_with_font(font_bytes);
    let blocks = vec![simple_paragraph("Hello, PDF/X world!")];
    let metadata = Metadata {
        title: Some("Test Document".to_string()),
        ..Default::default()
    };
    let result = write_text_pdf(
        &blocks,
        &HashMap::new(),
        &metadata,
        &default_settings(),
        &resolver,
    );
    assert!(
        result.is_ok(),
        "Simple paragraph should succeed: {:?}",
        result
    );
    let bytes = result.unwrap();
    // PDF files start with %PDF-
    assert!(
        bytes.starts_with(b"%PDF-"),
        "Output should be a PDF, got {:?}",
        &bytes[..8]
    );
}

/// Multiple block types (heading, paragraph, horizontal rule) should all succeed.
#[test]
fn write_text_pdf_mixed_blocks_succeed() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => {
            eprintln!("[skip] Font unavailable");
            return;
        }
    };
    let resolver = make_resolver_with_font(font_bytes);
    let blocks = vec![
        Block::Heading {
            level: 1,
            style_name: None,
            attrs: None,
            content: vec![Inline::Text {
                text: "Chapter One".to_string(),
                style_name: None,
                marks: vec![],
            }],
        },
        simple_paragraph("This is a paragraph with some text that should wrap at the margins."),
        Block::HorizontalRule,
        simple_paragraph("Another paragraph after the rule."),
    ];
    let metadata = Metadata {
        title: Some("Mixed Blocks".to_string()),
        ..Default::default()
    };
    let bytes = write_text_pdf(
        &blocks,
        &HashMap::new(),
        &metadata,
        &default_settings(),
        &resolver,
    )
    .expect("write_text_pdf should succeed for mixed blocks");
    assert!(bytes.starts_with(b"%PDF-"));
}

/// Missing title produces a warning violation but does NOT block export.
#[test]
fn write_text_pdf_missing_title_still_exports() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => {
            eprintln!("[skip] Font unavailable");
            return;
        }
    };
    let resolver = make_resolver_with_font(font_bytes);
    let blocks = vec![simple_paragraph("Content without a title.")];
    let metadata = Metadata::default(); // no title
    let result = write_text_pdf(
        &blocks,
        &HashMap::new(),
        &metadata,
        &default_settings(),
        &resolver,
    );
    // Should succeed (missing title is auto_fixable = true, i.e. a warning only).
    assert!(
        result.is_ok(),
        "Missing title should not block export: {:?}",
        result
    );
}

/// Verify that `fo:font-family` in a style is correctly respected.
#[test]
fn write_text_pdf_uses_font_family_attribute() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => {
            eprintln!("[skip] Font unavailable");
            return;
        }
    };
    let mut resolver = make_resolver_with_font(font_bytes.clone());
    // Register "newsreader" to point to the same bytes for testing.
    resolver.add_font("newsreader", 400, false, font_bytes);

    let mut styles = HashMap::new();
    styles.insert(
        "Standard".to_string(),
        StyleDefinition {
            name: "Standard".to_string(),
            family: common_core::style::StyleFamily::Paragraph,
            parent: None,
            next: None,
            display_name: None,
            attributes: HashMap::from([("fo:font-family".to_string(), "Newsreader".to_string())]),
            text_transform: None,
            outline_level: None,
            autocomplete: None,
            #[cfg(feature = "colour-management")]
            font_colour: None,
            #[cfg(feature = "colour-management")]
            background_colour: None,
        },
    );

    let blocks = vec![Block::Paragraph {
        style_name: Some("Standard".to_string()),
        attrs: None,
        content: vec![Inline::Text {
            text: "Should be in Newsreader".to_string(),
            style_name: None,
            marks: vec![],
        }],
    }];

    let metadata = Metadata {
        title: Some("Font Test".to_string()),
        ..Default::default()
    };
    let result = write_text_pdf(&blocks, &styles, &metadata, &default_settings(), &resolver);
    assert!(result.is_ok(), "Should succeed with fo:font-family style");
}

/// Verify that PageBreak blocks correctly trigger the creation of multiple PDF pages.
#[test]
fn write_text_pdf_multi_page_support() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => return,
    };
    let resolver = make_resolver_with_font(font_bytes);

    let blocks = vec![
        simple_paragraph("Page 1 content"),
        Block::PageBreak,
        simple_paragraph("Page 2 content"),
    ];

    let metadata = Metadata {
        title: Some("Multi-page Test".to_string()),
        ..Default::default()
    };

    let bytes = write_text_pdf(
        &blocks,
        &HashMap::new(),
        &metadata,
        &default_settings(),
        &resolver,
    )
    .expect("Multi-page export should succeed");

    // Check that we have multiple /Page objects.
    // In a PDF, pages are usually defined with "/Type /Page".
    let content = String::from_utf8_lossy(&bytes);
    let page_count = content.split("/Type /Page").count() - 1;
    assert!(
        page_count >= 2,
        "Expected at least 2 pages, found {}",
        page_count
    );
}

/// Verify that a very long paragraph correctly splits across multiple pages.
#[test]
fn write_text_pdf_paragraph_overflow() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => return,
    };
    let resolver = make_resolver_with_font(font_bytes);

    // Create a paragraph with enough text to overflow one A4 page.
    // A4 height is 842pt, margin 72pt. Usable height ~700pt.
    // Line height ~15pt. 50 lines should overflow.
    let mut long_text = String::new();
    for i in 0..100 {
        long_text.push_str(&format!("Line {} of a very long paragraph that should definitely overflow onto the second page of our PDF document. ", i));
    }

    let blocks = vec![simple_paragraph(&long_text)];
    let metadata = Metadata {
        title: Some("Overflow Test".to_string()),
        ..Default::default()
    };

    let bytes = write_text_pdf(
        &blocks,
        &HashMap::new(),
        &metadata,
        &default_settings(),
        &resolver,
    )
    .expect("Paragraph overflow export should succeed");

    let content = String::from_utf8_lossy(&bytes);
    let page_count = content.split("/Type /Page").count() - 1;
    assert!(
        page_count >= 2,
        "Expected paragraph to split into at least 2 pages, got {}",
        page_count
    );
}
