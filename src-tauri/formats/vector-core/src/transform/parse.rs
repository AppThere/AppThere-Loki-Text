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

use super::Transform;

/// Parse an SVG transform attribute string (may contain multiple transforms).
/// Handles `matrix(…)`, `translate(…)`, `scale(…)`, and `rotate(…)`.
/// Multiple transforms are composed left-to-right.
pub fn parse_svg_transform(s: &str) -> Transform {
    let mut result = Transform::identity();
    let mut s = s.trim();

    while !s.is_empty() {
        if let Some(rest) = s.strip_prefix("matrix(") {
            if let Some(end) = rest.find(')') {
                let nums = parse_nums(&rest[..end]);
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
                let nums = parse_nums(&rest[..end]);
                if !nums.is_empty() {
                    let ty = if nums.len() >= 2 { nums[1] } else { 0.0 };
                    result = result.multiply(&Transform::translate(nums[0], ty));
                }
                s = rest[end + 1..].trim_start();
                continue;
            }
        }
        if let Some(rest) = s.strip_prefix("scale(") {
            if let Some(end) = rest.find(')') {
                let nums = parse_nums(&rest[..end]);
                if !nums.is_empty() {
                    let sy = if nums.len() >= 2 { nums[1] } else { nums[0] };
                    result = result.multiply(&Transform::scale(nums[0], sy));
                }
                s = rest[end + 1..].trim_start();
                continue;
            }
        }
        if let Some(rest) = s.strip_prefix("rotate(") {
            if let Some(end) = rest.find(')') {
                let nums = parse_nums(&rest[..end]);
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
        break; // unknown function — stop
    }
    result
}

fn parse_nums(inner: &str) -> Vec<f64> {
    inner
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.parse::<f64>().ok())
        .collect()
}
