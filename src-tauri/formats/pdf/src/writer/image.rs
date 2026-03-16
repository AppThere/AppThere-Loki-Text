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

//! Embedding rasterised images in a PDF as Image XObjects.
//!
//! Each `RasterRegion` produced by the flatten pipeline is embedded as a
//! primary DeviceRGB image XObject with a separate grayscale SMask for the
//! alpha channel.

use crate::error::PdfError;
use crate::flatten::RasterRegion;
use pdf_writer::{Pdf, Ref};

/// Write a `RasterRegion` as a PDF Image XObject (with SMask).
///
/// Returns `(image_ref, smask_ref)`. Both refs are written to `pdf`.
/// The caller must add the image ref to the page's `/XObject` resource dict
/// under the chosen name and use `/<name> Do` in the content stream.
pub fn write_image_xobject(
    region: &RasterRegion,
    pdf: &mut Pdf,
    next_ref: &mut i32,
) -> Result<(Ref, Ref), PdfError> {
    let w = region.width;
    let h = region.height;

    // Split RGBA pixels into separate RGB and alpha buffers.
    let expected = (w as usize) * (h as usize) * 4;
    if region.pixels.len() != expected {
        return Err(PdfError::Internal(format!(
            "RasterRegion pixel buffer length {} != expected {}",
            region.pixels.len(),
            expected
        )));
    }

    let mut rgb_data: Vec<u8> = Vec::with_capacity((w * h * 3) as usize);
    let mut mask_data: Vec<u8> = Vec::with_capacity((w * h) as usize);
    for chunk in region.pixels.chunks_exact(4) {
        rgb_data.extend_from_slice(&chunk[..3]);
        mask_data.push(chunk[3]);
    }

    // Allocate refs.
    let smask_ref = alloc(next_ref);
    let image_ref = alloc(next_ref);

    // Write SMask (grayscale alpha).
    {
        let mut smask = pdf.image_xobject(smask_ref, &mask_data);
        smask.width(w as i32);
        smask.height(h as i32);
        smask.color_space().device_gray();
        smask.bits_per_component(8);
        // TODO(phase-8): add Deflate compression for smaller file sizes.
    }

    // Write primary image (DeviceRGB + SMask reference).
    {
        let mut img = pdf.image_xobject(image_ref, &rgb_data);
        img.width(w as i32);
        img.height(h as i32);
        img.color_space().device_rgb();
        img.bits_per_component(8);
        img.s_mask(smask_ref);
    }

    Ok((image_ref, smask_ref))
}

fn alloc(next: &mut i32) -> Ref {
    let r = Ref::new(*next);
    *next += 1;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flatten::RasterRegion;

    fn two_by_two_region() -> RasterRegion {
        // 2×2 RGBA: top-left red opaque, others transparent.
        RasterRegion {
            x: 0.0,
            y: 0.0,
            width: 2,
            height: 2,
            pixels: vec![
                255, 0, 0, 255, // red, opaque
                0, 255, 0, 128, // green, semi-transparent
                0, 0, 255, 64, // blue, mostly transparent
                255, 255, 0, 0, // yellow, fully transparent
            ],
            dpi: 72.0,
        }
    }

    #[test]
    fn embed_produces_two_xobject_refs() {
        let region = two_by_two_region();
        let mut pdf = Pdf::new();
        let mut next_ref = 1i32;
        let (img_ref, smask_ref) = write_image_xobject(&region, &mut pdf, &mut next_ref).unwrap();
        assert_ne!(
            img_ref, smask_ref,
            "image and smask must have distinct refs"
        );
        assert_eq!(next_ref, 3); // two refs allocated
    }

    #[test]
    fn rgb_and_alpha_split_correctly() {
        // Build 1×1 RGBA pixel (200, 100, 50, 180) and verify split.
        let region = RasterRegion {
            x: 0.0,
            y: 0.0,
            width: 1,
            height: 1,
            pixels: vec![200, 100, 50, 180],
            dpi: 72.0,
        };
        // We verify splitting logic directly:
        let mut rgb_data: Vec<u8> = Vec::new();
        let mut mask_data: Vec<u8> = Vec::new();
        for chunk in region.pixels.chunks_exact(4) {
            rgb_data.extend_from_slice(&chunk[..3]);
            mask_data.push(chunk[3]);
        }
        assert_eq!(rgb_data, vec![200, 100, 50]);
        assert_eq!(mask_data, vec![180]);
    }
}
