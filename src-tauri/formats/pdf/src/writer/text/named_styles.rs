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

//! Default properties for common ODF named styles.

use crate::writer::text::style_props::{ParagraphProps, TextAlign};

/// Get default properties for a style by name.
pub fn get_default_style_props(style_name: &str) -> Option<ParagraphProps> {
    match style_name {
        "Heading 1" | "Heading_20_1" | "h1" => Some(ParagraphProps {
            font_size: 24.0,
            bold: true,
            space_before: 12.0,
            space_after: 6.0,
            keep_with_next: true,
            ..ParagraphProps::default()
        }),
        "Heading 2" | "Heading_20_2" | "h2" => Some(ParagraphProps {
            font_size: 18.0,
            bold: true,
            space_before: 10.0,
            space_after: 4.0,
            keep_with_next: true,
            ..ParagraphProps::default()
        }),
        "Heading 3" | "Heading_20_3" | "h3" => Some(ParagraphProps {
            font_size: 14.0,
            bold: true,
            space_before: 8.0,
            space_after: 2.0,
            keep_with_next: true,
            ..ParagraphProps::default()
        }),
        "Heading 4" | "Heading_20_4" | "h4" => Some(ParagraphProps {
            font_size: 12.0,
            bold: true,
            space_before: 6.0,
            space_after: 2.0,
            keep_with_next: true,
            ..ParagraphProps::default()
        }),
        "Title" => Some(ParagraphProps {
            font_size: 32.0,
            bold: true,
            text_align: TextAlign::Center,
            space_after: 20.0,
            ..ParagraphProps::default()
        }),
        "Subtitle" => Some(ParagraphProps {
            font_size: 16.0,
            text_align: TextAlign::Center,
            space_after: 10.0,
            ..ParagraphProps::default()
        }),
        _ => None,
    }
}
