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

use crate::document::VectorDocument;
use crate::object::{CommonProps, EllipseObject, LineObject, PathObject, RectObject, VectorObject};
use crate::style::{LineCap, LineJoin, Paint};
use crate::transform::Transform;
use crate::units::UnitConverter;
use common_core::colour_management::{Colour, ColourContext};

/// Loki SVG extension namespace URI.
/// Used to persist non-RGB colour values and document colour settings.
const LOKI_NS: &str = "http://appthere.com/ns/loki/1.0";

/// Serialise a VectorDocument to an SVG string.
///
/// The `ctx` parameter is reserved for colour-managed conversion in Phase 2.
/// In Phase 1, colours are written using `to_svg_colour()` (sRGB fallback),
/// which is correct for sRGB documents and for SVG interoperability.
///
/// Non-RGB colours (CMYK, Lab, Spot, Linked) are written with a sRGB
/// approximation in `style="..."` and the full colour value as a
/// `loki:fill` / `loki:stroke` attribute so they round-trip without loss.
///
/// The document's [`DocumentColourSettings`] are preserved in
/// `loki:colour-settings` on the root `<svg>` element.
pub fn write(doc: &VectorDocument, _ctx: &mut ColourContext) -> Result<String, String> {
    let canvas = &doc.canvas;
    let unit_suffix = UnitConverter::unit_suffix(canvas.display_unit);
    let w = format_f64(canvas.display_width());
    let h = format_f64(canvas.display_height());
    let vb_w = format_f64(canvas.width);
    let vb_h = format_f64(canvas.height);

    let colour_settings_json =
        serde_json::to_string(&doc.colour_settings).unwrap_or_else(|_| "{}".to_string());

    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xmlns:loki="{LOKI_NS}" width="{w}{unit_suffix}" height="{h}{unit_suffix}" viewBox="0 0 {vb_w} {vb_h}" loki:colour-settings="{}">"#,
        escape_xml(&colour_settings_json)
    ));
    out.push('\n');

    for layer in &doc.layers {
        let vis = if layer.visible {
            ""
        } else {
            r#" display="none""#
        };
        out.push_str(&format!(
            r#"  <g inkscape:label="{}" inkscape:groupmode="layer" id="{}"{vis}>"#,
            escape_xml(&layer.name),
            escape_xml(&layer.id),
        ));
        out.push('\n');
        for obj in &layer.objects {
            write_object(obj, &mut out, 4);
        }
        out.push_str("  </g>\n");
    }

    out.push_str("</svg>\n");
    Ok(out)
}

fn write_object(obj: &VectorObject, out: &mut String, indent: usize) {
    match obj {
        VectorObject::Rect(r) => write_rect(r, out, indent),
        VectorObject::Ellipse(e) => write_ellipse(e, out, indent),
        VectorObject::Line(l) => write_line(l, out, indent),
        VectorObject::Path(p) => write_path(p, out, indent),
        VectorObject::Group(g) => {
            let pad = " ".repeat(indent);
            let id = escape_xml(&g.common.id.0);
            let transform_attr = transform_attr_str(&g.common.transform);
            let style_str = build_style(&g.common);
            out.push_str(&format!("{pad}<g id=\"{id}\"{transform_attr}{style_str}>"));
            out.push('\n');
            for child in &g.children {
                write_object(child, out, indent + 2);
            }
            out.push_str(&format!("{pad}</g>\n"));
        }
    }
}

