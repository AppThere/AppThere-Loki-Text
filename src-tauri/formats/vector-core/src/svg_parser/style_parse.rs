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

use crate::style::{LineCap, LineJoin, ObjectStyle, Paint, StrokeStyle};
use common_core::colour_management::Colour;

/// Loki SVG extension namespace URI for non-RGB colour preservation.
const LOKI_NS: &str = "http://appthere.com/ns/loki/1.0";

pub(crate) fn parse_style(node: &roxmltree::Node) -> ObjectStyle {
    let mut fill = Paint::Solid {
        colour: Colour::black(),
    };
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

    // Override fill/stroke with Loki non-RGB colour attributes when present.
    // These attributes carry the full lossless colour value (CMYK, Lab, Spot,
    // Linked) that cannot be represented in a standard CSS colour string.
    if let Some(json) = node
        .attribute((LOKI_NS, "fill"))
        .or_else(|| node.attribute("loki:fill"))
    {
        if let Ok(colour) = serde_json::from_str::<Colour>(json) {
            fill = Paint::Solid { colour };
        }
    }

    if let Some(json) = node
        .attribute((LOKI_NS, "stroke"))
        .or_else(|| node.attribute("loki:stroke"))
    {
        if let Ok(colour) = serde_json::from_str::<Colour>(json) {
            stroke.paint = Paint::Solid { colour };
        }
    }

    ObjectStyle {
        fill,
        stroke,
        opacity: 1.0,
        fill_opacity: 1.0,
        stroke_opacity: 1.0,
    }
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

/// Parse a CSS colour string into a `Colour::Rgb` value.
///
/// Handles hex strings, `rgb()`, `rgba()`, and basic CSS named colours.
/// Returns `None` for `"none"`, `"transparent"`, or unrecognised values.
///
/// SVG colours are always parsed as sRGB. CMYK colours are only present in
/// documents loaded from Loki's native format.
fn parse_css_colour(s: &str) -> Option<Colour> {
    let s = s.trim();
    if s == "none" || s == "transparent" {
        return None;
    }
    if s.starts_with('#') {
        return Colour::from_hex(s);
    }
    if s.starts_with("rgb(") || s.starts_with("rgba(") {
        return parse_rgb_function(s);
    }
    // CSS named colours — 16 basic + common extras
    match s.to_lowercase().as_str() {
        "black" => Some(Colour::black()),
        "white" => Some(Colour::white()),
        "red" => Some(Colour::from_u8_rgb(255, 0, 0)),
        "green" => Some(Colour::from_u8_rgb(0, 128, 0)),
        "blue" => Some(Colour::from_u8_rgb(0, 0, 255)),
        "yellow" => Some(Colour::from_u8_rgb(255, 255, 0)),
        "cyan" => Some(Colour::from_u8_rgb(0, 255, 255)),
        "magenta" => Some(Colour::from_u8_rgb(255, 0, 255)),
        "orange" => Some(Colour::from_u8_rgb(255, 165, 0)),
        "purple" => Some(Colour::from_u8_rgb(128, 0, 128)),
        "pink" => Some(Colour::from_u8_rgb(255, 192, 203)),
        "brown" => Some(Colour::from_u8_rgb(165, 42, 42)),
        "gray" | "grey" => Some(Colour::from_u8_rgb(128, 128, 128)),
        "lime" => Some(Colour::from_u8_rgb(0, 255, 0)),
        "navy" => Some(Colour::from_u8_rgb(0, 0, 128)),
        "silver" => Some(Colour::from_u8_rgb(192, 192, 192)),
        "maroon" => Some(Colour::from_u8_rgb(128, 0, 0)),
        "olive" => Some(Colour::from_u8_rgb(128, 128, 0)),
        "teal" => Some(Colour::from_u8_rgb(0, 128, 128)),
        "aqua" => Some(Colour::from_u8_rgb(0, 255, 255)),
        "fuchsia" => Some(Colour::from_u8_rgb(255, 0, 255)),
        _ => None,
    }
}

/// Parse `rgb(r, g, b)` or `rgba(r, g, b, a)` with integer or percentage values.
fn parse_rgb_function(s: &str) -> Option<Colour> {
    let inner = s
        .trim_start_matches("rgba(")
        .trim_start_matches("rgb(")
        .trim_end_matches(')');
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let parse_channel = |p: &str| -> Option<u8> {
        let p = p.trim();
        if p.ends_with('%') {
            let pct: f64 = p.trim_end_matches('%').parse().ok()?;
            Some((pct * 2.55).round().clamp(0.0, 255.0) as u8)
        } else {
            let v: f64 = p.parse().ok()?;
            Some(v.round().clamp(0.0, 255.0) as u8)
        }
    };

    let r = parse_channel(parts[0])?;
    let g = parse_channel(parts[1])?;
    let b = parse_channel(parts[2])?;

    if parts.len() >= 4 {
        let a_str = parts[3].trim();
        let a: f32 = a_str.parse().ok()?;
        let a_u8 = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
        Some(Colour::from_u8_rgba(r, g, b, a_u8))
    } else {
        Some(Colour::from_u8_rgb(r, g, b))
    }
}
