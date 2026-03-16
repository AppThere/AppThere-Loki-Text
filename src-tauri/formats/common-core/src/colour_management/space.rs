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

//! Colour space types and document colour settings.

use serde::{Deserialize, Serialize};

/// The colour space in which a document's colours are interpreted and blended.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColourSpace {
    /// Standard sRGB (IEC 61966-2-1). Default for screen documents.
    Srgb,
    /// Display P3. Wide gamut, used by modern Apple displays.
    DisplayP3,
    /// Adobe RGB (1998). Wider gamut than sRGB, used in photography.
    AdobeRgb,
    /// CMYK with an associated ICC press profile.
    Cmyk { profile: IccProfileRef },
    /// Any colour space defined by a user-supplied ICC profile.
    Custom { profile: IccProfileRef },
}

/// A reference to an ICC profile, either a well-known built-in or a
/// user-supplied file path.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IccProfileRef {
    /// One of the profiles bundled with Loki.
    BuiltIn(BuiltInProfile),
    /// Absolute path to a user-supplied ICC file.
    FilePath(String),
}

/// The ICC profiles bundled with Loki.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuiltInProfile {
    /// sRGB IEC61966-2.1 — universal screen RGB profile.
    SrgbIec61966,
    /// ISO Coated v2 — European offset printing on coated stock.
    IsoCoatedV2,
    /// SWOP v2 — North American offset printing.
    SwopV2,
    /// GRACoL 2006 — North American high-quality offset printing.
    GraCol2006,
}

impl BuiltInProfile {
    /// Human-readable name for UI display.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SrgbIec61966 => "sRGB IEC61966-2.1",
            Self::IsoCoatedV2 => "ISO Coated v2",
            Self::SwopV2 => "SWOP v2",
            Self::GraCol2006 => "GRACoL 2006",
        }
    }

    /// Short description of the intended use case.
    pub fn description(&self) -> &'static str {
        match self {
            Self::SrgbIec61966 => "Standard screen RGB. Default for digital delivery.",
            Self::IsoCoatedV2 => "European offset printing on coated paper stock.",
            Self::SwopV2 => "North American offset printing. SWOP standard.",
            Self::GraCol2006 => "North American high-quality offset printing.",
        }
    }
}

/// Determines how out-of-gamut colours are mapped during colour space conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingIntent {
    /// Compress the full gamut proportionally. Best for photographs.
    Perceptual,
    /// Clip out-of-gamut colours, preserve in-gamut colours exactly.
    /// Best for logos and flat colour artwork.
    RelativeColorimetric,
    /// Maximise saturation at the expense of accuracy. Best for charts.
    Saturation,
    /// Like RelativeColorimetric but also simulates the output medium's
    /// white point. Used for soft-proofing on paper with a tint.
    AbsoluteColorimetric,
}

/// Colour management settings attached to every document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentColourSettings {
    /// The colour space in which document colours are stored and blended.
    pub working_space: ColourSpace,
    /// How out-of-gamut colours are handled during conversion.
    pub rendering_intent: RenderingIntent,
    /// Whether to apply black point compensation during conversion.
    /// Recommended true for most workflows.
    pub blackpoint_compensation: bool,
}

impl Default for DocumentColourSettings {
    fn default() -> Self {
        Self {
            working_space: ColourSpace::Srgb,
            rendering_intent: RenderingIntent::RelativeColorimetric,
            blackpoint_compensation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_profile_display_name_non_empty() {
        let profiles = [
            BuiltInProfile::SrgbIec61966,
            BuiltInProfile::IsoCoatedV2,
            BuiltInProfile::SwopV2,
            BuiltInProfile::GraCol2006,
        ];
        for profile in &profiles {
            assert!(
                !profile.display_name().is_empty(),
                "display_name empty for {:?}",
                profile
            );
            assert!(
                !profile.description().is_empty(),
                "description empty for {:?}",
                profile
            );
        }
    }

    #[test]
    fn document_colour_settings_default() {
        let settings = DocumentColourSettings::default();
        assert_eq!(settings.working_space, ColourSpace::Srgb);
        assert_eq!(
            settings.rendering_intent,
            RenderingIntent::RelativeColorimetric
        );
        assert!(settings.blackpoint_compensation);
    }

    #[test]
    fn icc_profile_ref_serde_round_trip() {
        let built_in = IccProfileRef::BuiltIn(BuiltInProfile::SrgbIec61966);
        let json = serde_json::to_string(&built_in).unwrap();
        let round_tripped: IccProfileRef = serde_json::from_str(&json).unwrap();
        assert_eq!(built_in, round_tripped);

        let file_path = IccProfileRef::FilePath("/usr/share/icc/AdobeRGB.icc".to_string());
        let json = serde_json::to_string(&file_path).unwrap();
        let round_tripped: IccProfileRef = serde_json::from_str(&json).unwrap();
        assert_eq!(file_path, round_tripped);
    }
}
