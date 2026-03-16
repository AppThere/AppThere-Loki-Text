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

use crate::export_settings::{PdfExportSettings, PdfXStandard};
use common_core::colour_management::{Colour, ColourSpace};
use std::collections::HashMap;
use vector_core::document::VectorDocument;
use super::iter::{for_each_colour, for_each_object};
use super::types::{ConformanceViolation, ConformanceReport};

/// Validate a document against the given export settings.
pub fn validate(document: &VectorDocument, settings: &PdfExportSettings) -> ConformanceReport {
    let standard = settings.standard;
    let mut violations = Vec::new();

    check_output_condition(settings, &mut violations);
    check_colour_spaces(document, settings, &mut violations);
    check_transparency(document, settings, &mut violations);
    check_linked_colours(document, &mut violations);
    check_bleed(settings, &mut violations);
    check_document_not_empty(document, &mut violations);

    ConformanceReport {
        standard,
        violations,
    }
}

pub fn check_output_condition(
    settings: &PdfExportSettings,
    violations: &mut Vec<ConformanceViolation>,
) {
    if settings.output_condition_identifier.trim().is_empty() {
        violations.push(ConformanceViolation::new(
            "X/output-condition",
            "OutputConditionIdentifier must not be empty (PDF/X requirement)",
        ));
    }
}

pub fn check_colour_spaces(
    document: &VectorDocument,
    settings: &PdfExportSettings,
    violations: &mut Vec<ConformanceViolation>,
) {
    let standard = settings.standard;
    if standard.requires_cmyk_only() {
        match &document.colour_settings.working_space {
            ColourSpace::Cmyk { .. } => {}
            other => {
                violations.push(ConformanceViolation::auto_fixable(
                    "X1a/working-space-must-be-cmyk",
                    format!(
                        "Working space {:?} will be converted to CMYK automatically during export",
                        other
                    ),
                ));
            }
        }
    }

    for_each_colour(document, |colour, location| {
        check_colour_compatibility(colour, standard, location, violations);
    });
}

pub fn check_colour_compatibility(
    colour: &Colour,
    standard: PdfXStandard,
    location: &str,
    violations: &mut Vec<ConformanceViolation>,
) {
    match colour {
        Colour::Rgb { .. } => {
            if !standard.allows_rgb() {
                violations.push(ConformanceViolation::auto_fixable(
                    "X1a/no-rgb",
                    format!(
                        "RGB colour at {} will be converted to CMYK automatically during export",
                        location
                    ),
                ));
            }
        }
        Colour::Lab { .. } => {
            if standard.requires_cmyk_only() {
                violations.push(ConformanceViolation::auto_fixable(
                    "X1a/no-lab",
                    format!(
                        "Lab colour at {} will be converted to CMYK automatically during export",
                        location
                    ),
                ));
            }
        }
        Colour::Cmyk { .. } => {}
        Colour::Spot { cmyk_fallback, .. } => {
            check_colour_compatibility(cmyk_fallback, standard, location, violations);
        }
        Colour::Linked { id } => {
            violations.push(ConformanceViolation::new(
                "X/unresolved-linked-colour",
                format!("Unresolved Linked colour id={} at {}", id, location),
            ));
        }
    }
}

pub fn check_transparency(
    document: &VectorDocument,
    settings: &PdfExportSettings,
    violations: &mut Vec<ConformanceViolation>,
) {
    let standard = settings.standard;
    if standard.allows_transparency() {
        return;
    }
    let dpi = settings.resolution_dpi;

    for_each_object(document, |obj, location| {
        let style = &obj.common().style;
        if style.opacity < 1.0 - f64::EPSILON {
            violations.push(ConformanceViolation::auto_fixable(
                "X1a/no-transparency",
                format!(
                    "Transparency at {} will be flattened at {} DPI during export",
                    location, dpi
                ),
            ));
        }
        if style.fill_opacity < 1.0 - f64::EPSILON {
            violations.push(ConformanceViolation::auto_fixable(
                "X1a/no-fill-opacity",
                format!(
                    "Fill opacity at {} will be flattened at {} DPI during export",
                    location, dpi
                ),
            ));
        }
        if style.stroke_opacity < 1.0 - f64::EPSILON {
            violations.push(ConformanceViolation::auto_fixable(
                "X1a/no-stroke-opacity",
                format!(
                    "Stroke opacity at {} will be flattened at {} DPI during export",
                    location, dpi
                ),
            ));
        }
    });
}

pub fn check_linked_colours(document: &VectorDocument, violations: &mut Vec<ConformanceViolation>) {
    for_each_colour(document, |colour, location| {
        if let Colour::Linked { id } = colour {
            let rule = "X/unresolved-linked-colour";
            let already = violations
                .iter()
                .any(|v| v.rule == rule && v.message.contains(id.as_str()));
            if !already {
                violations.push(ConformanceViolation::new(
                    rule,
                    format!("Unresolved Linked colour id={} at {}", id, location),
                ));
            }
        }
    });
}

pub fn check_bleed(settings: &PdfExportSettings, violations: &mut Vec<ConformanceViolation>) {
    if settings.bleed_pt < 0.0 {
        violations.push(ConformanceViolation::new(
            "X/negative-bleed",
            format!("Bleed must be \u{2265} 0 pt; got {}", settings.bleed_pt),
        ));
    }
}

pub fn check_document_not_empty(document: &VectorDocument, violations: &mut Vec<ConformanceViolation>) {
    let has_objects = document.layers.iter().any(|l| !l.objects.is_empty());
    if !has_objects {
        violations.push(ConformanceViolation::new(
            "X/empty-document",
            "Document has no renderable objects; the exported PDF will be blank",
        ));
    }
}
