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

//! Utilities for ODT style writing.

/// Returns `true` if `key` is a paragraph-property attribute.
pub fn is_paragraph_property(key: &str) -> bool {
    key.starts_with("fo:margin")
        || key.starts_with("fo:text-indent")
        || key.starts_with("fo:text-align")
        || key.starts_with("fo:orphans")
        || key.starts_with("fo:widows")
        || key.starts_with("fo:hyphenate")
        || key.starts_with("fo:break-")
        || key == "fo:line-height"
}

/// Returns `true` if `key` is a text-property attribute.
pub fn is_text_property(key: &str) -> bool {
    key.starts_with("fo:font")
        || key.starts_with("fo:color")
        || key.starts_with("fo:font-size")
        || key.starts_with("fo:font-weight")
        || key.starts_with("fo:font-style")
        || key.starts_with("fo:text-transform")
        || key == "fo:background-color"
        || key == "loki:colour"
}

/// Normalizes unitless line-height values to ODF percent format.
pub fn coerce_line_height(key: &str, value: &str) -> String {
    if key != "fo:line-height" {
        return value.to_string();
    }
    let val = value.trim();
    if val.chars().all(|c| c.is_ascii_digit() || c == '.') {
        if let Ok(num) = val.parse::<f32>() {
            return format!("{}%", (num * 100.0).round());
        }
    }
    value.to_string()
}
