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

//! Shared test helpers for loki-pdf integration tests.

#![allow(dead_code)]

use common_core::colour_management::{
    BuiltInProfile, Colour, ColourSpace, DocumentColourSettings, IccProfileRef,
};
use loki_pdf::export_settings::{PdfExportSettings, PdfXStandard};
use vector_core::canvas::Canvas;
use vector_core::document::VectorDocument;
use vector_core::object::{CommonProps, ObjectId, RectObject, VectorObject};
use vector_core::style::{ObjectStyle, Paint, StrokeStyle};
use vector_core::transform::Transform;

pub fn cmyk_settings() -> DocumentColourSettings {
    DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..Default::default()
    }
}

pub fn srgb_settings() -> DocumentColourSettings {
    DocumentColourSettings::default()
}

pub fn x4_settings() -> PdfExportSettings {
    PdfExportSettings {
        standard: PdfXStandard::X4_2008,
        output_condition_identifier: "sRGB".to_string(),
        ..Default::default()
    }
}

pub fn x1a_settings() -> PdfExportSettings {
    PdfExportSettings {
        standard: PdfXStandard::X1a2001,
        output_condition_identifier: "FOGRA39".to_string(),
        output_condition: "ISO Coated v2".to_string(),
        registry_name: "http://www.color.org".to_string(),
        bleed_pt: 0.0,
        resolution_dpi: 300,
    }
}

/// Document with a single CMYK filled rectangle in layer 0.
pub fn cmyk_rect_document() -> VectorDocument {
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

/// Document with a single sRGB filled rectangle in layer 0.
pub fn rgb_rect_document() -> VectorDocument {
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

/// A CMYK rectangle object with the given opacity.
pub fn rect_with_opacity(opacity: f64) -> VectorObject {
    VectorObject::Rect(RectObject {
        common: CommonProps {
            id: ObjectId("semi".into()),
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
