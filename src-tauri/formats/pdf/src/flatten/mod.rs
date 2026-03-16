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

//! Transparency flattening for PDF/X-1a output.
//!
//! Groups of overlapping objects where any member has opacity < 1.0
//! are rasterised to bitmaps using resvg. The bitmaps are embedded in
//! the PDF as Image XObjects. Vector objects that do not overlap any
//! transparent object are written as normal vector content.
//!
//! This is a one-way transformation applied to a clone of the document
//! — the original is never modified.

mod groups;
mod raster;

use crate::error::PdfError;
use vector_core::document::VectorDocument;
use vector_core::object::VectorObject;

pub use raster::BoundingBox;

/// A flattened representation of a layer's content.
/// Transparent regions have been rasterised; the rest remains as vector objects.
pub struct FlattenedLayer {
    pub name: String,
    pub visible: bool,
    pub items: Vec<FlattenedItem>,
}

pub enum FlattenedItem {
    /// A vector object that needs no flattening (opaque, no blend mode).
    Vector(VectorObject),
    /// A rasterised region replacing one or more transparent objects.
    Raster(RasterRegion),
}

/// A rasterised bitmap region ready for embedding as a PDF Image XObject.
pub struct RasterRegion {
    /// X position in document coordinates (pixels).
    pub x: f64,
    /// Y position in document coordinates (pixels).
    pub y: f64,
    /// Width in output pixels.
    pub width: u32,
    /// Height in output pixels.
    pub height: u32,
    /// RGBA pixel data (row-major, 4 bytes per pixel).
    pub pixels: Vec<u8>,
    /// The DPI at which this region was rasterised.
    pub dpi: f64,
}

/// Flatten the transparency in a document, producing a per-layer representation.
///
/// For PDF/X-1a, transparent object groups are rasterised at `resolution_dpi`.
/// The original document is never modified.
///
/// # MVP simplification
/// If any object in a layer is transparent, the **entire layer** is rasterised
/// as a single `RasterRegion`. Future phases can implement proper overlap-based
/// grouping.
pub fn flatten_document(
    doc: &VectorDocument,
    resolution_dpi: f64,
) -> Result<Vec<FlattenedLayer>, PdfError> {
    let canvas_bbox = BoundingBox {
        x: 0.0,
        y: 0.0,
        width: doc.canvas.width,
        height: doc.canvas.height,
    };

    let mut result = Vec::new();
    for layer in &doc.layers {
        let items = if groups::layer_has_transparency(&layer.objects) {
            // TODO(phase-6): implement proper overlap-based grouping so only
            // the overlapping transparent region is rasterised, not the full layer.
            let region = raster::rasterise_objects(
                &layer.objects,
                doc.canvas.dpi,
                resolution_dpi,
                canvas_bbox,
            )?;
            vec![FlattenedItem::Raster(region)]
        } else {
            layer
                .objects
                .iter()
                .cloned()
                .map(FlattenedItem::Vector)
                .collect()
        };
        result.push(FlattenedLayer {
            name: layer.name.clone(),
            visible: layer.visible,
            items,
        });
    }
    Ok(result)
}
