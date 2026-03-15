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

//! Integration tests for the PDF/X conformance validator.
//!
//! All tests exercise `loki_pdf::conformance::validate` directly against
//! synthetic `VectorDocument` instances. These tests must all pass before
//! a single byte of PDF output is written.

use common_core::colour_management::{
    BuiltInProfile, Colour, ColourSpace, DocumentColourSettings, IccProfileRef,
};
use loki_pdf::conformance::validate;
use loki_pdf::export_settings::{PdfExportSettings, PdfXStandard};
use vector_core::canvas::Canvas;
use vector_core::document::VectorDocument;
use vector_core::layer::Layer;
use vector_core::object::{CommonProps, RectObject, VectorObject};
use vector_core::style::{ObjectStyle, Paint, StrokeStyle};
use vector_core::transform::Transform;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn cmyk_settings() -> DocumentColourSettings {
    DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..Default::default()
    }
}

fn srgb_settings() -> DocumentColourSettings {
    DocumentColourSettings::default() // Srgb by default
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

/// Build a document with a single CMYK filled rectangle in layer 1.
fn cmyk_rect_document() -> VectorDocument {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_settings());
    let rect = VectorObject::Rect(RectObject {
        common: CommonProps {
            id: vector_core::object::ObjectId("r1".into()),
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
    });
    doc.layers[0].objects.push(rect);
    doc
}

/// Build a document with an sRGB filled rectangle.
fn rgb_rect_document() -> VectorDocument {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), srgb_settings());
    let rect = VectorObject::Rect(RectObject {
        common: CommonProps {
            id: vector_core::object::ObjectId("r1".into()),
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
    });
    doc.layers[0].objects.push(rect);
    doc
}

fn rect_with_opacity(opacity: f64) -> VectorObject {
    VectorObject::Rect(RectObject {
        common: CommonProps {
            id: vector_core::object::ObjectId("semi".into()),
            label: None,
            style: ObjectStyle {
                fill: Paint::Solid {
                    colour: Colour::Cmyk {
                        c: 0.5,
                        m: 0.0,
                        y: 0.0,
                        k: 0.0,
                        alpha: 1.0,
                    },
                },
                stroke: StrokeStyle::none(),
                opacity,
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
    })
}

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
    // sRGB working space violates X1a.
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
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_settings());
    let obj = VectorObject::Rect(RectObject {
        common: CommonProps {
            id: vector_core::object::ObjectId("lab-rect".into()),
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
    let settings = x1a_settings();
    let report = validate(&doc, &settings);
    assert!(!report.is_conformant());
    assert!(report.violations.iter().any(|v| v.rule == "X1a/no-lab"));
}

// ---------------------------------------------------------------------------
// Test: transparency
// ---------------------------------------------------------------------------

#[test]
fn x1a_opacity_less_than_1_fails() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_settings());
    doc.layers[0].objects.push(rect_with_opacity(0.5));
    let settings = x1a_settings();
    let report = validate(&doc, &settings);
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
    let settings = x4_settings();
    let report = validate(&doc, &settings);
    let hard: Vec<_> = report
        .violations
        .iter()
        .filter(|v| v.rule != "X/empty-document")
        .collect();
    assert!(hard.is_empty(), "X4 allows transparency; got: {:?}", hard);
}

#[test]
fn x1a_opacity_exactly_1_passes_transparency_check() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_settings());
    doc.layers[0].objects.push(rect_with_opacity(1.0));
    let settings = x1a_settings();
    let report = validate(&doc, &settings);
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
            id: vector_core::object::ObjectId("linked-rect".into()),
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

// ---------------------------------------------------------------------------
// Test: empty document
// ---------------------------------------------------------------------------

#[test]
fn empty_document_produces_warning() {
    let doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), srgb_settings());
    let settings = x4_settings();
    let report = validate(&doc, &settings);
    assert!(report
        .violations
        .iter()
        .any(|v| v.rule == "X/empty-document"));
}

#[test]
fn document_with_objects_no_empty_warning() {
    let doc = cmyk_rect_document();
    let settings = x4_settings();
    let report = validate(&doc, &settings);
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
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_settings());
    let obj = VectorObject::Rect(RectObject {
        common: CommonProps {
            id: vector_core::object::ObjectId("spot-rect".into()),
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
    let settings = x1a_settings();
    let report = validate(&doc, &settings);
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
    // Build a fresh report with only hard violations to test into_result.
    let clean_report = loki_pdf::conformance::ConformanceReport {
        standard: settings.standard,
        violations: hard,
    };
    assert!(clean_report.into_result().is_ok());
}

#[test]
fn into_result_err_for_invalid_document() {
    let doc = rgb_rect_document();
    let settings = x1a_settings();
    let report = validate(&doc, &settings);
    assert!(report.into_result().is_err());
}

// ---------------------------------------------------------------------------
// Test: multi-layer documents
// ---------------------------------------------------------------------------

#[test]
fn objects_in_all_layers_are_checked() {
    let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_settings());

    // Layer 0: CMYK rect (ok).
    doc.layers[0].objects.push(rect_with_opacity(1.0));

    // Layer 1 with RGB colour (should fail X1a).
    let mut layer2 = Layer::new("Layer 2");
    layer2.objects.push(VectorObject::Rect(RectObject {
        common: CommonProps {
            id: vector_core::object::ObjectId("rgb-in-layer2".into()),
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
