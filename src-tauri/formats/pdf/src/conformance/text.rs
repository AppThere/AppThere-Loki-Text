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

use std::collections::HashMap;
use common_core::block::Block;
use common_core::colour_management::Colour;
use common_core::style::StyleDefinition;
use common_core::Metadata;
use crate::export_settings::{PdfExportSettings, PdfXStandard};
use super::types::ConformanceViolation;
use super::vector::check_output_condition;

/// Validate a text document (blocks + styles + metadata) against PDF/X settings.
pub fn validate_text(
    blocks: &[Block],
    styles: &HashMap<String, StyleDefinition>,
    metadata: &Metadata,
    settings: &PdfExportSettings,
) -> Vec<ConformanceViolation> {
    let mut violations = Vec::new();

    check_output_condition(settings, &mut violations);
    check_text_not_empty(blocks, &mut violations);
    check_text_title(metadata, &mut violations);

    #[cfg(feature = "colour-management")]
    check_text_colours_in_styles(styles, settings.standard, &mut violations);

    violations
}

pub fn check_text_not_empty(blocks: &[Block], violations: &mut Vec<ConformanceViolation>) {
    let has_content = blocks.iter().any(|b| match b {
        Block::Paragraph { content, .. } | Block::Heading { content, .. } => !content.is_empty(),
        Block::BulletList { content } | Block::OrderedList { content } => !content.is_empty(),
        Block::Table { content } => !content.is_empty(),
        Block::HorizontalRule | Block::PageBreak => true,
        _ => false,
    });
    if !has_content {
        violations.push(ConformanceViolation::new(
            "X/text-empty-document",
            "Text document has no renderable content; the exported PDF will be blank",
        ));
    }
}

pub fn check_text_title(metadata: &Metadata, violations: &mut Vec<ConformanceViolation>) {
    let has_title = metadata
        .title
        .as_deref()
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);
    if !has_title {
        violations.push(ConformanceViolation {
            rule: "X/text-missing-title".to_string(),
            message: "Document has no title; PDF/X metadata will be incomplete".to_string(),
            auto_fixable: true,
        });
    }
}

#[cfg(feature = "colour-management")]
pub fn check_text_colours_in_styles(
    styles: &HashMap<String, StyleDefinition>,
    standard: PdfXStandard,
    violations: &mut Vec<ConformanceViolation>,
) {
    for (name, style) in styles {
        if let Some(colour) = &style.font_colour {
            check_text_colour(colour, standard, &format!("font_colour in style '{}'" , name), violations);
        }
        if let Some(colour) = &style.background_colour {
            check_text_colour(colour, standard, &format!("background_colour in style '{}'", name), violations);
        }
    }
}

#[cfg(feature = "colour-management")]
pub fn check_text_colour(
    colour: &Colour,
    standard: PdfXStandard,
    location: &str,
    violations: &mut Vec<ConformanceViolation>,
) {
    match colour {
        Colour::Linked { id } => {
            violations.push(ConformanceViolation::new(
                "X/text-unresolved-linked-colour",
                format!("Unresolved Linked colour id={} at {}", id, location),
            ));
        }
        Colour::Spot { cmyk_fallback, .. } => {
            check_text_colour(cmyk_fallback, standard, location, violations);
        }
        Colour::Rgb { .. } => {
            if standard.requires_cmyk_only() {
                violations.push(ConformanceViolation::new(
                    "X1a/text-no-rgb-colour",
                    format!("RGB colour at {} not allowed in PDF/X-1a", location),
                ));
            }
        }
        Colour::Cmyk { .. } | Colour::Lab { .. } => {}
    }
}