fn write_rect(r: &RectObject, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    let transform_attr = transform_attr_str(&r.common.transform);
    let style_str = build_style(&r.common);
    let rx = if r.rx != 0.0 {
        format!(r#" rx="{}""#, format_f64(r.rx))
    } else {
        String::new()
    };
    let ry = if r.ry != 0.0 {
        format!(r#" ry="{}""#, format_f64(r.ry))
    } else {
        String::new()
    };
    out.push_str(&format!(
        r#"{pad}<rect id="{}" x="{}" y="{}" width="{}" height="{}"{}{}{}{}/>
"#,
        escape_xml(&r.common.id.0),
        format_f64(r.x),
        format_f64(r.y),
        format_f64(r.width),
        format_f64(r.height),
        rx,
        ry,
        transform_attr,
        style_str
    ));
}

fn write_ellipse(e: &EllipseObject, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    let transform_attr = transform_attr_str(&e.common.transform);
    let style_str = build_style(&e.common);
    out.push_str(&format!(
        r#"{pad}<ellipse id="{}" cx="{}" cy="{}" rx="{}" ry="{}"{}{}/>
"#,
        escape_xml(&e.common.id.0),
        format_f64(e.cx),
        format_f64(e.cy),
        format_f64(e.rx),
        format_f64(e.ry),
        transform_attr,
        style_str
    ));
}

fn write_line(l: &LineObject, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    let transform_attr = transform_attr_str(&l.common.transform);
    let style_str = build_style(&l.common);
    out.push_str(&format!(
        r#"{pad}<line id="{}" x1="{}" y1="{}" x2="{}" y2="{}"{}{}/>
"#,
        escape_xml(&l.common.id.0),
        format_f64(l.x1),
        format_f64(l.y1),
        format_f64(l.x2),
        format_f64(l.y2),
        transform_attr,
        style_str
    ));
}

fn write_path(p: &PathObject, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    let transform_attr = transform_attr_str(&p.common.transform);
    let style_str = build_style(&p.common);
    out.push_str(&format!(
        r#"{pad}<path id="{}" d="{}"{}{}/>
"#,
        escape_xml(&p.common.id.0),
        escape_xml(&p.d),
        transform_attr,
        style_str
    ));
}

fn transform_attr_str(t: &Transform) -> String {
    if t.is_identity() {
        String::new()
    } else {
        format!(r#" transform="{}""#, t.to_svg_matrix())
    }
}

/// Build the `style="..."` attribute string and any `loki:*` colour attributes.
///
/// For non-RGB colours the sRGB approximation (from `to_svg_colour()`) is
/// written into the standard `style` attribute so every SVG renderer can
/// display a reasonable colour. The full lossless colour value is written as
/// `loki:fill` or `loki:stroke` for round-trip fidelity in Loki.
fn build_style(common: &CommonProps) -> String {
    let style = &common.style;
    let mut parts: Vec<String> = Vec::new();
    let mut loki_attrs = String::new();

    // Fill
    let fill_str = match &style.fill {
        Paint::None => "none".to_string(),
        Paint::Solid { colour } => {
            if !matches!(colour, Colour::Rgb { .. }) {
                if let Ok(json) = serde_json::to_string(colour) {
                    loki_attrs.push_str(&format!(r#" loki:fill="{}""#, escape_xml(&json)));
                }
            }
            colour.to_svg_colour()
        }
    };
    parts.push(format!("fill:{fill_str}"));

    // Stroke
    let stroke_str = match &style.stroke.paint {
        Paint::None => "none".to_string(),
        Paint::Solid { colour } => {
            if !matches!(colour, Colour::Rgb { .. }) {
                if let Ok(json) = serde_json::to_string(colour) {
                    loki_attrs.push_str(&format!(r#" loki:stroke="{}""#, escape_xml(&json)));
                }
            }
            colour.to_svg_colour()
        }
    };
    parts.push(format!("stroke:{stroke_str}"));

    if matches!(style.stroke.paint, Paint::Solid { .. }) {
        parts.push(format!("stroke-width:{}", format_f64(style.stroke.width)));
        let cap = match style.stroke.line_cap {
            LineCap::Round => "round",
            LineCap::Square => "square",
            LineCap::Butt => "butt",
        };
        parts.push(format!("stroke-linecap:{cap}"));
        let join = match style.stroke.line_join {
            LineJoin::Round => "round",
            LineJoin::Bevel => "bevel",
            LineJoin::Miter => "miter",
        };
        parts.push(format!("stroke-linejoin:{join}"));
    }

    if style.opacity != 1.0 {
        parts.push(format!("opacity:{}", format_f64(style.opacity)));
    }

    format!(r#"{loki_attrs} style="{}""#, parts.join(";"))
}

fn format_f64(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e12 {
        format!("{}", v as i64)
    } else {
        format!("{:.6}", v)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "svg_writer_tests.rs"]
mod tests;
