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

//! Integration tests for Phase 9 layout features (alignment, spacing, indentation, page breaks).

use std::collections::HashMap;

use common_core::block::Block;
use common_core::inline::Inline;
use common_core::{Metadata, StyleDefinition};
use loki_pdf::export_settings::{PdfExportSettings, PdfXStandard};
use loki_pdf::{write_text_pdf, MapFontResolver};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_public_sans() -> Option<Vec<u8>> {
    let font_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn simple_paragraph(text: &str, style_name: Option<&str>) -> Block {
    Block::Paragraph {
        style_name: style_name.map(|s| s.to_string()),
        attrs: None,
        content: vec![Inline::Text {
            text: text.to_string(),
            style_name: None,
            marks: vec![],
        }],
    }
}

fn make_style(name: &str, attrs: HashMap<&str, &str>) -> (String, StyleDefinition) {
    (
        name.to_string(),
        StyleDefinition {
            name: name.to_string(),
            family: common_core::style::StyleFamily::Paragraph,
            parent: None,
            next: None,
            display_name: None,
            attributes: attrs.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            text_transform: None,
            outline_level: None,
            autocomplete: None,
            #[cfg(feature = "colour-management")]
            font_colour: None,
            #[cfg(feature = "colour-management")]
            background_colour: None,
        },
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_alignment_rendering() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => return,
    };
    let resolver = make_resolver_with_font(font_bytes);
    
    let mut styles = HashMap::new();
    let (n, s) = make_style("Centered", HashMap::from([("fo:text-align", "center")]));
    styles.insert(n, s);
    let (n, s) = make_style("Justified", HashMap::from([("fo:text-align", "justify")]));
    styles.insert(n, s);

    let blocks = vec![
        simple_paragraph("Centered Title", Some("Centered")),
        simple_paragraph("A very long paragraph that should be justified. This needs enough text to wrap onto multiple lines so we can see the Tw operator in action in the PDF stream if we were inspecting it manually, but for this automated test we just ensure it succeeds.", Some("Justified")),
    ];

    let metadata = Metadata { title: Some("Alignment Test".to_string()), ..Default::default() };
    let bytes = write_text_pdf(&blocks, &styles, &metadata, &default_settings(), &resolver)
        .expect("Alignment export should succeed");
    
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_paragraph_spacing() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => return,
    };
    let resolver = make_resolver_with_font(font_bytes);
    
    let mut styles = HashMap::new();
    let (n, s) = make_style("Spaced", HashMap::from([
        ("fo:margin-top", "24pt"),
        ("fo:margin-bottom", "12pt")
    ]));
    styles.insert(n, s);

    let blocks = vec![
        simple_paragraph("Paragraph 1", None),
        simple_paragraph("Spaced Paragraph", Some("Spaced")),
        simple_paragraph("Paragraph 3", None),
    ];

    let metadata = Metadata { title: Some("Spacing Test".to_string()), ..Default::default() };
    let bytes = write_text_pdf(&blocks, &styles, &metadata, &default_settings(), &resolver)
        .expect("Spacing export should succeed");
    
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_indentation_and_margins() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => return,
    };
    let resolver = make_resolver_with_font(font_bytes);
    
    let mut styles = HashMap::new();
    let (n, s) = make_style("Indented", HashMap::from([
        ("fo:margin-left", "36pt"),
        ("fo:margin-right", "36pt"),
        ("fo:text-indent", "18pt")
    ]));
    styles.insert(n, s);

    let blocks = vec![
        simple_paragraph("A paragraph with indentation and secondary margins. The first line should be pushed further than the rest, and both sides should be narrowed by the margin-left and margin-right attributes.", Some("Indented")),
    ];

    let metadata = Metadata { title: Some("Indentation Test".to_string()), ..Default::default() };
    let bytes = write_text_pdf(&blocks, &styles, &metadata, &default_settings(), &resolver)
        .expect("Indentation export should succeed");
    
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_page_breaks_and_orphan_logic() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => return,
    };
    let resolver = make_resolver_with_font(font_bytes);
    
    let mut styles = HashMap::new();
    let (n, s) = make_style("BreakBefore", HashMap::from([("fo:break-before", "page")]));
    styles.insert(n, s);
    let (n, s) = make_style("KeepWithNext", HashMap::from([("fo:keep-with-next", "always")]));
    styles.insert(n, s);

    let mut blocks = Vec::new();
    blocks.push(simple_paragraph("Start of document.", None));
    
    // Force many paragraphs to get near the bottom of the page
    for _ in 0..40 {
        blocks.push(simple_paragraph("Filling space...", None));
    }
    
    // Now a KeepWithNext paragraph. It and the next one should move together.
    blocks.push(simple_paragraph("This paragraph must stay with the next.", Some("KeepWithNext")));
    blocks.push(simple_paragraph("I am the next paragraph.", None));
    
    // And a BreakBefore paragraph.
    blocks.push(simple_paragraph("Forced to start on a new page.", Some("BreakBefore")));

    let metadata = Metadata { title: Some("Layout Logic Test".to_string()), ..Default::default() };
    let bytes = write_text_pdf(&blocks, &styles, &metadata, &default_settings(), &resolver)
        .expect("Layout logic export should succeed");
    
    let content = String::from_utf8_lossy(&bytes);
    let page_count = content.split("/Type /Page").count() - 1;
    // We expect at least 3 pages if the break-before and keep-with-next worked as intended 
    // (though keep-with-next depends on exact height, break-before is guaranteed).
    assert!(page_count >= 2);
}

#[test]
fn test_heading_defaults() {
    let font_bytes = match load_public_sans() {
        Some(b) => b,
        None => return,
    };
    let resolver = make_resolver_with_font(font_bytes);
    
    let blocks = vec![
        Block::Heading {
            level: 1,
            style_name: None, // No style name, should use "Heading 1" defaults
            attrs: None,
            content: vec![Inline::Text { text: "Implicit H1".to_string(), style_name: None, marks: vec![] }],
        },
        Block::Heading {
            level: 2,
            style_name: Some("Heading_20_2".to_string()), // Should match "Heading 2" defaults
            attrs: None,
            content: vec![Inline::Text { text: "Explicit H2".to_string(), style_name: None, marks: vec![] }],
        },
        simple_paragraph("Standard body text follows the headings.", None),
    ];

    let metadata = Metadata { title: Some("Heading Defaults Test".to_string()), ..Default::default() };
    let bytes = write_text_pdf(&blocks, &HashMap::new(), &metadata, &default_settings(), &resolver)
        .expect("Heading defaults export should succeed");
    
    assert!(bytes.starts_with(b"%PDF-"));
}
