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

//! PDF content stream builder — converts vector objects to PDF operators.

use crate::error::PdfError;
use crate::writer::colour::PdfColour;
use crate::writer::path_ops::{ops_to_pdf_stream, parse_svg_path, PathOp};
use common_core::colour_management::Colour;
use vector_core::object::{
    EllipseObject, GroupObject, LineObject, PathObject, RectObject, VectorObject,
};
use vector_core::style::{LineCap, LineJoin, ObjectStyle, Paint};
use vector_core::transform::Transform;

/// Build a PDF content stream for a list of vector objects.
///
/// Returns the content stream as a UTF-8 string of PDF operators.
pub fn build_content_stream(
    objects: &[VectorObject],
    page_height_pt: f64,
) -> Result<String, PdfError> {
    let mut buf = String::new();
    for obj in objects {
        write_object(obj, page_height_pt, &mut buf)?;
    }
    Ok(buf)
}

fn write_object(obj: &VectorObject, page_height_pt: f64, buf: &mut String) -> Result<(), PdfError> {
    let common = obj.common();
    if !common.visible {
        return Ok(());
    }

    // Apply transform.
    let has_transform = !common.transform.is_identity();
    if has_transform {
        buf.push_str("q\n");
        write_cm(&common.transform, buf);
    }

    match obj {
        VectorObject::Rect(r) => write_rect(r, page_height_pt, buf)?,
        VectorObject::Ellipse(e) => write_ellipse(e, page_height_pt, buf)?,
        VectorObject::Line(l) => write_line(l, page_height_pt, buf)?,
        VectorObject::Path(p) => write_path(p, page_height_pt, buf)?,
        VectorObject::Group(g) => write_group(g, page_height_pt, buf)?,
    }

    if has_transform {
        buf.push_str("Q\n");
    }

    Ok(())
}

fn write_cm(t: &Transform, buf: &mut String) {
    buf.push_str(&format!(
        "{:.6} {:.6} {:.6} {:.6} {:.6} {:.6} cm\n",
        t.a, t.b, t.c, t.d, t.e, t.f
    ));
}

fn write_rect(r: &RectObject, page_height_pt: f64, buf: &mut String) -> Result<(), PdfError> {
    let pdf_y = page_height_pt - r.y - r.height;
    let stream = format!(
        "{:.4} {:.4} {:.4} {:.4} re\n",
        r.x, pdf_y, r.width, r.height
    );
    buf.push_str(&stream);
    apply_paint_ops(&r.common.style, buf);
    Ok(())
}

fn write_ellipse(e: &EllipseObject, page_height_pt: f64, buf: &mut String) -> Result<(), PdfError> {
    // Approximate ellipse with 4 Bezier curves.
    let k = 0.5522847498; // magic constant for circular arcs
    let cx = e.cx;
    let cy = page_height_pt - e.cy; // PDF Y-up
    let rx = e.rx;
    let ry = e.ry;

    let ops = vec![
        PathOp::MoveTo(cx + rx, cy),
        PathOp::CurveTo(cx + rx, cy + ry * k, cx + rx * k, cy + ry, cx, cy + ry),
        PathOp::CurveTo(cx - rx * k, cy + ry, cx - rx, cy + ry * k, cx - rx, cy),
        PathOp::CurveTo(cx - rx, cy - ry * k, cx - rx * k, cy - ry, cx, cy - ry),
        PathOp::CurveTo(cx + rx * k, cy - ry, cx + rx, cy - ry * k, cx + rx, cy),
        PathOp::ClosePath,
    ];
    buf.push_str(&ops_to_pdf_stream(&ops));
    apply_paint_ops(&e.common.style, buf);
    Ok(())
}

fn write_line(l: &LineObject, page_height_pt: f64, buf: &mut String) -> Result<(), PdfError> {
    let ops = vec![
        PathOp::MoveTo(l.x1, page_height_pt - l.y1),
        PathOp::LineTo(l.x2, page_height_pt - l.y2),
    ];
    buf.push_str(&ops_to_pdf_stream(&ops));
    // Lines are always stroked, never filled.
    apply_stroke_only(&l.common.style, buf);
    Ok(())
}

