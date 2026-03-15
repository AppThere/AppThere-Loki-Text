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
        Some(Transform {
            a: nums[0], b: nums[1], c: nums[2],
            d: nums[3], e: nums[4], f: nums[5],
        })
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

/// Parse an SVG transform attribute string (may contain multiple transforms).
pub fn parse_svg_transform(s: &str) -> Transform {
    let mut result = Transform::identity();
    let mut s = s.trim();

    while !s.is_empty() {
        if let Some(rest) = s.strip_prefix("matrix(") {
            if let Some(end) = rest.find(')') {
                let inner = &rest[..end];
                let nums: Vec<f64> = inner
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .filter(|p| !p.is_empty())
                    .filter_map(|p| p.parse::<f64>().ok())
                    .collect();
                if nums.len() == 6 {
                    let t = Transform {
                        a: nums[0], b: nums[1], c: nums[2],
                        d: nums[3], e: nums[4], f: nums[5],
                    };
                    result = result.multiply(&t);
                }
                s = rest[end + 1..].trim_start();
                continue;
            }
        }
        if let Some(rest) = s.strip_prefix("translate(") {
            if let Some(end) = rest.find(')') {
                let inner = &rest[..end];
                let nums: Vec<f64> = inner
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .filter(|p| !p.is_empty())
                    .filter_map(|p| p.parse::<f64>().ok())
                    .collect();
                if !nums.is_empty() {
                    let tx = nums[0];
                    let ty = if nums.len() >= 2 { nums[1] } else { 0.0 };
                    result = result.multiply(&Transform::translate(tx, ty));
                }
                s = rest[end + 1..].trim_start();
                continue;
            }
        }
        if let Some(rest) = s.strip_prefix("scale(") {
            if let Some(end) = rest.find(')') {
                let inner = &rest[..end];
                let nums: Vec<f64> = inner
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .filter(|p| !p.is_empty())
                    .filter_map(|p| p.parse::<f64>().ok())
                    .collect();
                if !nums.is_empty() {
                    let sx = nums[0];
                    let sy = if nums.len() >= 2 { nums[1] } else { sx };
                    result = result.multiply(&Transform::scale(sx, sy));
                }
                s = rest[end + 1..].trim_start();
                continue;
            }
        }
        if let Some(rest) = s.strip_prefix("rotate(") {
            if let Some(end) = rest.find(')') {
                let inner = &rest[..end];
                let nums: Vec<f64> = inner
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .filter(|p| !p.is_empty())
                    .filter_map(|p| p.parse::<f64>().ok())
                    .collect();
                let t = if nums.len() >= 3 {
                    Transform::rotate_around(nums[0], nums[1], nums[2])
                } else if !nums.is_empty() {
                    Transform::rotate(nums[0])
                } else {
                    Transform::identity()
                };
                result = result.multiply(&t);
                s = rest[end + 1..].trim_start();
                continue;
            }
        }
        // Skip unknown transform
        break;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-9;

    fn eq(a: f64, b: f64) -> bool { (a - b).abs() < EPS }

    #[test]
    fn test_identity_apply() {
        let t = Transform::identity();
        let (x, y) = t.apply(3.0, 4.0);
        assert!(eq(x, 3.0) && eq(y, 4.0));
    }

    #[test]
    fn test_translate() {
        let t = Transform::translate(10.0, 20.0);
        let (x, y) = t.apply(0.0, 0.0);
        assert!(eq(x, 10.0) && eq(y, 20.0));
    }

    #[test]
    fn test_scale() {
        let t = Transform::scale(2.0, 3.0);
        let (x, y) = t.apply(5.0, 4.0);
        assert!(eq(x, 10.0) && eq(y, 12.0));
    }

    #[test]
    fn test_rotate_90() {
        let t = Transform::rotate(90.0);
        let (x, y) = t.apply(1.0, 0.0);
        assert!(eq(x, 0.0) && eq(y, 1.0), "got ({}, {})", x, y);
    }

    #[test]
    fn test_multiply_associativity() {
        let a = Transform::translate(5.0, 0.0);
        let b = Transform::scale(2.0, 2.0);
        let c = Transform::rotate(45.0);
        let ab_c = a.multiply(&b).multiply(&c);
        let a_bc = a.multiply(&b.multiply(&c));
        assert!(eq(ab_c.a, a_bc.a) && eq(ab_c.e, a_bc.e));
    }

    #[test]
    fn test_svg_matrix_roundtrip() {
        let t = Transform { a: 1.0, b: 0.5, c: -0.5, d: 1.0, e: 10.0, f: 20.0 };
        let s = t.to_svg_matrix();
        let back = Transform::from_svg_matrix(&s).unwrap();
        assert!(eq(t.a, back.a) && eq(t.e, back.e) && eq(t.f, back.f));
    }

    #[test]
    fn test_is_identity() {
        assert!(Transform::identity().is_identity());
        assert!(!Transform::translate(1.0, 0.0).is_identity());
    }
}
