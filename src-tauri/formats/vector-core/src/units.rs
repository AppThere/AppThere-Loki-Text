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

/// Length units supported by the vector editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LengthUnit {
    Px,
    Mm,
    Cm,
    In,
    Pt,
    Pc,
}

/// Converts between length units and pixels at a given DPI.
///
/// Conversion factors at 96 DPI:
/// - 1in = 96px
/// - 1mm = 96/25.4 px ≈ 3.7795px
/// - 1cm = 960/25.4 px ≈ 37.795px
/// - 1pt = 96/72 px ≈ 1.3333px
/// - 1pc = 16px
pub struct UnitConverter {
    pub dpi: f64,
}

impl UnitConverter {
    pub fn new(dpi: f64) -> Self {
        Self { dpi }
    }

    pub fn at_96_dpi() -> Self {
        Self { dpi: 96.0 }
    }

    /// Convert a value in the given unit to pixels.
    pub fn to_px(&self, value: f64, unit: LengthUnit) -> f64 {
        let factor = self.dpi / 96.0;
        match unit {
            LengthUnit::Px => value,
            LengthUnit::Mm => value * (96.0 / 25.4) * factor,
            LengthUnit::Cm => value * (960.0 / 25.4) * factor,
            LengthUnit::In => value * 96.0 * factor,
            LengthUnit::Pt => value * (96.0 / 72.0) * factor,
            LengthUnit::Pc => value * 16.0 * factor,
        }
    }

    /// Convert a value in pixels to the given unit.
    pub fn from_px(&self, value: f64, unit: LengthUnit) -> f64 {
        let factor = self.dpi / 96.0;
        match unit {
            LengthUnit::Px => value,
            LengthUnit::Mm => value / ((96.0 / 25.4) * factor),
            LengthUnit::Cm => value / ((960.0 / 25.4) * factor),
            LengthUnit::In => value / (96.0 * factor),
            LengthUnit::Pt => value / ((96.0 / 72.0) * factor),
            LengthUnit::Pc => value / (16.0 * factor),
        }
    }

    /// Return the CSS/SVG suffix string for a unit.
    pub fn unit_suffix(unit: LengthUnit) -> &'static str {
        match unit {
            LengthUnit::Px => "px",
            LengthUnit::Mm => "mm",
            LengthUnit::Cm => "cm",
            LengthUnit::In => "in",
            LengthUnit::Pt => "pt",
            LengthUnit::Pc => "pc",
        }
    }

    /// Parse a length string with optional unit suffix into pixels.
    pub fn parse_length(&self, s: &str) -> Option<f64> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        if let Some(v) = s.strip_suffix("px") {
            return v.trim().parse::<f64>().ok();
        }
        if let Some(v) = s.strip_suffix("mm") {
            return v.trim().parse::<f64>().ok().map(|n| self.to_px(n, LengthUnit::Mm));
        }
        if let Some(v) = s.strip_suffix("cm") {
            return v.trim().parse::<f64>().ok().map(|n| self.to_px(n, LengthUnit::Cm));
        }
        if let Some(v) = s.strip_suffix("in") {
            return v.trim().parse::<f64>().ok().map(|n| self.to_px(n, LengthUnit::In));
        }
        if let Some(v) = s.strip_suffix("pt") {
            return v.trim().parse::<f64>().ok().map(|n| self.to_px(n, LengthUnit::Pt));
        }
        if let Some(v) = s.strip_suffix("pc") {
            return v.trim().parse::<f64>().ok().map(|n| self.to_px(n, LengthUnit::Pc));
        }
        s.parse::<f64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-9;

    fn nearly_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_px_roundtrip_96dpi() {
        let c = UnitConverter::new(96.0);
        let units = [LengthUnit::Px, LengthUnit::Mm, LengthUnit::Cm,
                     LengthUnit::In, LengthUnit::Pt, LengthUnit::Pc];
        for unit in units {
            let original = 42.0;
            let px = c.to_px(original, unit);
            let back = c.from_px(px, unit);
            assert!(nearly_eq(original, back),
                "roundtrip failed for {:?}: {} -> {} -> {}", unit, original, px, back);
        }
    }

    #[test]
    fn test_px_roundtrip_300dpi() {
        let c = UnitConverter::new(300.0);
        let units = [LengthUnit::Px, LengthUnit::Mm, LengthUnit::Cm,
                     LengthUnit::In, LengthUnit::Pt, LengthUnit::Pc];
        for unit in units {
            let original = 42.0;
            let px = c.to_px(original, unit);
            let back = c.from_px(px, unit);
            assert!(nearly_eq(original, back),
                "roundtrip failed at 300dpi for {:?}: {} -> {} -> {}", unit, original, px, back);
        }
    }

    #[test]
    fn test_known_conversions_96dpi() {
        let c = UnitConverter::new(96.0);
        assert!(nearly_eq(c.to_px(1.0, LengthUnit::In), 96.0));
        assert!(nearly_eq(c.to_px(1.0, LengthUnit::Pc), 16.0));
        let mm_px = c.to_px(1.0, LengthUnit::Mm);
        assert!((mm_px - 3.7795275591).abs() < 1e-6);
    }

    #[test]
    fn test_parse_length() {
        let c = UnitConverter::new(96.0);
        assert!(nearly_eq(c.parse_length("100px").unwrap(), 100.0));
        assert!(nearly_eq(c.parse_length("25.4mm").unwrap(), 96.0));
        assert!(nearly_eq(c.parse_length("1in").unwrap(), 96.0));
        assert!(nearly_eq(c.parse_length("72pt").unwrap(), 96.0));
    }
}