fn write_path(p: &PathObject, page_height_pt: f64, buf: &mut String) -> Result<(), PdfError> {
    let ops = parse_svg_path(&p.d, page_height_pt);
    buf.push_str(&ops_to_pdf_stream(&ops));
    apply_paint_ops(&p.common.style, buf);
    Ok(())
}

fn write_group(g: &GroupObject, page_height_pt: f64, buf: &mut String) -> Result<(), PdfError> {
    buf.push_str("q\n");
    for child in &g.children {
        write_object(child, page_height_pt, buf)?;
    }
    buf.push_str("Q\n");
    Ok(())
}

/// Emit colour-setting and fill/stroke operators for a style.
fn apply_paint_ops(style: &ObjectStyle, buf: &mut String) {
    let has_fill = matches!(&style.fill, Paint::Solid { .. });
    let has_stroke = matches!(&style.stroke.paint, Paint::Solid { .. });

    // Set fill colour.
    if let Paint::Solid { colour } = &style.fill {
        write_fill_colour(colour, buf);
    }

    // Set stroke colour and attributes.
    if let Paint::Solid { colour } = &style.stroke.paint {
        write_stroke_colour(colour, buf);
        buf.push_str(&format!("{:.4} w\n", style.stroke.width));
        write_line_cap(&style.stroke.line_cap, buf);
        write_line_join(&style.stroke.line_join, buf);
    }

    // Paint operator.
    match (has_fill, has_stroke) {
        (true, true) => buf.push_str("B\n"),   // fill and stroke
        (true, false) => buf.push_str("f\n"),  // fill only
        (false, true) => buf.push_str("S\n"),  // stroke only
        (false, false) => buf.push_str("n\n"), // no-op (consume path)
    }
}

fn apply_stroke_only(style: &ObjectStyle, buf: &mut String) {
    if let Paint::Solid { colour } = &style.stroke.paint {
        write_stroke_colour(colour, buf);
        buf.push_str(&format!("{:.4} w\n", style.stroke.width));
    }
    buf.push_str("S\n");
}

fn write_fill_colour(colour: &Colour, buf: &mut String) {
    match PdfColour::from_colour(colour) {
        PdfColour::Cmyk([c, m, y, k]) => {
            buf.push_str(&format!("{:.4} {:.4} {:.4} {:.4} k\n", c, m, y, k));
        }
        PdfColour::Rgb([r, g, b]) => {
            buf.push_str(&format!("{:.4} {:.4} {:.4} rg\n", r, g, b));
        }
        PdfColour::Separation { tint, .. } => {
            buf.push_str(&format!("{:.4} sc\n", tint));
        }
    }
}

fn write_stroke_colour(colour: &Colour, buf: &mut String) {
    match PdfColour::from_colour(colour) {
        PdfColour::Cmyk([c, m, y, k]) => {
            buf.push_str(&format!("{:.4} {:.4} {:.4} {:.4} K\n", c, m, y, k));
        }
        PdfColour::Rgb([r, g, b]) => {
            buf.push_str(&format!("{:.4} {:.4} {:.4} RG\n", r, g, b));
        }
        PdfColour::Separation { tint, .. } => {
            buf.push_str(&format!("{:.4} SC\n", tint));
        }
    }
}

fn write_line_cap(cap: &LineCap, buf: &mut String) {
    let code = match cap {
        LineCap::Butt => 0,
        LineCap::Round => 1,
        LineCap::Square => 2,
    };
    buf.push_str(&format!("{} J\n", code));
}

fn write_line_join(join: &LineJoin, buf: &mut String) {
    let code = match join {
        LineJoin::Miter => 0,
        LineJoin::Round => 1,
        LineJoin::Bevel => 2,
    };
    buf.push_str(&format!("{} j\n", code));
}
