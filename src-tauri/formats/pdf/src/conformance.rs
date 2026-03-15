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

use crate::error::PdfError;
use crate::export_settings::{PdfExportSettings, PdfXStandard};
use common_core::colour_management::{Colour, ColourSpace};
use vector_core::document::VectorDocument;
use vector_core::object::VectorObject;
use vector_core::style::Paint;

/// A single conformance violation.
#[derive(Debug, Clone, PartialEq)]
pub struct ConformanceViolation {
    /// Short rule identifier, e.g. "X1a/no-transparency".
    pub rule: String,
    /// Human-readable description of the violation.
    pub message: String,
}

impl ConformanceViolation {
    fn new(rule: impl Into<String>, message: impl Into<String>) -> Self {
        ConformanceViolation {
            rule: rule.into(),
            message: message.into(),
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
    check_colour_spaces(document, standard, &mut violations);
    check_transparency(document, standard, &mut violations);
    check_linked_colours(document, &mut violations);
    check_bleed(settings, &mut violations);
    check_document_not_empty(document, &mut violations);

    ConformanceReport {
        standard,
        violations,
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
    standard: PdfXStandard,
    violations: &mut Vec<ConformanceViolation>,
) {
    // For X-1a: document working space must be CMYK.
    if standard.requires_cmyk_only() {
        match &document.colour_settings.working_space {
            ColourSpace::Cmyk { .. } => {}
            other => {
                violations.push(ConformanceViolation::new(
                    "X1a/working-space-must-be-cmyk",
                    format!("PDF/X-1a requires CMYK working space; found {:?}", other),
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
                violations.push(ConformanceViolation::new(
                    "X1a/no-rgb",
                    format!(
                        "PDF/X-1a does not allow RGB colours (found at {})",
                        location
                    ),
                ));
            }
        }
        Colour::Lab { .. } => {
            // Lab is device-independent; allowed in X-4 but not X-1a.
            if standard.requires_cmyk_only() {
                violations.push(ConformanceViolation::new(
                    "X1a/no-lab",
                    format!(
                        "PDF/X-1a does not allow Lab colours (found at {})",
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
fn check_transparency(
    document: &VectorDocument,
    standard: PdfXStandard,
    violations: &mut Vec<ConformanceViolation>,
) {
    if standard.allows_transparency() {
        return;
    }

    for_each_object(document, |obj, location| {
        let style = &obj.common().style;
        if style.opacity < 1.0 - f64::EPSILON {
            violations.push(ConformanceViolation::new(
                "X1a/no-transparency",
                format!(
                    "PDF/X-1a forbids transparency; object at {} has opacity {}",
                    location, style.opacity
                ),
            ));
        }
        if style.fill_opacity < 1.0 - f64::EPSILON {
            violations.push(ConformanceViolation::new(
                "X1a/no-fill-opacity",
                format!(
                    "PDF/X-1a forbids transparency; object at {} has fill-opacity {}",
                    location, style.fill_opacity
                ),
            ));
        }
        if style.stroke_opacity < 1.0 - f64::EPSILON {
            violations.push(ConformanceViolation::new(
                "X1a/no-stroke-opacity",
                format!(
                    "PDF/X-1a forbids transparency; object at {} has stroke-opacity {}",
                    location, style.stroke_opacity
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

// ---------------------------------------------------------------------------
// Helpers for iterating objects and colours
// ---------------------------------------------------------------------------

fn for_each_object<F>(document: &VectorDocument, mut f: F)
where
    F: FnMut(&VectorObject, &str),
{
    for layer in &document.layers {
        for obj in &layer.objects {
            let loc = format!("layer '{}' / object '{}'", layer.name, obj.id().0);
            visit_object(obj, &loc, &mut f);
        }
    }
}

fn visit_object<F>(obj: &VectorObject, location: &str, f: &mut F)
where
    F: FnMut(&VectorObject, &str),
{
    f(obj, location);
    if let VectorObject::Group(g) = obj {
        for child in &g.children {
            let loc = format!("{} / '{}'", location, child.id().0);
            visit_object(child, &loc, f);
        }
    }
}

fn for_each_colour<F>(document: &VectorDocument, mut f: F)
where
    F: FnMut(&Colour, &str),
{
    for_each_object(document, |obj, location| {
        let style = &obj.common().style;
        if let Paint::Solid { colour } = &style.fill {
            f(colour, location);
        }
        if let Paint::Solid { colour } = &style.stroke.paint {
            f(colour, location);
        }
    });
}
