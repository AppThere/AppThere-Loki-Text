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

//! Integration tests for the PDF/X conformance validator — Part 2.
//!
//! Covers: empty document warnings, Spot colour with CMYK fallback,
//! ConformanceReport::into_result, and multi-layer document checks.

mod common;

use common::{cmyk_rect_document, rect_with_opacity, rgb_rect_document, srgb_settings};
use common::{x1a_settings, x4_settings};
use common_core::colour_management::Colour;
use loki_pdf::conformance::validate;
use vector_core::canvas::Canvas;
use vector_core::document::VectorDocument;
use vector_core::layer::Layer;
use vector_core::object::{CommonProps, ObjectId, RectObject, VectorObject};
use vector_core::style::{ObjectStyle, Paint, StrokeStyle};
use vector_core::transform::Transform;

// ---------------------------------------------------------------------------
// Test: empty document
// ---------------------------------------------------------------------------

#[test]
fn empty_document_produces_warning() {
    let doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), srgb_settings());
    let report = validate(&doc, &x4_settings());
    assert!(report
        .violations
        .iter()
        .any(|v| v.rule == "X/empty-document"));
}

#[test]
fn document_with_objects_no_empty_warning() {
    let doc = cmyk_rect_document();
    let report = validate(&doc, &x4_settings());
    let empty_viols: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule == "X/empty-document")
        .collect();
    assert!(
        empty_viols.is_empty(),
        "Document with objects should not produce empty-document violation"
    );
}

// ---------------------------------------------------------------------------
// Test: Spot colour with CMYK fallback passes X1a
// ---------------------------------------------------------------------------

#[test]
fn spot_colour_with_cmyk_fallback_passes_x1a() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), common::cmyk_settings());
    let obj = VectorObject::Rect(RectObject {
        common: CommonProps {
            id: ObjectId("spot-rect".into()),
            label: None,
            style: ObjectStyle {
                fill: Paint::Solid {
                    colour: Colour::Spot {
                        name: "PANTONE 186 C".to_string(),
                        tint: 1.0,
                        lab_ref: [38.0, 56.0, 28.0],
                        cmyk_fallback: Box::new(Colour::Cmyk {
                            c: 0.0,
                            m: 0.91,
                            y: 0.76,
                            k: 0.06,
                            alpha: 1.0,
                        }),
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
        width: 100.0,
        height: 100.0,
        rx: 0.0,
        ry: 0.0,
    });
    doc.layers[0].objects.push(obj);
    let report = validate(&doc, &x1a_settings());
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document")
        .collect();
    assert!(
        hard.is_empty(),
        "Spot with CMYK fallback should pass X1a; got: {:?}",
        hard
    );
}

// ---------------------------------------------------------------------------
// Test: into_result() conversion
// ---------------------------------------------------------------------------

#[test]
fn into_result_ok_for_valid_document() {
    let doc = cmyk_rect_document();
    let settings = x4_settings();
    let report = validate(&doc, &settings);
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document")
        .cloned()
        .collect();
    let clean_report = loki_pdf::conformance::ConformanceReport {
        standard: settings.standard,
        violations: hard,
    };
    assert!(clean_report.into_result().is_ok());
}

#[test]
fn into_result_err_for_invalid_document() {
    let doc = rgb_rect_document();
    let report = validate(&doc, &x1a_settings());
    assert!(report.into_result().is_err());
}

// ---------------------------------------------------------------------------
// Test: multi-layer documents
// ---------------------------------------------------------------------------

#[test]
fn objects_in_all_layers_are_checked() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), common::cmyk_settings());

    // Layer 0: CMYK rect (ok).
    doc.layers[0].objects.push(rect_with_opacity(1.0));

    // Layer 1 with RGB colour (should fail X1a).
    let mut layer2 = Layer::new("Layer 2");
    layer2.objects.push(VectorObject::Rect(RectObject {
        common: CommonProps {
            id: ObjectId("rgb-in-layer2".into()),
            label: None,
            style: ObjectStyle {
                fill: Paint::Solid {
                    colour: Colour::Rgb {
                        r: 0.0,
                        g: 0.5,
                        b: 1.0,
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
        x: 0.0,
        y: 0.0,
        width: 50.0,
        height: 50.0,
        rx: 0.0,
        ry: 0.0,
    }));
    doc.layers.push(layer2);

    let report = validate(&doc, &x1a_settings());
    assert!(
        !report.is_conformant(),
        "RGB in layer 2 should be caught by X1a validator"
    );
    assert!(report.violations.iter().any(|v| v.rule == "X1a/no-rgb"));
}
