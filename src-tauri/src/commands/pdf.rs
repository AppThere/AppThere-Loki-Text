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

use loki_pdf::conformance::validate;
use loki_pdf::export_settings::PdfExportSettings;
use loki_pdf::write_pdf_x;
use vector_core::document::VectorDocument;

/// Validate a vector document against a PDF/X standard.
///
/// Returns a list of conformance violations. An empty list means the document
/// is conformant. Each violation has a `rule` and a `message`.
#[tauri::command]
pub fn validate_pdf_x_conformance(
    document: VectorDocument,
    settings: PdfExportSettings,
) -> Vec<serde_json::Value> {
    use serde_json::json;
    let report = validate(&document, &settings);
    report
        .violations
        .iter()
        .map(|v| {
            json!({
                "rule": v.rule,
                "message": v.message,
            })
        })
        .collect()
}

/// Export a vector document to PDF/X bytes and write them to `path`.
///
/// The document is validated before any bytes are written. Returns an error
/// string if validation fails or if the file cannot be written.
#[tauri::command]
pub fn export_pdf_x(
    document: VectorDocument,
    settings: PdfExportSettings,
    path: String,
) -> Result<(), String> {
    let bytes = write_pdf_x(&document, &settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write PDF to '{path}': {e}"))
}
