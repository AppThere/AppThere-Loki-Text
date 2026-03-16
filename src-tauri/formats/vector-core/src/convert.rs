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

//! Colour mode conversion for vector documents.
//!
//! Converts all object colours in a [`VectorDocument`] from one colour space
//! to another via display sRGB as an intermediate representation.
//!
//! Phase 2 converts to sRGB only. A future phase will support direct CMYK
//! output by building a target-space transform from source to CMYK.

use crate::document::VectorDocument;
use crate::object::VectorObject;
use crate::style::Paint;
use common_core::colour_management::{
    Colour, ColourContext, DocumentColourSettings, IccProfileStore,
};
use serde::Serialize;

/// Describes a colour that could not be precisely converted during a colour
/// mode conversion operation.
#[derive(Debug, Clone, Serialize)]
pub struct ConversionWarning {
    /// ID of the object containing the unconvertable colour.
    pub object_id: String,
    /// `"fill"` or `"stroke"`.
    pub property: String,
    /// Human-readable description of why conversion was limited.
    pub message: String,
}

/// Convert all colours in `doc` from its current working space to `target`.
///
/// In Phase 2, conversion goes via display sRGB: every colour is converted to
/// `Colour::Rgb` using the source [`ColourContext`]. The target
/// `DocumentColourSettings` is applied to the returned document so the
/// renderer will use the correct output profile from then on.
///
/// [`Colour::Linked`] colours cannot be resolved without the swatch library
/// and are left unchanged with a warning.
///
/// # Errors
///
/// Returns an error string if a [`ColourContext`] cannot be created for the
/// source document (e.g. the referenced ICC profile cannot be loaded).
pub fn convert_document_colour_mode(
    doc: &VectorDocument,
    target: DocumentColourSettings,
) -> Result<(VectorDocument, Vec<ConversionWarning>), String> {
    let mut store = IccProfileStore::new();
    let mut src_ctx = ColourContext::new_for_display(&doc.colour_settings, &mut store)?;

    let mut warnings: Vec<ConversionWarning> = Vec::new();
    let mut new_doc = doc.clone();
    new_doc.colour_settings = target.clone();

    for layer in &mut new_doc.layers {
        for obj in &mut layer.objects {
            convert_object(obj, &mut src_ctx, &target, &mut warnings);
        }
    }

    Ok((new_doc, warnings))
}

fn convert_object(
    obj: &mut VectorObject,
    ctx: &mut ColourContext,
    target: &DocumentColourSettings,
    warnings: &mut Vec<ConversionWarning>,
) {
    let id = obj.common().id.0.clone();
    let common = obj.common_mut();
    convert_paint(&mut common.style.fill, &id, "fill", ctx, target, warnings);
    convert_paint(
        &mut common.style.stroke.paint,
        &id,
        "stroke",
        ctx,
        target,
        warnings,
    );

    if let VectorObject::Group(g) = obj {
        for child in &mut g.children {
            convert_object(child, ctx, target, warnings);
        }
    }
}

fn convert_paint(
    paint: &mut Paint,
    id: &str,
    property: &str,
    ctx: &mut ColourContext,
    _target: &DocumentColourSettings,
    warnings: &mut Vec<ConversionWarning>,
) {
    if let Paint::Solid { colour } = paint {
        *colour = colour_to_display_rgb(colour, id, property, ctx, warnings);
    }
}

/// Convert a [`Colour`] to `Colour::Rgb` via the source display context.
///
/// All colour variants except [`Colour::Linked`] are converted. Linked
/// colours require a swatch library lookup and produce a warning instead.
fn colour_to_display_rgb(
    colour: &Colour,
    id: &str,
    property: &str,
    ctx: &mut ColourContext,
    warnings: &mut Vec<ConversionWarning>,
) -> Colour {
    if let Colour::Linked { .. } = colour {
        warnings.push(ConversionWarning {
            object_id: id.to_string(),
            property: property.to_string(),
            message: "Linked colour cannot be converted inline; resolve swatch first.".to_string(),
        });
        return colour.clone();
    }

    let [r, g, b, a] = ctx.convert(colour);
    Colour::Rgb { r, g, b, a }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;
    use crate::document::VectorDocument;
    use crate::layer::Layer;
    use crate::object::{CommonProps, RectObject, VectorObject};
    use crate::style::{ObjectStyle, Paint};
    use common_core::colour_management::Colour;

    fn make_doc_with_colour(colour: Colour) -> VectorDocument {
        let mut layer = Layer::new("Layer 1");
        let mut common = CommonProps::new("rect1");
        common.style = ObjectStyle {
            fill: Paint::Solid { colour },
            ..ObjectStyle::default_fill()
        };
        layer.objects.push(VectorObject::Rect(RectObject {
            common,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            rx: 0.0,
            ry: 0.0,
        }));
        VectorDocument {
            canvas: Canvas::new(400.0, 300.0),
            layers: vec![layer],
            metadata: common_core::Metadata::default(),
            colour_settings: DocumentColourSettings::default(),
            swatch_library: common_core::colour_management::SwatchLibrary::new(),
        }
    }

    #[test]
    fn convert_rgb_to_srgb_is_identity() {
        let red = Colour::from_u8_rgb(255, 0, 0);
        let doc = make_doc_with_colour(red.clone());
        let (new_doc, warnings) =
            convert_document_colour_mode(&doc, DocumentColourSettings::default()).unwrap();
        assert!(warnings.is_empty());
        if let VectorObject::Rect(r) = &new_doc.layers[0].objects[0] {
            if let Paint::Solid { colour } = &r.common.style.fill {
                if let Colour::Rgb { r, g, b, a } = colour {
                    assert!((*r - 1.0).abs() < 0.01);
                    assert!((*g).abs() < 0.01);
                    assert!((*b).abs() < 0.01);
                    assert!((*a - 1.0).abs() < 0.01);
                } else {
                    panic!("Expected Rgb colour");
                }
            }
        }
    }

    #[test]
    fn convert_linked_colour_produces_warning() {
        let linked = Colour::Linked {
            id: "swatch-001".to_string(),
        };
        let doc = make_doc_with_colour(linked);
        let (_, warnings) =
            convert_document_colour_mode(&doc, DocumentColourSettings::default()).unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].object_id, "rect1");
        assert_eq!(warnings[0].property, "fill");
    }
}
