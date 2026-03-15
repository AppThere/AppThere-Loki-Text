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

    let svg_str = std::str::from_utf8(&bytes)
        .map_err(|e| format!("File is not valid UTF-8: {e}"))?;

    svg_parser::parse(svg_str)
        .map_err(|e| format!("SVG parse failed: {e}"))
}

/// Save a vector document to a file path.
#[tauri::command]
pub async fn save_vector_document<R: Runtime>(
    _app: AppHandle<R>,
    path: String,
    document: VectorDocument,
) -> Result<(), String> {
    let svg = svg_writer::write(&document);
    std::fs::write(&path, svg.as_bytes())
        .map_err(|e| format!("Failed to write file '{path}': {e}"))
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
        "a4-landscape" => VectorDocument::new(Canvas::a4_landscape()),
        "letter-portrait" => VectorDocument::blank_letter(),
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
    Ok(svg_writer::write(&document).into_bytes())
}

/// Deserialise SVG bytes back to a vector document.
#[tauri::command]
pub fn deserialize_vector_document(file_content: Vec<u8>) -> Result<VectorDocument, String> {
    let svg_str = std::str::from_utf8(&file_content)
        .map_err(|e| format!("Content is not valid UTF-8: {e}"))?;
    svg_parser::parse(svg_str)
        .map_err(|e| format!("SVG parse failed: {e}"))
}
