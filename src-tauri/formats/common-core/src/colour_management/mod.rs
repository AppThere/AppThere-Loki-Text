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

//! Colour management infrastructure for AppThere Loki.
//!
//! This module provides:
//! - [`Colour`]: multi-space colour value type (RGB, CMYK, Lab, Spot, Linked)
//! - [`ColourSpace`] and [`DocumentColourSettings`]: document colour settings
//! - [`SwatchLibrary`]: named colour palette management
//! - [`lookup_pantone`]: PMS colour lookup by name
//! - [`IccProfileStore`] and [`ColourContext`]: LCMS2-backed colour transforms
//!   (requires `colour-management` feature)
//!
//! # Feature Flag
//!
//! The `colour-management` feature enables LCMS2 integration. Without it,
//! all types compile but LCMS2-backed conversion is unavailable.

pub mod colour;
pub mod pantone;
pub mod space;
pub mod swatch;

#[cfg(feature = "colour-management")]
pub mod profile;

#[cfg(feature = "colour-management")]
pub mod transform;

// Public re-exports — the surface that other crates import from.
pub use colour::Colour;
pub use space::{
    BuiltInProfile, ColourSpace, DocumentColourSettings,
    IccProfileRef, RenderingIntent,
};
pub use swatch::{ColourSwatch, SwatchId, SwatchLibrary};
pub use pantone::lookup_pantone;

#[cfg(feature = "colour-management")]
pub use profile::IccProfileStore;

#[cfg(feature = "colour-management")]
pub use transform::ColourContext;

/// Convenience: create a display ColourContext from document colour settings.
///
/// This wires together [`IccProfileStore`] and [`ColourContext`] in one call,
/// which is the common pattern for Tauri command handlers.
#[cfg(feature = "colour-management")]
pub fn create_display_context(
    settings: &DocumentColourSettings,
) -> Result<ColourContext, String> {
    let mut store = IccProfileStore::new();
    ColourContext::new_for_display(settings, &mut store)
}
