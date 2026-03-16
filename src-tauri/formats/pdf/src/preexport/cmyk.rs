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

//! Stage 2 of the pre-export pipeline: convert RGB and Lab colours to CMYK.
//!
//! For PDF/X-1a, all colours must be CMYK. This module builds an lcms2
//! colour transform from the document's working CMYK profile (or falls back
//! to naive math when the profile is a stub in development builds).

use crate::error::PdfError;
use common_core::colour_management::{
    BuiltInProfile, Colour, ColourSpace, IccProfileRef, IccProfileStore,
};
use lcms2::{Flags, Intent, PixelFormat, Transform};
use vector_core::document::VectorDocument;
use vector_core::object::VectorObject;
use vector_core::style::Paint;

// ---------------------------------------------------------------------------
// Naive fallback conversions (used when ICC profile is a development stub)
// ---------------------------------------------------------------------------

/// Naive RGB [0,1] → CMYK [0,1] approximation (GCR on K channel).
fn naive_rgb_to_cmyk(r: f32, g: f32, b: f32) -> [f32; 4] {
    let c = 1.0 - r;
    let m = 1.0 - g;
    let y = 1.0 - b;
    let k = c.min(m).min(y);
    if k >= 1.0 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let inv = 1.0 - k;
    [(c - k) / inv, (m - k) / inv, (y - k) / inv, k]
}

/// Naive Lab → CMYK stub fallback: maps L* to luminance, ignores a*/b*.
fn naive_lab_to_cmyk(l: f32, a: f32, b: f32) -> [f32; 4] {
    let v = (l / 100.0).clamp(0.0, 1.0);
    let _ = (a, b);
    naive_rgb_to_cmyk(v, v, v)
}

// ---------------------------------------------------------------------------
// Colour-conversion mode
// ---------------------------------------------------------------------------

/// Selected based on whether a real CMYK ICC profile was successfully loaded.
enum CmykMode {
    /// Full ICC-based transforms via lcms2.
    Icc {
        rgb_xf: Transform<[f32; 3], [f32; 4]>,
        lab_xf: Transform<[f32; 3], [f32; 4]>,
    },
    /// Stub profile detected: fall back to naive mathematical approximation.
    Naive,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Convert all non-CMYK colours in `doc` to CMYK in-place.
///
/// Uses the document's working CMYK profile for the lcms2 transform; falls
/// back to naive math if the profile is a development stub.
pub(crate) fn convert_to_cmyk(doc: &mut VectorDocument) -> Result<(), PdfError> {
    let target = match &doc.colour_settings.working_space {
        ColourSpace::Cmyk { profile } => profile.clone(),
        _ => IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
    };

    let mode = build_cmyk_mode(&target);

    for layer in &mut doc.layers {
        for obj in &mut layer.objects {
            convert_object(obj, &mode)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn build_cmyk_mode(target: &IccProfileRef) -> CmykMode {
    let intent = Intent::RelativeColorimetric;
    let flags = Flags::default();
    let srgb = lcms2::Profile::new_srgb();

    let mut store = IccProfileStore::new();
    let cmyk = match store.get_or_load(target) {
        Some(p) => p,
        None => return CmykMode::Naive,
    };

    let rgb_xf = match Transform::<[f32; 3], [f32; 4]>::new_flags(
        &srgb,
        PixelFormat::RGB_FLT,
        cmyk,
        PixelFormat::CMYK_FLT,
        intent,
        flags,
    ) {
        Ok(xf) => xf,
        Err(_) => return CmykMode::Naive,
    };

    let d50 = lcms2::CIExyY {
        x: 0.3457,
        y: 0.3585,
        Y: 1.0,
    };
    let lab_profile = match lcms2::Profile::new_lab4_context(lcms2::GlobalContext::new(), &d50) {
        Ok(p) => p,
        Err(_) => return CmykMode::Naive,
    };
    let lab_xf = match Transform::<[f32; 3], [f32; 4]>::new_flags(
        &lab_profile,
        PixelFormat::Lab_FLT,
        cmyk,
        PixelFormat::CMYK_FLT,
        intent,
        flags,
    ) {
        Ok(xf) => xf,
        Err(_) => return CmykMode::Naive,
    };

    CmykMode::Icc { rgb_xf, lab_xf }
}

fn convert_object(obj: &mut VectorObject, mode: &CmykMode) -> Result<(), PdfError> {
    {
        let style = &mut obj.common_mut().style;
        if let Paint::Solid { colour } = &mut style.fill {
            *colour = to_cmyk(colour, mode)?;
        }
        if let Paint::Solid { colour } = &mut style.stroke.paint {
            *colour = to_cmyk(colour, mode)?;
        }
    }
    if let VectorObject::Group(g) = obj {
        for child in &mut g.children {
            convert_object(child, mode)?;
        }
    }
    Ok(())
}

fn to_cmyk(colour: &Colour, mode: &CmykMode) -> Result<Colour, PdfError> {
    Ok(match colour {
        Colour::Rgb { r, g, b, a } => {
            let [c, m, y, k] = match mode {
                CmykMode::Icc { rgb_xf, .. } => {
                    let mut out = [0.0f32; 4];
                    rgb_xf.transform_pixels(&[[*r, *g, *b]], std::slice::from_mut(&mut out));
                    out
                }
                CmykMode::Naive => naive_rgb_to_cmyk(*r, *g, *b),
            };
            Colour::Cmyk {
                c,
                m,
                y,
                k,
                alpha: *a,
            }
        }
        Colour::Lab { l, a, b, alpha } => {
            let [c, m, y, k] = match mode {
                CmykMode::Icc { lab_xf, .. } => {
                    let mut out = [0.0f32; 4];
                    lab_xf.transform_pixels(&[[*l, *a, *b]], std::slice::from_mut(&mut out));
                    out
                }
                CmykMode::Naive => naive_lab_to_cmyk(*l, *a, *b),
            };
            Colour::Cmyk {
                c,
                m,
                y,
                k,
                alpha: *alpha,
            }
        }
        Colour::Linked { id } => {
            return Err(PdfError::ColourProfile(format!(
                "Unresolved Linked colour '{}' reached colour-conversion stage",
                id
            )))
        }
        // CMYK and Spot are already correct for X-1a output.
        other => other.clone(),
    })
}

// ---------------------------------------------------------------------------
// Unit tests for naive fallback functions
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naive_rgb_black_gives_k_only() {
        let [c, m, y, k] = naive_rgb_to_cmyk(0.0, 0.0, 0.0);
        assert_eq!([c, m, y, k], [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn naive_rgb_white_gives_zero_ink() {
        let [c, m, y, k] = naive_rgb_to_cmyk(1.0, 1.0, 1.0);
        assert!(k < 0.01, "white should have near-zero K: {}", k);
        assert!(c < 0.01 && m < 0.01 && y < 0.01);
    }

    #[test]
    fn naive_lab_midgray_gives_nonzero_k() {
        // L=50 should give a gray-ish CMYK with some K.
        let [_c, _m, _y, k] = naive_lab_to_cmyk(50.0, 0.0, 0.0);
        assert!(k > 0.0 && k < 1.0, "L=50 should give mid K: {}", k);
    }
}
