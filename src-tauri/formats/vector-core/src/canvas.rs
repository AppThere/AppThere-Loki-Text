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

use serde::{Deserialize, Serialize};
use crate::units::{LengthUnit, UnitConverter};

/// The canvas (page) dimensions and display settings for a vector document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    /// Canvas width stored in pixels.
    pub width: f64,
    /// Canvas height stored in pixels.
    pub height: f64,
    /// Unit to use when displaying dimensions to the user.
    pub display_unit: LengthUnit,
    /// Screen DPI (default 96.0).
    pub dpi: f64,
    /// Optional SVG viewBox override.
    pub viewbox: Option<ViewBox>,
}

/// SVG viewBox rectangle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Canvas {
    pub fn new(width_px: f64, height_px: f64) -> Self {
        Canvas {
            width: width_px,
            height: height_px,
            display_unit: LengthUnit::Px,
            dpi: 96.0,
            viewbox: None,
        }
    }

    /// A4 portrait: 210mm × 297mm at 96 DPI.
    pub fn a4_portrait() -> Self {
        let c = UnitConverter::new(96.0);
        Canvas::new(c.to_px(210.0, LengthUnit::Mm), c.to_px(297.0, LengthUnit::Mm))
            .with_display_unit(LengthUnit::Mm)
    }

    /// A4 landscape: 297mm × 210mm at 96 DPI.
    pub fn a4_landscape() -> Self {
        let c = UnitConverter::new(96.0);
        Canvas::new(c.to_px(297.0, LengthUnit::Mm), c.to_px(210.0, LengthUnit::Mm))
            .with_display_unit(LengthUnit::Mm)
    }

    /// US Letter portrait: 8.5in × 11in at 96 DPI.
    pub fn letter_portrait() -> Self {
        let c = UnitConverter::new(96.0);
        Canvas::new(c.to_px(8.5, LengthUnit::In), c.to_px(11.0, LengthUnit::In))
            .with_display_unit(LengthUnit::In)
    }

    fn with_display_unit(mut self, unit: LengthUnit) -> Self {
        self.display_unit = unit;
        self
    }

    /// Width in the current display unit.
    pub fn display_width(&self) -> f64 {
        let c = UnitConverter::new(self.dpi);
        c.from_px(self.width, self.display_unit)
    }

    /// Height in the current display unit.
    pub fn display_height(&self) -> f64 {
        let c = UnitConverter::new(self.dpi);
        c.from_px(self.height, self.display_unit)
    }
}
