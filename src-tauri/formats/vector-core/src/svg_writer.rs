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

/// Serialise a VectorDocument to an SVG string.
pub fn write(doc: &VectorDocument) -> String {
    let canvas = &doc.canvas;
    let unit_suffix = UnitConverter::unit_suffix(canvas.display_unit);
    let w = format_f64(canvas.display_width());
    let h = format_f64(canvas.display_height());
    let vb_w = format_f64(canvas.width);
    let vb_h = format_f64(canvas.height);

    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    out.push('\n');
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" width="{w}{unit_suffix}" height="{h}{unit_suffix}" viewBox="0 0 {vb_w} {vb_h}">"#
    ));
    out.push('\n');

    for layer in &doc.layers {
        let vis = if layer.visible {
            ""
        } else {
            r#" display="none""#
        };
        out.push_str(&format!(
            r#"  <g inkscape:label="{}" inkscape:groupmode="layer" id="{}"{}>"#,
            escape_xml(&layer.name),
            escape_xml(&layer.id),
            vis
        ));
        out.push('\n');
        for obj in &layer.objects {
            write_object(obj, &mut out, 4);
        }
        out.push_str("  </g>\n");
    }

    out.push_str("</svg>\n");
    out
}

fn write_object(obj: &VectorObject, out: &mut String, indent: usize) {
    match obj {
        VectorObject::Rect(r) => write_rect(r, out, indent),
        VectorObject::Ellipse(e) => write_ellipse(e, out, indent),
        VectorObject::Line(l) => write_line(l, out, indent),
        VectorObject::Path(p) => write_path(p, out, indent),
        VectorObject::Group(g) => {
            let pad = " ".repeat(indent);
            let transform_attr = transform_attr_str(&g.common.transform);
            let style_str = build_style(&g.common);
            out.push_str(&format!(
                r#"{pad}<g id="{}"{transform_attr}{style_str}>"#,
                escape_xml(&g.common.id.0)
            ));
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
        r#"{pad}<rect id="{}" x="{}" y="{}" width="{}" height="{}"{rx}{ry}{transform_attr}{style_str}/>"#,
        escape_xml(&r.common.id.0),
        format_f64(r.x), format_f64(r.y),
        format_f64(r.width), format_f64(r.height)
    ));
    out.push('\n');
}

fn write_ellipse(e: &EllipseObject, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    let transform_attr = transform_attr_str(&e.common.transform);
    let style_str = build_style(&e.common);
    out.push_str(&format!(
        r#"{pad}<ellipse id="{}" cx="{}" cy="{}" rx="{}" ry="{}"{transform_attr}{style_str}/>"#,
        escape_xml(&e.common.id.0),
        format_f64(e.cx),
        format_f64(e.cy),
        format_f64(e.rx),
        format_f64(e.ry)
    ));
    out.push('\n');
}

fn write_line(l: &LineObject, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    let transform_attr = transform_attr_str(&l.common.transform);
    let style_str = build_style(&l.common);
    out.push_str(&format!(
        r#"{pad}<line id="{}" x1="{}" y1="{}" x2="{}" y2="{}"{transform_attr}{style_str}/>"#,
        escape_xml(&l.common.id.0),
        format_f64(l.x1),
        format_f64(l.y1),
        format_f64(l.x2),
        format_f64(l.y2)
    ));
    out.push('\n');
}

fn write_path(p: &PathObject, out: &mut String, indent: usize) {
    let pad = " ".repeat(indent);
    let transform_attr = transform_attr_str(&p.common.transform);
    let style_str = build_style(&p.common);
    out.push_str(&format!(
        r#"{pad}<path id="{}" d="{}"{transform_attr}{style_str}/>"#,
        escape_xml(&p.common.id.0),
        escape_xml(&p.d)
    ));
    out.push('\n');
}

fn transform_attr_str(t: &Transform) -> String {
    if t.is_identity() {
        String::new()
    } else {
        format!(r#" transform="{}""#, t.to_svg_matrix())
    }
}

fn build_style(common: &CommonProps) -> String {
    let style = &common.style;
    let mut parts: Vec<String> = Vec::new();

    let fill_str = match &style.fill {
        Paint::None => "none".to_string(),
        Paint::Solid { colour } => colour.to_svg_colour(),
    };
    parts.push(format!("fill:{fill_str}"));

    let stroke_str = match &style.stroke.paint {
        Paint::None => "none".to_string(),
        Paint::Solid { colour } => colour.to_svg_colour(),
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

    format!(r#" style="{}""#, parts.join(";"))
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
mod tests {
    use super::*;
    use crate::canvas::Canvas;
    use crate::document::VectorDocument;
    use crate::layer::Layer;
    use crate::object::{CommonProps, EllipseObject, RectObject};
    use crate::style::ObjectStyle;
    use crate::svg_parser::parse;

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
        }
    }

    #[test]
    fn test_write_and_parse_roundtrip() {
        let doc = make_doc();
        let svg = write(&doc);
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
        let svg = write(&doc);
        assert!(svg.contains(r#"<?xml version="1.0""#));
        assert!(svg.contains(r#"xmlns="http://www.w3.org/2000/svg""#));
    }
}
