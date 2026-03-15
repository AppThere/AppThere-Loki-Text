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

//! Colour space conversion using lcms2.

use super::colour::Colour;
use super::profile::IccProfileStore;
use super::space::{ColourSpace, DocumentColourSettings, RenderingIntent};
use lcms2::{Flags, Intent, PixelFormat, Transform};
use std::collections::HashMap;

/// Holds either a 3-channel (RGB/Lab) or 4-channel (CMYK) lcms2 transform
/// to sRGB output, or a naive stub fallback.
pub(crate) enum WorkingTransform {
    /// Input is 3-channel float (RGB or Lab).
    ThreeChannel(Transform<[f32; 3], [f32; 3]>),
    /// Input is 4-channel float (CMYK).
    FourChannel(Transform<[f32; 4], [f32; 3]>),
    /// Naive CMYK→RGB fallback when the CMYK profile is a stub.
    NaiveCmyk,
    /// Direct sRGB identity: Rgb values pass through unchanged.
    SrgbIdentity,
}

/// A live colour transform pipeline for converting document colours to
/// display sRGB. Created once per document and reused across render passes.
///
/// The context is not Send + Sync due to lcms2 internals — it must be
/// used on a single thread. In Tauri, this means it lives in a
/// per-request scope or is re-created per command invocation.
pub struct ColourContext {
    pub(crate) working: WorkingTransform,
    /// Lab→sRGB transform for Colour::Lab inputs.
    lab_transform: Transform<[f32; 3], [f32; 3]>,
    /// Cache: bit-cast input colour → sRGB [f32; 4] result.
    cache: HashMap<[u32; 4], [f32; 4]>,
}

fn to_lcms_intent(intent: RenderingIntent) -> Intent {
    match intent {
        RenderingIntent::Perceptual => Intent::Perceptual,
        RenderingIntent::RelativeColorimetric => Intent::RelativeColorimetric,
        RenderingIntent::Saturation => Intent::Saturation,
        RenderingIntent::AbsoluteColorimetric => Intent::AbsoluteColorimetric,
    }
}

impl ColourContext {
    /// Create a context for converting from the document's working space
    /// to display sRGB.
    pub fn new_for_display(
        settings: &DocumentColourSettings,
        store: &mut IccProfileStore,
    ) -> Result<Self, String> {
        let intent = to_lcms_intent(settings.rendering_intent);
        let flags = if settings.blackpoint_compensation {
            Flags::BLACKPOINT_COMPENSATION
        } else {
            Flags::default()
        };
        let srgb_out = lcms2::Profile::new_srgb();
        let lab_transform = {
            let d50 = lcms2::CIExyY {
                x: 0.3457,
                y: 0.3585,
                Y: 1.0,
            };
            let lab_in = lcms2::Profile::new_lab4_context(lcms2::GlobalContext::new(), &d50)
                .map_err(|_| "Failed to create Lab profile".to_string())?;
            Transform::new_flags(
                &lab_in,
                PixelFormat::Lab_FLT,
                &srgb_out,
                PixelFormat::RGB_FLT,
                intent,
                flags,
            )
            .map_err(|_| "Failed to create Lab→sRGB transform".to_string())?
        };
        let working = build_working_transform(settings, store, intent, flags)?;
        Ok(Self {
            working,
            lab_transform,
            cache: HashMap::new(),
        })
    }

