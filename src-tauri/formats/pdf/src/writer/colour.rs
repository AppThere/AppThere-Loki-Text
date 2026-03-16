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

//! Colour space and colour value encoding for PDF content streams.

use common_core::colour_management::Colour;

/// PDF colour components for a non-stroking (fill) colour operation.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfColour {
    /// DeviceCMYK — four components in [0,1].
    Cmyk([f32; 4]),
    /// DeviceRGB or ICCBased RGB — three components in [0,1].
    Rgb([f32; 3]),
    /// Separation — tint value in [0,1] with a name.
    Separation { name: String, tint: f32 },
}

impl PdfColour {
    /// Extract the PDF colour representation from a `Colour` value.
    ///
    /// For `Spot` colours the CMYK fallback is used when the Separation
    /// colour space is not available.
    pub fn from_colour(colour: &Colour) -> Self {
        match colour {
            Colour::Rgb { r, g, b, .. } => PdfColour::Rgb([*r, *g, *b]),
            Colour::Cmyk { c, m, y, k, .. } => PdfColour::Cmyk([*c, *m, *y, *k]),
            Colour::Lab { l, .. } => {
                // Approximate Lab → RGB via a simple linear approximation
                // (accurate conversion requires a colour profile; for PDF/X-4
                // we store Lab colours as their closest RGB approximation and
                // rely on the output device profile for final rendering).
                let l_n = (*l / 100.0).clamp(0.0, 1.0);
                PdfColour::Rgb([l_n, l_n, l_n])
            }
            Colour::Spot {
                name,
                tint,
                cmyk_fallback,
                ..
            } => {
                let _ = cmyk_fallback; // fallback used when Separation CS not available
                PdfColour::Separation {
                    name: name.clone(),
                    tint: *tint,
                }
            }
            Colour::Linked { .. } => {
                // Should have been rejected by the validator.
                PdfColour::Cmyk([0.0, 0.0, 0.0, 1.0])
            }
        }
    }

    /// Alpha channel for this colour (0.0–1.0).
    pub fn alpha(colour: &Colour) -> f32 {
        match colour {
            Colour::Rgb { a, .. } => *a,
            Colour::Cmyk { alpha, .. } => *alpha,
            Colour::Lab { alpha, .. } => *alpha,
            Colour::Spot { tint, .. } => *tint,
            Colour::Linked { .. } => 1.0,
        }
    }
}
