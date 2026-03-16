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

//! PDF/X conformance validator.
//!
//! This module validates a `VectorDocument` against a target PDF/X standard
//! **before** any PDF bytes are written. The validator is the specification
//! made executable: only documents that pass all checks are written.

mod iter;

use serde::{Deserialize, Serialize};
use crate::error::PdfError;
use crate::export_settings::{PdfExportSettings, PdfXStandard};
use common_core::block::Block;
use common_core::colour_management::{Colour, ColourSpace};
use common_core::style::StyleDefinition;
use common_core::Metadata;
use iter::{for_each_colour, for_each_object};
use std::collections::HashMap;
use vector_core::document::VectorDocument;

/// A single conformance violation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConformanceViolation {
    /// Short rule identifier, e.g. "X1a/no-transparency".
    pub rule: String,
    /// Human-readable description of the violation.
    pub message: String,
    /// Whether the export pipeline will resolve this violation automatically.
    /// When true, the violation does not block export; the pipeline handles it.
    pub auto_fixable: bool,
}

impl ConformanceViolation {
    fn new(rule: impl Into<String>, message: impl Into<String>) -> Self {
        ConformanceViolation {
            rule: rule.into(),
            message: message.into(),
            auto_fixable: false,
        }
    }

    fn auto_fixable(rule: impl Into<String>, message: impl Into<String>) -> Self {
        ConformanceViolation {
            rule: rule.into(),
            message: message.into(),
            auto_fixable: true,
        }
    }
}

/// Result of a conformance check — either OK or a list of violations.
#[derive(Debug, Clone)]
pub struct ConformanceReport {
    pub standard: PdfXStandard,
    pub violations: Vec<ConformanceViolation>,
}

impl ConformanceReport {
    pub fn is_conformant(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn into_result(self) -> Result<(), PdfError> {
        if self.is_conformant() {
            Ok(())
        } else {
            let msg = self
                .violations
                .iter()
                .map(|v| format!("[{}] {}", v.rule, v.message))
                .collect::<Vec<_>>()
                .join("; ");
            Err(PdfError::Conformance(msg))
        }
    }
}

/// Validate a document against the given export settings.
///
/// Returns a `ConformanceReport` describing all violations found.
/// Call `report.into_result()` to convert to a `Result<(), PdfError>`.
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

/// Validate a text document (blocks + styles + metadata) against PDF/X settings.
///
/// Returns a flat list of violations. Hard violations (not auto-fixable) block
/// export; warnings can be reported to the user but do not block export.
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

// ---------------------------------------------------------------------------
// Text document check functions
// ---------------------------------------------------------------------------

/// At least one block must exist with non-empty content.
fn check_text_not_empty(blocks: &[Block], violations: &mut Vec<ConformanceViolation>) {
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

/// Document title is recommended for PDF/X.
fn check_text_title(metadata: &Metadata, violations: &mut Vec<ConformanceViolation>) {
    let has_title = metadata
        .title
        .as_deref()
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false);
    if !has_title {
        violations.push(ConformanceViolation {
            rule: "X/text-missing-title".to_string(),
            message: "Document has no title; PDF/X metadata will be incomplete".to_string(),
            auto_fixable: true, // warning — does not block export
        });
    }
}

/// Validate colour usage in style definitions.
#[cfg(feature = "colour-management")]
fn check_text_colours_in_styles(
    styles: &HashMap<String, StyleDefinition>,
    standard: PdfXStandard,
    violations: &mut Vec<ConformanceViolation>,
) {
    // Removed unused Colour import
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
fn check_text_colour(
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
            // Recursively validate the fallback.
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

// ---------------------------------------------------------------------------
// Individual check functions
// ---------------------------------------------------------------------------

/// PDF/X requires a non-empty OutputConditionIdentifier.
fn check_output_condition(
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

/// Check that all colours are compatible with the target standard.
fn check_colour_spaces(
    document: &VectorDocument,
    settings: &PdfExportSettings,
    violations: &mut Vec<ConformanceViolation>,
) {
    let standard = settings.standard;
    // For X-1a: document working space will be converted to CMYK by the
    // pre-export pass — report as auto_fixable rather than a hard error.
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

/// Verify a single colour is compatible with the standard.
fn check_colour_compatibility(
    colour: &Colour,
    standard: PdfXStandard,
    location: &str,
    violations: &mut Vec<ConformanceViolation>,
) {
    match colour {
        Colour::Rgb { .. } => {
            if !standard.allows_rgb() {
                // The pre-export pass converts RGB → CMYK automatically.
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
            // Lab is device-independent; allowed in X-4 but not X-1a.
            // The pre-export pass converts Lab → CMYK automatically.
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
        Colour::Cmyk { .. } => {} // always allowed
        Colour::Spot { cmyk_fallback, .. } => {
            // Recursively check the fallback.
            check_colour_compatibility(cmyk_fallback, standard, location, violations);
        }
        Colour::Linked { id } => {
            // Linked colours without resolution are a validator error.
            violations.push(ConformanceViolation::new(
                "X/unresolved-linked-colour",
                format!("Unresolved Linked colour id={} at {}", id, location),
            ));
        }
    }
}

/// PDF/X-1a forbids any transparency (opacity < 1.0).
///
/// For X-1a, transparent objects will be rasterised automatically at the
/// configured resolution — violations are reported as auto-fixable.
fn check_transparency(
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

/// Linked colours cannot be exported (they must be resolved first).
fn check_linked_colours(document: &VectorDocument, violations: &mut Vec<ConformanceViolation>) {
    // Already handled inside check_colour_compatibility; this is a belt-and-
    // suspenders pass that catches Linked colours regardless of standard.
    for_each_colour(document, |colour, location| {
        if let Colour::Linked { id } = colour {
            // Avoid duplicate violations if already reported above.
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

/// Bleed must be non-negative.
fn check_bleed(settings: &PdfExportSettings, violations: &mut Vec<ConformanceViolation>) {
    if settings.bleed_pt < 0.0 {
        violations.push(ConformanceViolation::new(
            "X/negative-bleed",
            format!("Bleed must be ≥ 0 pt; got {}", settings.bleed_pt),
        ));
    }
}

/// A document with no layers or all layers empty produces an empty PDF —
/// warn about it (not a hard conformance violation, but practically useless).
fn check_document_not_empty(document: &VectorDocument, violations: &mut Vec<ConformanceViolation>) {
    let has_objects = document.layers.iter().any(|l| !l.objects.is_empty());
    if !has_objects {
        violations.push(ConformanceViolation::new(
            "X/empty-document",
            "Document has no renderable objects; the exported PDF will be blank",
        ));
    }
}
