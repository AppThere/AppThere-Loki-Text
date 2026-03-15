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

use crate::canvas::Canvas;
use crate::layer::Layer;
use common_core::colour_management::{
    BuiltInProfile, ColourSpace, DocumentColourSettings, IccProfileRef, SwatchLibrary,
};
use common_core::Metadata;
use serde::{Deserialize, Serialize};

/// A complete vector image document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub canvas: Canvas,
    pub layers: Vec<Layer>,
    pub metadata: Metadata,
    /// Colour management settings for this document.
    /// Defaults to sRGB with relative colorimetric intent.
    #[serde(default)]
    pub colour_settings: DocumentColourSettings,
    /// Named colour swatches for this document.
    #[serde(default)]
    pub swatch_library: SwatchLibrary,
}

impl VectorDocument {
    /// Create a new document with one default layer named "Layer 1".
    pub fn new(canvas: Canvas) -> Self {
        VectorDocument {
            canvas,
            layers: vec![Layer::new("Layer 1")],
            metadata: Metadata::default(),
            colour_settings: DocumentColourSettings::default(),
            swatch_library: SwatchLibrary::new(),
        }
    }

    /// Create a new document with custom colour settings and one default layer.
    pub fn new_with_settings(canvas: Canvas, colour_settings: DocumentColourSettings) -> Self {
        VectorDocument {
            canvas,
            layers: vec![Layer::new("Layer 1")],
            metadata: Metadata::default(),
            colour_settings,
            swatch_library: SwatchLibrary::new(),
        }
    }

    pub fn blank_a4() -> Self {
        VectorDocument::new(Canvas::a4_portrait())
    }

    pub fn blank_letter() -> Self {
        VectorDocument::new(Canvas::letter_portrait())
    }

    /// Create a blank A4 portrait CMYK document.
    ///
    /// Uses the ISO Coated v2 press profile — the standard for European offset
    /// printing on coated stock. Suitable for print-ready PDF export.
    pub fn blank_a4_cmyk() -> Self {
        VectorDocument::new_with_settings(
            Canvas::a4_portrait(),
            DocumentColourSettings {
                working_space: ColourSpace::Cmyk {
                    profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
                },
                ..DocumentColourSettings::default()
            },
        )
    }

    /// Create a blank US Letter portrait CMYK document.
    ///
    /// Uses the SWOP v2 press profile — the standard for North American offset
    /// printing. Suitable for print-ready PDF export.
    pub fn blank_letter_cmyk() -> Self {
        VectorDocument::new_with_settings(
            Canvas::letter_portrait(),
            DocumentColourSettings {
                working_space: ColourSpace::Cmyk {
                    profile: IccProfileRef::BuiltIn(BuiltInProfile::SwopV2),
                },
                ..DocumentColourSettings::default()
            },
        )
    }
}
