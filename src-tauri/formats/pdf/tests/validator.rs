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

//! Integration tests for the PDF/X conformance validator — Part 1.
//!
//! Covers: output condition, colour space (X4/X1a), transparency, linked
//! colours, and bleed validation. Part 2 (validator_objects.rs) covers
//! empty document, spot colours, into_result, and multi-layer checks.

mod common;

use common::{cmyk_rect_document, rect_with_opacity, rgb_rect_document, srgb_settings};
use common::{x1a_settings, x4_settings};
use common_core::colour_management::Colour;
use loki_pdf::conformance::validate;
use vector_core::canvas::Canvas;
use vector_core::document::VectorDocument;
use vector_core::object::{CommonProps, ObjectId, RectObject, VectorObject};
use vector_core::style::{ObjectStyle, Paint, StrokeStyle};
use vector_core::transform::Transform;

// ---------------------------------------------------------------------------
// Test: empty output_condition_identifier
// ---------------------------------------------------------------------------

#[test]
fn empty_output_condition_identifier_fails() {
    let doc = cmyk_rect_document();
    let mut settings = x4_settings();
    settings.output_condition_identifier = String::new();
    let report = validate(&doc, &settings);
    assert!(
        !report.is_conformant(),
        "Empty OutputConditionIdentifier should fail"
    );
    assert!(report
        .violations
        .iter()
        .any(|v| v.rule == "X/output-condition"));
}

#[test]
fn whitespace_only_output_condition_identifier_fails() {
    let doc = cmyk_rect_document();
    let mut settings = x4_settings();
    settings.output_condition_identifier = "   ".to_string();
    let report = validate(&doc, &settings);
    assert!(!report.is_conformant());
}

// ---------------------------------------------------------------------------
// Test: X4 allows RGB + sRGB working space
// ---------------------------------------------------------------------------

#[test]
fn x4_rgb_document_passes() {
    let doc = rgb_rect_document();
    let settings = x4_settings();
    let report = validate(&doc, &settings);
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document")
        .collect();
    assert!(hard.is_empty(), "X4 should allow RGB; got: {:?}", hard);
}

#[test]
fn x4_cmyk_document_passes() {
    let doc = cmyk_rect_document();
    let settings = x4_settings();
    let report = validate(&doc, &settings);
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document")
        .collect();
    assert!(hard.is_empty(), "X4 CMYK should pass; got: {:?}", hard);
}

// ---------------------------------------------------------------------------
// Test: X1a requires CMYK
// ---------------------------------------------------------------------------

#[test]
fn x1a_rgb_colour_fails() {
    let doc = rgb_rect_document();
    let settings = x1a_settings();
    let report = validate(&doc, &settings);
    assert!(!report.is_conformant());
    let has_rgb_violation = report
        .violations
        .iter()
        .any(|v| v.rule == "X1a/no-rgb" || v.rule == "X1a/working-space-must-be-cmyk");
    assert!(
        has_rgb_violation,
        "Should flag RGB/sRGB violation: {:?}",
        report.violations
    );
}

#[test]
fn x1a_cmyk_document_passes() {
    let doc = cmyk_rect_document();
    let settings = x1a_settings();
    let report = validate(&doc, &settings);
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document")
        .collect();
    assert!(hard.is_empty(), "X1a CMYK should pass; got: {:?}", hard);
}

#[test]
fn x1a_lab_colour_fails() {
    let cmyk_doc_settings = common::cmyk_settings();
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_doc_settings);
    let obj = VectorObject::Rect(RectObject {
        common: CommonProps {
            id: ObjectId("lab-rect".into()),
            label: None,
            style: ObjectStyle {
                fill: Paint::Solid {
                    colour: Colour::Lab {
                        l: 50.0,
                        a: 20.0,
                        b: -10.0,
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
        x: 0.0,
        y: 0.0,
        width: 50.0,
        height: 50.0,
        rx: 0.0,
        ry: 0.0,
    });
    doc.layers[0].objects.push(obj);
    let report = validate(&doc, &x1a_settings());
    assert!(!report.is_conformant());
    assert!(report.violations.iter().any(|v| v.rule == "X1a/no-lab"));
}

// ---------------------------------------------------------------------------
// Test: transparency
// ---------------------------------------------------------------------------

#[test]
fn x1a_opacity_less_than_1_fails() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), common::cmyk_settings());
    doc.layers[0].objects.push(rect_with_opacity(0.5));
    let report = validate(&doc, &x1a_settings());
    assert!(!report.is_conformant());
    assert!(report
        .violations
        .iter()
        .any(|v| v.rule == "X1a/no-transparency"));
}

#[test]
fn x4_opacity_less_than_1_passes() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), srgb_settings());
    doc.layers[0].objects.push(rect_with_opacity(0.5));
    let report = validate(&doc, &x4_settings());
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document")
        .collect();
    assert!(hard.is_empty(), "X4 allows transparency; got: {:?}", hard);
}

#[test]
fn x1a_opacity_exactly_1_passes_transparency_check() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), common::cmyk_settings());
    doc.layers[0].objects.push(rect_with_opacity(1.0));
    let report = validate(&doc, &x1a_settings());
    let transparency_viols: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule.starts_with("X1a/no-transparency") || v.rule.starts_with("X1a/no-fill"))
        .collect();
    assert!(
        transparency_viols.is_empty(),
        "Opacity=1.0 should not trigger transparency violation: {:?}",
        transparency_viols
    );
}

// ---------------------------------------------------------------------------
// Test: Linked colour is rejected
// ---------------------------------------------------------------------------

#[test]
fn linked_colour_fails_x4() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), srgb_settings());
    let obj = VectorObject::Rect(RectObject {
        common: CommonProps {
            id: ObjectId("linked-rect".into()),
            label: None,
            style: ObjectStyle {
                fill: Paint::Solid {
                    colour: Colour::Linked {
                        id: "swatch-abc".to_string(),
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
        x: 0.0,
        y: 0.0,
        width: 50.0,
        height: 50.0,
        rx: 0.0,
        ry: 0.0,
    });
    doc.layers[0].objects.push(obj);
    let report = validate(&doc, &x4_settings());
    assert!(!report.is_conformant());
    assert!(report
        .violations
        .iter()
        .any(|v| v.rule == "X/unresolved-linked-colour"));
}

// ---------------------------------------------------------------------------
// Test: bleed validation
// ---------------------------------------------------------------------------

#[test]
fn negative_bleed_fails() {
    let doc = cmyk_rect_document();
    let mut settings = x4_settings();
    settings.bleed_pt = -1.0;
    let report = validate(&doc, &settings);
    assert!(!report.is_conformant());
    assert!(report
        .violations
        .iter()
        .any(|v| v.rule == "X/negative-bleed"));
}

#[test]
fn zero_bleed_passes() {
    let doc = cmyk_rect_document();
    let mut settings = x4_settings();
    settings.bleed_pt = 0.0;
    let report = validate(&doc, &settings);
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule == "X/negative-bleed")
        .collect();
    assert!(hard.is_empty());
}

#[test]
fn positive_bleed_passes() {
    let doc = cmyk_rect_document();
    let mut settings = x4_settings();
    settings.bleed_pt = 8.504; // ~3mm
    let report = validate(&doc, &settings);
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule == "X/negative-bleed")
        .collect();
    assert!(hard.is_empty());
}
