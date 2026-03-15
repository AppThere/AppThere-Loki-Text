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

use crate::colour::{parse_css_colour, Colour};
use crate::style::{LineCap, LineJoin, ObjectStyle, Paint, StrokeStyle};

pub(crate) fn parse_style(node: &roxmltree::Node) -> ObjectStyle {
    let mut fill = Paint::Solid { colour: Colour::black() };
    let mut stroke = StrokeStyle::none();

    if let Some(style_str) = node.attribute("style") {
        for part in style_str.split(';') {
            let part = part.trim();
            if let Some((k, v)) = part.split_once(':') {
                apply_style_prop(k.trim(), v.trim(), &mut fill, &mut stroke);
            }
        }
    }

    for attr in node.attributes() {
        apply_style_prop(attr.name(), attr.value(), &mut fill, &mut stroke);
    }

    ObjectStyle { fill, stroke, opacity: 1.0, fill_opacity: 1.0, stroke_opacity: 1.0 }
}

fn apply_style_prop(key: &str, value: &str, fill: &mut Paint, stroke: &mut StrokeStyle) {
    match key {
        "fill" => {
            *fill = if value == "none" {
                Paint::None
            } else if let Some(c) = parse_css_colour(value) {
                Paint::Solid { colour: c }
            } else {
                Paint::None
            };
        }
        "stroke" => {
            stroke.paint = if value == "none" {
                Paint::None
            } else if let Some(c) = parse_css_colour(value) {
                Paint::Solid { colour: c }
            } else {
                Paint::None
            };
        }
        "stroke-width" => {
            if let Ok(w) = value.trim_end_matches("px").parse::<f64>() {
                stroke.width = w;
            }
        }
        "stroke-linecap" => {
            stroke.line_cap = match value {
                "round" => LineCap::Round,
                "square" => LineCap::Square,
                _ => LineCap::Butt,
            };
        }
        "stroke-linejoin" => {
            stroke.line_join = match value {
                "round" => LineJoin::Round,
                "bevel" => LineJoin::Bevel,
                _ => LineJoin::Miter,
            };
        }
        _ => {}
    }
}
