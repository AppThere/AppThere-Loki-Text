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

use tauri::{AppHandle, Runtime};
use vector_core::canvas::Canvas;
use vector_core::convert::ConversionWarning;
use vector_core::document::VectorDocument;
use vector_core::{svg_parser, svg_writer};

/// Open a vector document from a file path or raw bytes.
#[tauri::command]
pub async fn open_vector_document<R: Runtime>(
    _app: AppHandle<R>,
    path: String,
    file_content: Option<Vec<u8>>,
) -> Result<VectorDocument, String> {
    let bytes = if let Some(content) = file_content {
        content
    } else {
        std::fs::read(&path).map_err(|e| format!("Failed to read file '{path}': {e}"))?
    };

    let svg_str =
        std::str::from_utf8(&bytes).map_err(|e| format!("File is not valid UTF-8: {e}"))?;

    svg_parser::parse(svg_str).map_err(|e| format!("SVG parse failed: {e}"))
}

/// Save a vector document to a file path.
#[tauri::command]
pub async fn save_vector_document<R: Runtime>(
    _app: AppHandle<R>,
    path: String,
    document: VectorDocument,
) -> Result<(), String> {
    use common_core::colour_management::create_display_context;

    let mut ctx = create_display_context(&document.colour_settings)?;
    let svg = svg_writer::write(&document, &mut ctx)?;
    std::fs::write(&path, svg.as_bytes()).map_err(|e| format!("Failed to write file '{path}': {e}"))
}

/// Create a new vector document from a preset.
#[tauri::command]
pub fn new_vector_document(
    preset: String,
    width_px: Option<f64>,
    height_px: Option<f64>,
) -> VectorDocument {
    match preset.as_str() {
        "a4-portrait" => VectorDocument::blank_a4(),
        "a4-portrait-cmyk" => VectorDocument::blank_a4_cmyk(),
        "a4-landscape" => VectorDocument::new(Canvas::a4_landscape()),
        "letter-portrait" => VectorDocument::blank_letter(),
        "letter-portrait-cmyk" => VectorDocument::blank_letter_cmyk(),
        "custom" => {
            let w = width_px.unwrap_or(800.0);
            let h = height_px.unwrap_or(600.0);
            VectorDocument::new(Canvas::new(w, h))
        }
        _ => VectorDocument::blank_a4(),
    }
}

/// Serialise a vector document to SVG bytes (for session autosave).
#[tauri::command]
pub fn serialize_vector_document(document: VectorDocument) -> Result<Vec<u8>, String> {
    use common_core::colour_management::create_display_context;

    let mut ctx = create_display_context(&document.colour_settings)?;
    let svg = svg_writer::write(&document, &mut ctx)?;
    Ok(svg.into_bytes())
}

/// Deserialise SVG bytes back to a vector document.
#[tauri::command]
pub fn deserialize_vector_document(file_content: Vec<u8>) -> Result<VectorDocument, String> {
    let svg_str = std::str::from_utf8(&file_content)
        .map_err(|e| format!("Content is not valid UTF-8: {e}"))?;
    svg_parser::parse(svg_str).map_err(|e| format!("SVG parse failed: {e}"))
}

/// Convert a batch of Colour values to display sRGB.
/// Used by the frontend renderer to avoid per-object IPC calls.
#[tauri::command]
pub fn batch_convert_colours(
    colours: Vec<common_core::colour_management::Colour>,
    settings: common_core::colour_management::DocumentColourSettings,
) -> Result<Vec<[f32; 4]>, String> {
    use common_core::colour_management::{ColourContext, IccProfileStore};

    let mut store = IccProfileStore::new();
    let mut ctx = ColourContext::new_for_display(&settings, &mut store)?;
    Ok(ctx.convert_batch(&colours))
}

/// Convert a vector document's colour mode to a new colour space.
///
/// All object colours are converted from the document's current working space
/// to display sRGB, then the target `DocumentColourSettings` is applied.
/// Returns the converted document and any warnings for colours that could not
/// be precisely converted (e.g. Linked colours).
#[tauri::command]
pub fn convert_document_colour_mode(
    document: VectorDocument,
    target_settings: common_core::colour_management::DocumentColourSettings,
) -> Result<(VectorDocument, Vec<ConversionWarning>), String> {
    vector_core::convert::convert_document_colour_mode(&document, target_settings)
}

/// Return all built-in ICC output intent profiles available for export.
///
/// Each entry contains a machine-readable `id` and human-readable `name` and
/// `description` suitable for display in a profile picker UI.
#[tauri::command]
pub fn get_output_intent_profiles() -> Vec<serde_json::Value> {
    use common_core::colour_management::BuiltInProfile;
    use serde_json::json;

    [
        BuiltInProfile::SrgbIec61966,
        BuiltInProfile::IsoCoatedV2,
        BuiltInProfile::SwopV2,
        BuiltInProfile::GraCol2006,
    ]
    .iter()
    .map(|p| {
        json!({
            "id": format!("{:?}", p),
            "name": p.display_name(),
            "description": p.description(),
        })
    })
    .collect()
}

/// Preview what a document's colours will look like after colour mode conversion.
///
/// Returns an array of `[r, g, b, a]` display-sRGB values for the first
/// `max_colours` unique colours found in the document, converted using
/// `target_settings`. This allows the UI to render a before/after preview
/// without committing to the conversion.
#[tauri::command]
pub fn preview_colour_conversion(
    document: VectorDocument,
    target_settings: common_core::colour_management::DocumentColourSettings,
    max_colours: Option<usize>,
) -> Result<Vec<[f32; 4]>, String> {
    use common_core::colour_management::{ColourContext, IccProfileStore};
    use vector_core::style::Paint;

    let limit = max_colours.unwrap_or(64);

    // Collect unique colours from the document
    let mut colours: Vec<common_core::colour_management::Colour> = Vec::new();
    for layer in &document.layers {
        for obj in &layer.objects {
            let style = &obj.common().style;
            if let Paint::Solid { colour } = &style.fill {
                if !colours.contains(colour) && colours.len() < limit {
                    colours.push(colour.clone());
                }
            }
            if let Paint::Solid { colour } = &style.stroke.paint {
                if !colours.contains(colour) && colours.len() < limit {
                    colours.push(colour.clone());
                }
            }
        }
    }

    let mut store = IccProfileStore::new();
    let mut ctx = ColourContext::new_for_display(&target_settings, &mut store)?;
    Ok(ctx.convert_batch(&colours))
}

/// Search the Pantone colour library by name.
///
/// Performs a case-insensitive substring search over all known Pantone names.
/// Returns up to 50 matches, each with the colour name and its CIE Lab reference.
#[tauri::command]
pub fn search_pantone(
    query: String,
) -> Vec<serde_json::Value> {
    use common_core::colour_management::pantone::all_pantone_names;
    use serde_json::json;

    let q = query.to_uppercase();
    all_pantone_names()
        .filter(|name| name.contains(q.as_str()))
        .take(50)
        .filter_map(|name| {
            common_core::colour_management::lookup_pantone(name).map(|lab| {
                json!({
                    "name": name,
                    "lab_ref": [lab[0], lab[1], lab[2]],
                })
            })
        })
        .collect()
}
