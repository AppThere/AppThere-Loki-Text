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

//! Style property resolution and ODF attribute parsing.

use common_core::style::StyleDefinition;
use std::collections::HashMap;

/// Text alignment within a paragraph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Resolved properties for a paragraph (block level).
#[derive(Debug, Clone, PartialEq)]
pub struct ParagraphProps {
    pub font_family: String,
    pub font_size: f64,
    pub line_height_factor: f64, // e.g. 1.2 for 120%
    pub text_align: TextAlign,
    pub margin_left: f64,
    pub margin_right: f64,
    pub text_indent: f64,
    pub space_before: f64,
    pub space_after: f64,
    pub break_before: bool,
    pub keep_with_next: bool,
    pub bold: bool,
    pub italic: bool,
}

impl Default for ParagraphProps {
    fn default() -> Self {
        Self {
            font_family: "public sans".to_string(),
            font_size: 11.0,
            line_height_factor: 1.35,
            text_align: TextAlign::Left,
            margin_left: 0.0,
            margin_right: 0.0,
            text_indent: 0.0,
            space_before: 0.0,
            space_after: 0.0,
            break_before: false,
            keep_with_next: false,
            bold: false,
            italic: false,
        }
    }
}

/// Resolve paragraph properties by merging named style attributes into defaults.
pub fn resolve_paragraph_props(
    style_name: Option<&str>,
    styles: &HashMap<String, StyleDefinition>,
    level: Option<u32>,
) -> ParagraphProps {
    let mut props = style_name
        .and_then(|n| crate::writer::text::named_styles::get_default_style_props(n))
        .or_else(|| {
            level.and_then(|l| {
                let name = format!("Heading {}", l);
                crate::writer::text::named_styles::get_default_style_props(&name)
            })
        })
        .unwrap_or_default();

    // 1. Apply named style if it exists
    if let Some(style) = style_name.and_then(|n| styles.get(n)) {
        if let Some(family) = style
            .attributes
            .get("fo:font-family")
            .or_else(|| style.attributes.get("style:font-name"))
        {
            props.font_family = family.to_lowercase();
        }

        if let Some(size_str) = style.attributes.get("fo:font-size") {
            if let Some(size) = parse_length(size_str) {
                props.font_size = size;
            }
        }

        if let Some(lh_str) = style.attributes.get("fo:line-height") {
            props.line_height_factor = parse_line_height(lh_str, props.font_size);
        }

        if let Some(align_str) = style.attributes.get("fo:text-align") {
            props.text_align = match align_str.as_str() {
                "center" => TextAlign::Center,
                "right" | "end" => TextAlign::Right,
                "justify" => TextAlign::Justify,
                _ => TextAlign::Left,
            };
        }

        if let Some(val) = style.attributes.get("fo:margin-left") {
            props.margin_left = parse_length(val).unwrap_or(0.0);
        }
        if let Some(val) = style.attributes.get("fo:margin-right") {
            props.margin_right = parse_length(val).unwrap_or(0.0);
        }
        if let Some(val) = style.attributes.get("fo:text-indent") {
            props.text_indent = parse_length(val).unwrap_or(0.0);
        }
        if let Some(val) = style.attributes.get("fo:margin-top") {
            props.space_before = parse_length(val).unwrap_or(0.0);
        }
        if let Some(val) = style.attributes.get("fo:margin-bottom") {
            props.space_after = parse_length(val).unwrap_or(0.0);
        }

        if let Some(val) = style.attributes.get("fo:break-before") {
            props.break_before = val == "page";
        }
        if let Some(val) = style.attributes.get("fo:keep-with-next") {
            props.keep_with_next = val == "always" || val == "true";
        }

        if let Some(val) = style.attributes.get("fo:font-weight") {
            props.bold = val == "bold" || val.parse::<u16>().map(|w| w >= 700).unwrap_or(false);
        }
        if let Some(val) = style.attributes.get("fo:font-style") {
            props.italic = val == "italic";
        }
    }

    props
}

/// Parse a length string (e.g. "12pt", "1in", "2.54cm") into points.
pub fn parse_length(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if s.ends_with("pt") {
        s[..s.len() - 2].parse::<f64>().ok()
    } else if s.ends_with("in") {
        s[..s.len() - 2].parse::<f64>().ok().map(|v| v * 72.0)
    } else if s.ends_with("cm") {
        s[..s.len() - 2].parse::<f64>().ok().map(|v| v * 28.346)
    } else if s.ends_with("mm") {
        s[..s.len() - 2].parse::<f64>().ok().map(|v| v * 2.8346)
    } else {
        s.parse::<f64>().ok()
    }
}

/// Parse line height which can be a factor ("1.2"), a percentage ("120%"), or a length ("15pt").
/// Returns a multiplier relative to font_size.
pub fn parse_line_height(s: &str, font_size: f64) -> f64 {
    let s = s.trim();
    if s.ends_with('%') {
        s[..s.len() - 1]
            .parse::<f64>()
            .ok()
            .map(|v| v / 100.0)
            .unwrap_or(1.35)
    } else if s.ends_with("pt") || s.ends_with("in") || s.ends_with("cm") || s.ends_with("mm") {
        parse_length(s).map(|v| v / font_size).unwrap_or(1.35)
    } else if let Ok(factor) = s.parse::<f64>() {
        factor
    } else {
        1.35
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_length() {
        assert_eq!(parse_length("12pt"), Some(12.0));
        assert_eq!(parse_length("1in"), Some(72.0));
        assert!((parse_length("1cm").unwrap() - 28.346).abs() < 0.01);
        assert_eq!(parse_length("12"), Some(12.0));
    }

    #[test]
    fn test_parse_line_height() {
        assert_eq!(parse_line_height("1.5", 10.0), 1.5);
        assert_eq!(parse_line_height("150%", 10.0), 1.5);
        assert_eq!(parse_line_height("20pt", 10.0), 2.0);
    }
}