    /// Convert a single Colour to display sRGB [r, g, b, a] (0.0–1.0).
    /// Results are cached after the first call.
    pub fn convert(&mut self, colour: &Colour) -> [f32; 4] {
        match colour {
            Colour::Rgb { r, g, b, a } => {
                if let WorkingTransform::SrgbIdentity = &self.working {
                    return [*r, *g, *b, *a];
                }
                let key = [r.to_bits(), g.to_bits(), b.to_bits(), a.to_bits()];
                if let Some(&cached) = self.cache.get(&key) {
                    return cached;
                }
                let mut out = [0.0f32; 3];
                if let WorkingTransform::ThreeChannel(t) = &self.working {
                    t.transform_pixels(&[[*r, *g, *b]], std::slice::from_mut(&mut out));
                }
                let result = [out[0], out[1], out[2], *a];
                self.cache.insert(key, result);
                result
            }
            Colour::Cmyk { c, m, y, k, alpha } => {
                let key = [c.to_bits(), m.to_bits(), y.to_bits(), k.to_bits()];
                if let Some(&cached) = self.cache.get(&key) {
                    return [cached[0], cached[1], cached[2], *alpha];
                }
                let rgb = match &self.working {
                    WorkingTransform::FourChannel(t) => {
                        let mut out = [0.0f32; 3];
                        t.transform_pixels(&[[*c, *m, *y, *k]], std::slice::from_mut(&mut out));
                        [out[0], out[1], out[2]]
                    }
                    _ => naive_cmyk(*c, *m, *y, *k),
                };
                self.cache.insert(key, [rgb[0], rgb[1], rgb[2], 0.0]);
                [rgb[0], rgb[1], rgb[2], *alpha]
            }
            Colour::Lab { l, a, b, alpha } => {
                let key = [l.to_bits(), a.to_bits(), b.to_bits(), alpha.to_bits()];
                if let Some(&cached) = self.cache.get(&key) {
                    return cached;
                }
                let mut out = [0.0f32; 3];
                self.lab_transform
                    .transform_pixels(&[[*l, *a, *b]], std::slice::from_mut(&mut out));
                let result = [out[0], out[1], out[2], *alpha];
                self.cache.insert(key, result);
                result
            }
            Colour::Spot {
                cmyk_fallback,
                tint,
                ..
            } => {
                let rgb = self.convert(cmyk_fallback);
                let r = 1.0 - tint * (1.0 - rgb[0]);
                let g = 1.0 - tint * (1.0 - rgb[1]);
                let b = 1.0 - tint * (1.0 - rgb[2]);
                [r, g, b, rgb[3]]
            }
            Colour::Linked { .. } => {
                eprintln!("WARNING: Cannot resolve Colour::Linked without swatch library.");
                [0.0, 0.0, 0.0, 1.0]
            }
        }
    }

    /// Convert a slice of colours to display sRGB in a single batch.
    pub fn convert_batch(&mut self, colours: &[Colour]) -> Vec<[f32; 4]> {
        colours.iter().map(|c| self.convert(c)).collect()
    }

    /// Clear the conversion cache. Call when document colour settings change.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Returns the number of entries currently in the conversion cache.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

pub(crate) fn naive_cmyk(c: f32, m: f32, y: f32, k: f32) -> [f32; 3] {
    [
        ((1.0 - c) * (1.0 - k)).clamp(0.0, 1.0),
        ((1.0 - m) * (1.0 - k)).clamp(0.0, 1.0),
        ((1.0 - y) * (1.0 - k)).clamp(0.0, 1.0),
    ]
}

fn build_working_transform(
    settings: &DocumentColourSettings,
    store: &mut IccProfileStore,
    intent: Intent,
    flags: Flags,
) -> Result<WorkingTransform, String> {
    let srgb_out = lcms2::Profile::new_srgb();
    match &settings.working_space {
        ColourSpace::Srgb => Ok(WorkingTransform::SrgbIdentity),
        ColourSpace::DisplayP3 | ColourSpace::AdobeRgb => Err(format!(
            "{:?} requires a user-supplied ICC profile.",
            settings.working_space
        )),
        ColourSpace::Cmyk { profile } => {
            if store.is_stub_ref(profile) {
                eprintln!("WARNING: CMYK profile is a stub. Using naive CMYK→sRGB fallback.");
                return Ok(WorkingTransform::NaiveCmyk);
            }
            let input = store
                .get_or_load(profile)
                .ok_or_else(|| format!("Failed to load CMYK profile: {:?}", profile))?;
            let t = Transform::new_flags(
                input,
                PixelFormat::CMYK_FLT,
                &srgb_out,
                PixelFormat::RGB_FLT,
                intent,
                flags,
            )
            .map_err(|_| "Failed to create CMYK→sRGB transform".to_string())?;
            Ok(WorkingTransform::FourChannel(t))
        }
        ColourSpace::Custom { profile } => {
            let input = store
                .get_or_load(profile)
                .ok_or_else(|| format!("Failed to load custom profile: {:?}", profile))?;
            let t = Transform::new_flags(
                input,
                PixelFormat::RGB_FLT,
                &srgb_out,
                PixelFormat::RGB_FLT,
                intent,
                flags,
            )
            .map_err(|_| "Failed to create custom profile transform".to_string())?;
            Ok(WorkingTransform::ThreeChannel(t))
        }
    }
}

/// Adds `to_display_rgb` to `Colour` when colour-management is active.
impl Colour {
    /// Convert this colour to display sRGB using the provided colour context.
    ///
    /// This is the colour-managed path. For a naive fallback, use `to_css_string()`.
    pub fn to_display_rgb(&self, ctx: &mut ColourContext) -> [f32; 4] {
        ctx.convert(self)
    }
}

#[cfg(test)]
#[path = "transform_tests.rs"]
mod tests;
