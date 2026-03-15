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

mod parse;
#[cfg(test)]
mod tests;

pub use parse::parse_svg_transform;

/// A 2D affine transform stored as the 6 independent values of a 3×3 matrix.
///
/// The matrix is:
/// ```text
/// | a  c  e |
/// | b  d  f |
/// | 0  0  1 |
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl Transform {
    pub fn identity() -> Self {
        Transform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
    }

    pub fn translate(tx: f64, ty: f64) -> Self {
        Transform { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: tx, f: ty }
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        Transform { a: sx, b: 0.0, c: 0.0, d: sy, e: 0.0, f: 0.0 }
    }

    /// Rotate by `angle_deg` degrees (counter-clockwise in SVG coords).
    pub fn rotate(angle_deg: f64) -> Self {
        let rad = angle_deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        Transform { a: cos, b: sin, c: -sin, d: cos, e: 0.0, f: 0.0 }
    }

    /// Rotate by `angle_deg` degrees around centre (`cx`, `cy`).
    pub fn rotate_around(angle_deg: f64, cx: f64, cy: f64) -> Self {
        Transform::translate(cx, cy)
            .multiply(&Transform::rotate(angle_deg))
            .multiply(&Transform::translate(-cx, -cy))
    }

    /// Multiply two transforms: `self` then `other` (right-multiply).
    pub fn multiply(&self, other: &Transform) -> Self {
        Transform {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// Apply this transform to a point.
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }

    /// Return `"matrix(a b c d e f)"` SVG string.
    pub fn to_svg_matrix(&self) -> String {
        format!("matrix({} {} {} {} {} {})", self.a, self.b, self.c, self.d, self.e, self.f)
    }

    /// Parse `"matrix(a b c d e f)"` string.
    pub fn from_svg_matrix(s: &str) -> Option<Self> {
        let s = s.trim();
        let inner = s.strip_prefix("matrix(")?.strip_suffix(')')?;
        let nums: Vec<f64> = inner
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|p| !p.is_empty())
            .map(|p| p.parse::<f64>().ok())
            .collect::<Option<Vec<_>>>()?;
        if nums.len() != 6 {
            return None;
        }
        Some(Transform { a: nums[0], b: nums[1], c: nums[2], d: nums[3], e: nums[4], f: nums[5] })
    }

    /// Returns true if this transform is the identity (within epsilon).
    pub fn is_identity(&self) -> bool {
        const EPS: f64 = 1e-10;
        (self.a - 1.0).abs() < EPS
            && self.b.abs() < EPS
            && self.c.abs() < EPS
            && (self.d - 1.0).abs() < EPS
            && self.e.abs() < EPS
            && self.f.abs() < EPS
    }
}
