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

//! Structured error types for the loki-pdf crate.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for PDF operations.
pub type PdfResult<T> = Result<T, PdfError>;

/// Errors that can occur during PDF/X validation or export.
#[derive(Debug, Error, Serialize, Deserialize, Clone, PartialEq)]
pub enum PdfError {
    /// The document does not conform to the requested PDF/X standard.
    #[error("PDF/X conformance error: {0}")]
    Conformance(String),

    /// An I/O error occurred writing the PDF.
    #[error("I/O error: {0}")]
    Io(String),

    /// A colour profile could not be loaded or embedded.
    #[error("Colour profile error: {0}")]
    ColourProfile(String),

    /// An unsupported feature was encountered.
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    /// An internal error (bug in the writer).
    #[error("Internal error: {0}")]
    Internal(String),

    /// A font could not be parsed or loaded.
    #[error("Font load error: {0}")]
    FontLoad(String),

    /// An error occurred during font subsetting.
    #[error("Font subsetting error: {0}")]
    FontSubset(String),
}

impl PdfError {
    /// Returns a user-facing suggestion for how to fix the error.
    pub fn suggestion(&self) -> Option<&'static str> {
        match self {
            PdfError::Conformance(msg) => {
                if msg.contains("Transparency") {
                    Some("Export as PDF/X-4 to preserve transparency, or remove opacity from your document.")
                } else if msg.contains("RGB") {
                    Some("The document contains RGB colours. The writer will try to convert them, but check your swatches.")
                } else {
                    Some("Review the conformance report for specific violations.")
                }
            }
            PdfError::ColourProfile(_) => Some("Ensure the Output Intent ICC profile is installed and readable."),
            PdfError::FontLoad(msg) if msg.contains("not found") => {
                Some("The requested font is missing on the system. Try using a standard font like Newsreader or Public Sans.")
            }
            PdfError::FontSubset(_) => Some("Try flattening your text or using a different font face."),
            PdfError::Unsupported(msg) if msg.contains("Image") => {
                Some("Some image formats are not supported in PDF/X yet. Try using PNG or JPEG.")
            }
            _ => None,
        }
    }
}

impl From<std::io::Error> for PdfError {
    fn from(e: std::io::Error) -> Self {
        PdfError::Io(e.to_string())
    }
}
