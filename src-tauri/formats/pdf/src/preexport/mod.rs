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

//! Pre-export document preparation.
//!
//! These functions prepare a *cloned* `VectorDocument` for PDF/X export by:
//! 1. Expanding `Colour::Linked` references from the swatch library.
//! 2. Converting non-CMYK colours to CMYK (PDF/X-1a only).
//!
//! The original document is never modified.

pub(crate) mod cmyk;

use crate::error::PdfError;
use crate::export_settings::PdfExportSettings;
use common_core::colour_management::{Colour, SwatchId};
use vector_core::document::VectorDocument;
use vector_core::object::VectorObject;
use vector_core::style::Paint;

/// Prepare a clone of `doc` for PDF/X export.
///
/// For PDF/X-4, only linked colours are expanded.
/// For PDF/X-1a, colours are also converted to CMYK.
pub fn prepare_for_export(
    doc: &VectorDocument,
    settings: &PdfExportSettings,
) -> Result<VectorDocument, PdfError> {
    let mut out = doc.clone();
    expand_linked_colours(&mut out)?;
    if settings.standard.requires_cmyk_only() {
        cmyk::convert_to_cmyk(&mut out)?;
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Stage 1 — expand Colour::Linked references
// ---------------------------------------------------------------------------

fn expand_linked_colours(doc: &mut VectorDocument) -> Result<(), PdfError> {
    let lib = doc.swatch_library.clone(); // clone to avoid simultaneous borrow
    for layer in &mut doc.layers {
        for obj in &mut layer.objects {
            expand_object(obj, &lib)?;
        }
    }
    Ok(())
}

fn expand_object(
    obj: &mut VectorObject,
    lib: &common_core::colour_management::SwatchLibrary,
) -> Result<(), PdfError> {
    {
        let style = &mut obj.common_mut().style;
        if let Paint::Solid { colour } = &mut style.fill {
            resolve_linked(colour, lib)?;
        }
        if let Paint::Solid { colour } = &mut style.stroke.paint {
            resolve_linked(colour, lib)?;
        }
    }
    if let VectorObject::Group(g) = obj {
        for child in &mut g.children {
            expand_object(child, lib)?;
        }
    }
    Ok(())
}

fn resolve_linked(
    colour: &mut Colour,
    lib: &common_core::colour_management::SwatchLibrary,
) -> Result<(), PdfError> {
    if let Colour::Linked { id } = colour {
        let swatch_id = SwatchId(id.clone());
        match lib.get(&swatch_id) {
            Some(s) => *colour = s.colour.clone(),
            None => {
                return Err(PdfError::ColourProfile(format!(
                    "Swatch '{}' not found in library",
                    id
                )))
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use common_core::colour_management::{
        BuiltInProfile, ColourSpace, ColourSwatch, DocumentColourSettings, IccProfileRef,
        SwatchId, SwatchLibrary,
    };
    use vector_core::canvas::Canvas;
    use vector_core::object::{CommonProps, ObjectId, RectObject, VectorObject};
    use vector_core::style::{ObjectStyle, Paint, StrokeStyle};
    use vector_core::transform::Transform as VTransform;

    fn cmyk_settings() -> DocumentColourSettings {
        DocumentColourSettings {
            working_space: ColourSpace::Cmyk {
                profile: IccProfileRef::BuiltIn(BuiltInProfile::IsoCoatedV2),
            },
            ..Default::default()
        }
    }

    fn x1a() -> PdfExportSettings {
        PdfExportSettings {
            standard: crate::export_settings::PdfXStandard::X1a2001,
            output_condition_identifier: "FOGRA39".into(),
            output_condition: "ISO Coated v2".into(),
            registry_name: "http://www.color.org".into(),
            bleed_pt: 0.0,
            resolution_dpi: 300,
        }
    }

    fn x4() -> PdfExportSettings {
        PdfExportSettings::default()
    }

    fn doc_with_fill(colour: Colour) -> VectorDocument {
        let mut doc = VectorDocument::new_with_settings(Canvas::a4_portrait(), cmyk_settings());
        doc.layers[0].objects.push(VectorObject::Rect(RectObject {
            common: CommonProps {
                id: ObjectId("r1".into()),
                label: None,
                style: ObjectStyle {
                    fill: Paint::Solid { colour },
                    stroke: StrokeStyle::none(),
                    opacity: 1.0,
                    fill_opacity: 1.0,
                    stroke_opacity: 1.0,
                },
                transform: VTransform::identity(),
                visible: true,
                locked: false,
            },
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            rx: 0.0,
            ry: 0.0,
        }));
        doc
    }

    #[test]
    fn expand_linked_colour_resolves_to_swatch_colour() {
        let target = Colour::Cmyk { c: 0.1, m: 0.2, y: 0.3, k: 0.0, alpha: 1.0 };
        let mut lib = SwatchLibrary::new();
        let sid = SwatchId("sw-1".into());
        lib.add(ColourSwatch {
            id: sid.clone(),
            name: "Test".into(),
            colour: target.clone(),
            is_spot: false,
        });
        let mut doc = doc_with_fill(Colour::Linked { id: "sw-1".into() });
        doc.swatch_library = lib;
        let prepared = prepare_for_export(&doc, &x4()).unwrap();
        let obj = &prepared.layers[0].objects[0];
        let style = &obj.common().style;
        match &style.fill {
            Paint::Solid { colour } => {
                assert_eq!(format!("{:?}", colour), format!("{:?}", target))
            }
            other => panic!("expected Solid fill, got {:?}", other),
        }
    }

    #[test]
    fn missing_swatch_returns_error() {
        let doc = doc_with_fill(Colour::Linked { id: "nonexistent".into() });
        assert!(prepare_for_export(&doc, &x4()).is_err());
    }

    #[test]
    fn rgb_to_cmyk_conversion_for_x1a() {
        let doc = doc_with_fill(Colour::Rgb { r: 1.0, g: 0.0, b: 0.0, a: 1.0 });
        let prepared = prepare_for_export(&doc, &x1a()).unwrap();
        let obj = &prepared.layers[0].objects[0];
        if let Paint::Solid { colour } = &obj.common().style.fill {
            assert!(
                matches!(colour, Colour::Cmyk { .. }),
                "Expected CMYK after X-1a conversion, got {:?}",
                colour
            );
        } else {
            panic!("Expected Solid fill");
        }
    }

    #[test]
    fn x4_preparation_does_not_convert_rgb_colours() {
        let doc = doc_with_fill(Colour::Rgb { r: 0.5, g: 0.5, b: 0.5, a: 1.0 });
        let prepared = prepare_for_export(&doc, &x4()).unwrap();
        let obj = &prepared.layers[0].objects[0];
        assert!(
            matches!(obj.common().style.fill, Paint::Solid { colour: Colour::Rgb { .. } }),
            "X-4 should not convert RGB colours"
        );
    }

    #[test]
    fn original_document_is_not_mutated() {
        let original = doc_with_fill(Colour::Rgb { r: 0.5, g: 0.5, b: 0.5, a: 1.0 });
        let _ = prepare_for_export(&original, &x1a()).unwrap();
        let obj = &original.layers[0].objects[0];
        assert!(
            matches!(obj.common().style.fill, Paint::Solid { colour: Colour::Rgb { .. } }),
            "Original document must not be mutated"
        );
    }
}
