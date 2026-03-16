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

//! Pagination tests for `write_text_pdf` — multi-page and overflow behaviour.

use common_core::block::Block;
use common_core::inline::Inline;
use common_core::{Metadata, StyleDefinition};
use loki_pdf::export_settings::{PdfExportSettings, PdfXStandard};
use loki_pdf::{write_text_pdf, MapFontResolver};
use std::collections::HashMap;

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
    let mut long_text = String::new();
    for i in 0..100 {
        long_text.push_str(&format!(
            "Line {} of a very long paragraph that should definitely overflow onto the next page. ",
            i
        ));
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
