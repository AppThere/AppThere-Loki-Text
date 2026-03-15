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

//! The `Colour` type and constructors/accessors.
//!
//! For colour-managed conversion to display sRGB, see `transform.rs` which
//! adds `to_display_rgb` via a separate `impl Colour` block.

use serde::{Deserialize, Serialize};

/// A colour value in one of several colour spaces.
///
/// All `f32` channel values are normalised to `0.0–1.0` unless otherwise
/// documented. Alpha is always `0.0` (fully transparent) to `1.0` (fully
/// opaque).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Colour {
    /// sRGB or linear RGB colour.
    Rgb { r: f32, g: f32, b: f32, a: f32 },
    /// CMYK ink percentages. Each channel: 0.0 (no ink) to 1.0 (full ink).
    Cmyk {
        c: f32,
        m: f32,
        y: f32,
        k: f32,
        alpha: f32,
    },
    /// CIE L*a*b* colour. L: 0.0–100.0, a and b: −128.0–127.0.
    Lab { l: f32, a: f32, b: f32, alpha: f32 },
    /// A spot (named) ink colour with a Lab reference and a CMYK fallback.
    Spot {
        /// The spot ink name, e.g. "PANTONE 186 C".
        name: String,
        /// Tint: 0.0 = no ink, 1.0 = full density.
        tint: f32,
        /// CIE L*a*b* reference values from the colour standard.
        lab_ref: [f32; 3],
        /// CMYK process equivalent for devices without spot ink support.
        cmyk_fallback: Box<Colour>,
    },
    /// A reference into the document's swatch library by swatch ID.
    Linked { id: String },
}

impl Colour {
    /// Construct from 8-bit RGB values (0–255). Alpha defaults to fully opaque.
    pub fn from_u8_rgb(r: u8, g: u8, b: u8) -> Self {
        Colour::Rgb {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    /// Construct from 8-bit RGBA values (0–255).
    pub fn from_u8_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Colour::Rgb {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    /// Parse a CSS hex colour string.
    ///
    /// Accepts "#rgb", "#rrggbb", "#rrggbbaa", with or without the "#".
    /// Returns None for invalid input.
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        match s.len() {
            3 => {
                let expanded: String = s.chars().flat_map(|c| [c, c]).collect();
                Self::parse_hex_6(&expanded, 255)
            }
            6 => Self::parse_hex_6(s, 255),
            8 => {
                let aa = u8::from_str_radix(&s[6..8], 16).ok()?;
                Self::parse_hex_6(&s[0..6], aa)
            }
            _ => None,
        }
    }

    fn parse_hex_6(s: &str, a: u8) -> Option<Self> {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Colour::from_u8_rgba(r, g, b, a))
    }

    /// Construct a fully opaque black.
    pub fn black() -> Self {
        Colour::Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    /// Construct a fully opaque white.
    pub fn white() -> Self {
        Colour::Rgb {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    /// Construct a fully transparent colour (black with alpha 0).
    pub fn transparent() -> Self {
        Colour::Rgb {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }

    /// Returns the alpha channel value (0.0–1.0).
    /// For Spot colours, returns the tint value.
    /// For Linked colours, returns 1.0 (resolved via the swatch library).
    pub fn alpha(&self) -> f32 {
        match self {
            Colour::Rgb { a, .. } => *a,
            Colour::Cmyk { alpha, .. } => *alpha,
            Colour::Lab { alpha, .. } => *alpha,
            Colour::Spot { tint, .. } => *tint,
            Colour::Linked { .. } => 1.0,
        }
    }

    /// Returns a copy of this colour with the alpha channel replaced.
    /// For Spot colours, replaces the tint value.
    /// For Linked colours, returns self unchanged.
    pub fn with_alpha(self, alpha: f32) -> Self {
        match self {
            Colour::Rgb { r, g, b, .. } => Colour::Rgb { r, g, b, a: alpha },
            Colour::Cmyk { c, m, y, k, .. } => Colour::Cmyk { c, m, y, k, alpha },
            Colour::Lab { l, a, b, .. } => Colour::Lab { l, a, b, alpha },
            Colour::Spot {
                name,
                lab_ref,
                cmyk_fallback,
                ..
            } => Colour::Spot {
                name,
                tint: alpha,
                lab_ref,
                cmyk_fallback,
            },
            Colour::Linked { .. } => self,
        }
    }

    /// Returns true if this colour is fully transparent.
    pub fn is_transparent(&self) -> bool {
        self.alpha() == 0.0
    }

    /// Returns a CSS colour string for SVG/HTML output (always sRGB).
    ///
    /// - Rgb → "#rrggbb" (opaque) or "rgba(r, g, b, a)"
    /// - Cmyk → naive device-dependent conversion. Use `to_display_rgb` for accuracy.
    /// - Lab → naive approximation. Use `to_display_rgb` for accuracy.
    /// - Spot → converts cmyk_fallback.
    /// - Linked → "#000000" (cannot resolve without swatch library).
    pub fn to_css_string(&self) -> String {
        match self {
            Colour::Rgb { r, g, b, a } => {
                let ri = (*r * 255.0).round() as u8;
                let gi = (*g * 255.0).round() as u8;
                let bi = (*b * 255.0).round() as u8;
                if *a >= 1.0 {
                    format!("#{:02x}{:02x}{:02x}", ri, gi, bi)
                } else {
                    let ai = (*a * 255.0).round() as u8;
                    format!("rgba({}, {}, {}, {})", ri, gi, bi, ai)
                }
            }
            Colour::Cmyk { c, m, y, k, alpha } => {
                // DEVICE-DEPENDENT. Inaccurate for colour-managed workflows.
                let r = (1.0 - c) * (1.0 - k);
                let g = (1.0 - m) * (1.0 - k);
                let b = (1.0 - y) * (1.0 - k);
                Colour::Rgb { r, g, b, a: *alpha }.to_css_string()
            }
            Colour::Lab {
                l,
                a: la,
                b: lb,
                alpha,
            } => {
                // APPROXIMATION only. Use ColourContext::convert() for accuracy.
                let r = (*l / 100.0).clamp(0.0, 1.0);
                let g = ((*la + 128.0) / 255.0).clamp(0.0, 1.0);
                let b = ((*lb + 128.0) / 255.0).clamp(0.0, 1.0);
                Colour::Rgb { r, g, b, a: *alpha }.to_css_string()
            }
            Colour::Spot { cmyk_fallback, .. } => cmyk_fallback.to_css_string(),
            Colour::Linked { .. } => "#000000".to_string(),
        }
    }

    /// Alias for `to_css_string`. Convenience for SVG attribute generation.
    pub fn to_svg_colour(&self) -> String {
        self.to_css_string()
    }

    /// Returns "#rrggbb" or "#rrggbbaa". Returns None for non-RGB variants.
    pub fn to_hex(&self) -> Option<String> {
        match self {
            Colour::Rgb { r, g, b, a } => {
                let ri = (*r * 255.0).round() as u8;
                let gi = (*g * 255.0).round() as u8;
                let bi = (*b * 255.0).round() as u8;
                if *a >= 1.0 {
                    Some(format!("#{:02x}{:02x}{:02x}", ri, gi, bi))
                } else {
                    let ai = (*a * 255.0).round() as u8;
                    Some(format!("#{:02x}{:02x}{:02x}{:02x}", ri, gi, bi, ai))
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "colour_tests.rs"]
mod tests;
