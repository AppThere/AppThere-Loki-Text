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

//! SVG generation and resvg-based rasterisation for transparency flattening.

use crate::error::PdfError;
use crate::flatten::RasterRegion;
use common_core::colour_management::Colour;
use resvg::usvg;
use vector_core::object::{GroupObject, VectorObject};
use vector_core::style::Paint;

/// A bounding box in document coordinates (pixels).
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Rasterise `objects` into a `RasterRegion` at `resolution_dpi`.
///
/// `doc_dpi` is the source document DPI; `bbox` is the region to render
/// (typically the full canvas for the MVP).
pub(crate) fn rasterise_objects(
    objects: &[VectorObject],
    doc_dpi: f64,
    resolution_dpi: f64,
    bbox: BoundingBox,
) -> Result<RasterRegion, PdfError> {
    let w_px = ((bbox.width * resolution_dpi / doc_dpi).ceil() as u32).max(1);
    let h_px = ((bbox.height * resolution_dpi / doc_dpi).ceil() as u32).max(1);

    let svg = objects_to_svg(objects, bbox, w_px, h_px);

    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg, &options)
        .map_err(|e| PdfError::Internal(format!("SVG parse error during flattening: {}", e)))?;

    let mut pixmap = resvg::tiny_skia::Pixmap::new(w_px, h_px)
        .ok_or_else(|| PdfError::Internal("Failed to allocate rasterisation pixmap".into()))?;

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    Ok(RasterRegion {
        x: bbox.x,
        y: bbox.y,
        width: w_px,
        height: h_px,
        pixels: pixmap.data().to_vec(),
        dpi: resolution_dpi,
    })
}

/// Build a minimal SVG string for the given objects within `bbox`.
///
/// The SVG uses a translate transform so that `bbox.x, bbox.y` maps to
/// the SVG origin. Colours are converted to CSS `rgb()` using a naive
/// CMYK→sRGB approximation — acceptable for rasterisation since the
/// resulting bitmap is embedded with the output intent providing colour
/// mapping for the final PDF rendering.
fn objects_to_svg(objects: &[VectorObject], bbox: BoundingBox, w_px: u32, h_px: u32) -> String {
    let mut buf = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        w_px, h_px, bbox.width, bbox.height
    );
    buf.push('\n');

    // Translate so bbox origin maps to SVG (0,0).
    buf.push_str(&format!(
        "<g transform=\"translate({:.4},{:.4})\">\n",
        -bbox.x, -bbox.y
    ));
    for obj in objects {
        if obj.common().visible {
            write_object(obj, &mut buf);
        }
    }
    buf.push_str("</g>\n</svg>");
    buf
}

fn write_object(obj: &VectorObject, buf: &mut String) {
    let c = obj.common();
    let opacity = c.style.opacity;
    let fill_str = paint_to_css(&c.style.fill);
    let fill_op = c.style.fill_opacity;
    let stroke_str = paint_to_css(&c.style.stroke.paint);
    let stroke_op = c.style.stroke_opacity;
    let sw = c.style.stroke.width;
    let t = &c.transform;

    let transform_attr = if t.is_identity() {
        String::new()
    } else {
        format!(
            " transform=\"matrix({:.6},{:.6},{:.6},{:.6},{:.6},{:.6})\"",
            t.a, t.b, t.c, t.d, t.e, t.f
        )
    };

    match obj {
        VectorObject::Rect(r) => buf.push_str(&format!(
            "<rect x=\"{:.4}\" y=\"{:.4}\" width=\"{:.4}\" height=\"{:.4}\" \
             rx=\"{:.4}\" ry=\"{:.4}\" fill=\"{}\" fill-opacity=\"{:.4}\" \
             stroke=\"{}\" stroke-width=\"{:.4}\" stroke-opacity=\"{:.4}\" \
             opacity=\"{:.4}\"{}/>\n",
            r.x,
            r.y,
            r.width,
            r.height,
            r.rx,
            r.ry,
            fill_str,
            fill_op,
            stroke_str,
            sw,
            stroke_op,
            opacity,
            transform_attr
        )),
        VectorObject::Ellipse(e) => buf.push_str(&format!(
            "<ellipse cx=\"{:.4}\" cy=\"{:.4}\" rx=\"{:.4}\" ry=\"{:.4}\" \
             fill=\"{}\" fill-opacity=\"{:.4}\" stroke=\"{}\" \
             stroke-width=\"{:.4}\" stroke-opacity=\"{:.4}\" \
             opacity=\"{:.4}\"{}/>\n",
            e.cx,
            e.cy,
            e.rx,
            e.ry,
            fill_str,
            fill_op,
            stroke_str,
            sw,
            stroke_op,
            opacity,
            transform_attr
        )),
        VectorObject::Line(l) => buf.push_str(&format!(
            "<line x1=\"{:.4}\" y1=\"{:.4}\" x2=\"{:.4}\" y2=\"{:.4}\" \
             stroke=\"{}\" stroke-width=\"{:.4}\" stroke-opacity=\"{:.4}\" \
             opacity=\"{:.4}\"{}/>\n",
            l.x1, l.y1, l.x2, l.y2, stroke_str, sw, stroke_op, opacity, transform_attr
        )),
        VectorObject::Path(p) => buf.push_str(&format!(
            "<path d=\"{}\" fill=\"{}\" fill-opacity=\"{:.4}\" \
             stroke=\"{}\" stroke-width=\"{:.4}\" stroke-opacity=\"{:.4}\" \
             opacity=\"{:.4}\"{}/>\n",
            p.d, fill_str, fill_op, stroke_str, sw, stroke_op, opacity, transform_attr
        )),
        VectorObject::Group(g) => write_group(g, opacity, buf),
    }
}

fn write_group(g: &GroupObject, opacity: f64, buf: &mut String) {
    buf.push_str(&format!("<g opacity=\"{:.4}\">\n", opacity));
    for child in &g.children {
        if child.common().visible {
            write_object(child, buf);
        }
    }
    buf.push_str("</g>\n");
}

/// Convert a `Paint` to a CSS colour string.
fn paint_to_css(paint: &Paint) -> String {
    match paint {
        Paint::None => "none".into(),
        Paint::Solid { colour } => colour_to_css(colour),
    }
}

/// Naive colour → CSS `rgb()` conversion.
/// Accuracy is acceptable for rasterisation; the bitmap is embedded with
/// the output intent profile for final colour mapping in the PDF viewer.
pub(crate) fn colour_to_css(colour: &Colour) -> String {
    match colour {
        Colour::Rgb { r, g, b, .. } => css_rgb(*r, *g, *b),
        Colour::Cmyk { c, m, y, k, .. } => {
            let r = (1.0 - c) * (1.0 - k);
            let g = (1.0 - m) * (1.0 - k);
            let b = (1.0 - y) * (1.0 - k);
            css_rgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
        }
        Colour::Lab { l, .. } => {
            let v = (l / 100.0).clamp(0.0, 1.0) as f32;
            css_rgb(v, v, v)
        }
        Colour::Spot { cmyk_fallback, .. } => colour_to_css(cmyk_fallback),
        Colour::Linked { .. } => "rgb(0,0,0)".into(),
    }
}

fn css_rgb(r: f32, g: f32, b: f32) -> String {
    format!(
        "rgb({},{},{})",
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8
    )
}
