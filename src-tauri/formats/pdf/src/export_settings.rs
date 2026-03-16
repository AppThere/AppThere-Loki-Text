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

//! PDF export settings and PDF/X standard selection.

use serde::{Deserialize, Serialize};

/// The PDF/X conformance standard to target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdfXStandard {
    /// PDF/X-1a:2001 — CMYK only, no transparency, PDF 1.3.
    X1a2001,
    /// PDF/X-4:2008 — RGB/CMYK with ICC, transparency allowed, PDF 1.6+.
    X4_2008,
}

impl PdfXStandard {
    /// The minimum PDF version required by this standard.
    pub fn min_pdf_version(&self) -> (u8, u8) {
        match self {
            PdfXStandard::X1a2001 => (1, 3),
            PdfXStandard::X4_2008 => (1, 6),
        }
    }

    /// The GTS_PDFXVersion string embedded in XMP metadata.
    pub fn gts_version_string(&self) -> &'static str {
        match self {
            PdfXStandard::X1a2001 => "PDF/X-1a:2001",
            PdfXStandard::X4_2008 => "PDF/X-4",
        }
    }

    /// Whether this standard allows transparency (opacity < 1.0, blend modes).
    pub fn allows_transparency(&self) -> bool {
        match self {
            PdfXStandard::X1a2001 => false,
            PdfXStandard::X4_2008 => true,
        }
    }

    /// Whether this standard allows RGB colour spaces.
    pub fn allows_rgb(&self) -> bool {
        match self {
            PdfXStandard::X1a2001 => false,
            PdfXStandard::X4_2008 => true,
        }
    }

    /// Whether this standard requires CMYK-only content.
    pub fn requires_cmyk_only(&self) -> bool {
        match self {
            PdfXStandard::X1a2001 => true,
            PdfXStandard::X4_2008 => false,
        }
    }
}

/// Settings controlling the PDF export.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfExportSettings {
    /// The PDF/X standard to conform to.
    pub standard: PdfXStandard,

    /// Bleed amount in points (1/72 inch) on each side. 0.0 for no bleed.
    pub bleed_pt: f64,

    /// Output condition identifier (e.g. "FOGRA39" for ISO Coated v2).
    /// Required by PDF/X.
    pub output_condition_identifier: String,

    /// Human-readable output condition description.
    pub output_condition: String,

    /// Registry URL for the output condition.
    pub registry_name: String,

    /// Rasterisation DPI for transparency flattening (PDF/X-1a only).
    /// Transparent objects are rasterised at this resolution.
    /// Typical values: 150 (draft), 300 (standard print), 600 (high quality).
    #[serde(default = "default_resolution_dpi")]
    pub resolution_dpi: u32,
}

fn default_resolution_dpi() -> u32 {
    300
}

impl Default for PdfExportSettings {
    fn default() -> Self {
        PdfExportSettings {
            standard: PdfXStandard::X4_2008,
            bleed_pt: 0.0,
            output_condition_identifier: "sRGB".to_string(),
            output_condition: "sRGB IEC61966-2.1".to_string(),
            registry_name: "http://www.color.org".to_string(),
            resolution_dpi: 300,
        }
    }
}
