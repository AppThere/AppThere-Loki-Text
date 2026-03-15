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

//! Page geometry and PDF page box calculations.

use crate::export_settings::PdfExportSettings;

/// Page geometry in PDF points (1/72 inch).
#[derive(Debug, Clone, Copy)]
pub struct PageGeometry {
    /// MediaBox — full physical page including bleed.
    pub media_box: PdfRect,
    /// TrimBox — final trimmed page size.
    pub trim_box: PdfRect,
    /// BleedBox — trim box extended by bleed on all sides.
    pub bleed_box: PdfRect,
}

/// A rectangle in PDF coordinates (lower-left origin, Y-up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfRect {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl PdfRect {
    pub fn new(x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        PdfRect {
            x_min,
            y_min,
            x_max,
            y_max,
        }
    }

    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }
}

/// Convert canvas pixels to PDF points using the document DPI.
///
/// `px / dpi * 72` gives points.
pub fn px_to_pt(px: f64, dpi: f64) -> f64 {
    px / dpi * 72.0
}

/// Compute page geometry for the given canvas dimensions and export settings.
///
/// `canvas_width_px` and `canvas_height_px` are in pixels; `dpi` is the
/// canvas DPI. The bleed from `settings.bleed_pt` is in points.
pub fn compute_page_geometry(
    canvas_width_px: f64,
    canvas_height_px: f64,
    dpi: f64,
    settings: &PdfExportSettings,
) -> PageGeometry {
    let w_pt = px_to_pt(canvas_width_px, dpi);
    let h_pt = px_to_pt(canvas_height_px, dpi);
    let bleed = settings.bleed_pt;

    let trim_box = PdfRect::new(0.0, 0.0, w_pt, h_pt);
    let bleed_box = PdfRect::new(-bleed, -bleed, w_pt + bleed, h_pt + bleed);
    // MediaBox is at least as large as bleed box.
    let media_box = bleed_box;

    PageGeometry {
        media_box,
        trim_box,
        bleed_box,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_at_96dpi_converts_to_points() {
        // A4 at 96 DPI: 794 × 1123 px → ~595 × ~842 pt.
        let w_px = 794.0_f64;
        let h_px = 1123.0_f64;
        let w_pt = px_to_pt(w_px, 96.0);
        let h_pt = px_to_pt(h_px, 96.0);
        assert!((w_pt - 595.0).abs() < 1.0, "width ~595pt, got {}", w_pt);
        assert!((h_pt - 842.0).abs() < 1.0, "height ~842pt, got {}", h_pt);
    }

    #[test]
    fn bleed_expands_boxes_correctly() {
        let settings = PdfExportSettings {
            bleed_pt: 8.503937, // 3mm ≈ 8.5pt
            ..Default::default()
        };
        let geo = compute_page_geometry(595.0, 842.0, 72.0, &settings);
        assert!((geo.trim_box.width() - 595.0).abs() < 0.01);
        assert!((geo.bleed_box.x_min + 8.503937).abs() < 0.01);
        assert!((geo.bleed_box.x_max - (595.0 + 8.503937)).abs() < 0.01);
    }

    #[test]
    fn no_bleed_boxes_are_equal() {
        let settings = PdfExportSettings {
            bleed_pt: 0.0,
            ..Default::default()
        };
        let geo = compute_page_geometry(595.0, 842.0, 72.0, &settings);
        assert_eq!(geo.trim_box, geo.bleed_box);
        assert_eq!(geo.trim_box, geo.media_box);
    }
}
