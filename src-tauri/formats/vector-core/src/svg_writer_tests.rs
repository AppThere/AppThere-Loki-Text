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

//! Unit tests for svg_writer.rs.

use super::*;
use crate::canvas::Canvas;
use crate::document::VectorDocument;
use crate::layer::Layer;
use crate::object::{CommonProps, EllipseObject, RectObject};
use crate::style::ObjectStyle;
use crate::svg_parser::parse;
use common_core::colour_management::{ColourContext, DocumentColourSettings, IccProfileStore};

fn make_ctx() -> ColourContext {
    let settings = DocumentColourSettings::default();
    let mut store = IccProfileStore::new();
    ColourContext::new_for_display(&settings, &mut store).unwrap()
}

fn make_doc() -> VectorDocument {
    let mut layer = Layer::new("Layer 1");
    layer.id = "layer1".to_string();

    let mut rect_common = CommonProps::new("rect1");
    rect_common.style = ObjectStyle::default_fill();

    layer.objects.push(VectorObject::Rect(RectObject {
        common: rect_common,
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 50.0,
        rx: 0.0,
        ry: 0.0,
    }));

    let mut ellipse_common = CommonProps::new("ellipse1");
    ellipse_common.style = ObjectStyle::default_fill();

    layer.objects.push(VectorObject::Ellipse(EllipseObject {
        common: ellipse_common,
        cx: 200.0,
        cy: 100.0,
        rx: 30.0,
        ry: 20.0,
    }));

    let canvas = Canvas::new(400.0, 300.0);
    VectorDocument {
        canvas,
        layers: vec![layer],
        metadata: common_core::Metadata::default(),
        colour_settings: DocumentColourSettings::default(),
        swatch_library: common_core::colour_management::SwatchLibrary::new(),
    }
}

#[test]
fn test_write_and_parse_roundtrip() {
    let doc = make_doc();
    let mut ctx = make_ctx();
    let svg = write(&doc, &mut ctx).unwrap();
    let parsed = parse(&svg).unwrap();

    assert_eq!(parsed.layers[0].objects.len(), 2);
    if let VectorObject::Rect(r) = &parsed.layers[0].objects[0] {
        assert!((r.x - 10.0).abs() < 0.01);
        assert!((r.width - 100.0).abs() < 0.01);
    } else {
        panic!("expected rect");
    }
    if let VectorObject::Ellipse(e) = &parsed.layers[0].objects[1] {
        assert!((e.cx - 200.0).abs() < 0.01);
        assert!((e.ry - 20.0).abs() < 0.01);
    } else {
        panic!("expected ellipse");
    }
}

#[test]
fn test_write_contains_svg_header() {
    let doc = make_doc();
    let mut ctx = make_ctx();
    let svg = write(&doc, &mut ctx).unwrap();
    assert!(svg.contains(r#"<?xml version="1.0""#));
    assert!(svg.contains(r#"xmlns="http://www.w3.org/2000/svg""#));
}

#[test]
fn test_write_includes_loki_namespace() {
    let doc = make_doc();
    let mut ctx = make_ctx();
    let svg = write(&doc, &mut ctx).unwrap();
    assert!(
        svg.contains("xmlns:loki="),
        "SVG should declare loki namespace"
    );
    assert!(
        svg.contains("loki:colour-settings="),
        "SVG root should have loki:colour-settings attribute"
    );
}

#[test]
fn test_loki_colour_settings_roundtrip() {
    use common_core::colour_management::{
        BuiltInProfile, ColourSpace, DocumentColourSettings, IccProfileRef,
    };
    let cmyk_settings = DocumentColourSettings {
        working_space: ColourSpace::Cmyk {
            profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
        },
        ..DocumentColourSettings::default()
    };
    let doc = VectorDocument {
        canvas: crate::canvas::Canvas::new(400.0, 300.0),
        layers: vec![],
        metadata: common_core::Metadata::default(),
        colour_settings: cmyk_settings.clone(),
        swatch_library: common_core::colour_management::SwatchLibrary::new(),
    };
    let mut ctx = make_ctx();
    let svg = write(&doc, &mut ctx).unwrap();
    let parsed = parse(&svg).unwrap();
    assert_eq!(parsed.colour_settings, cmyk_settings);
}
