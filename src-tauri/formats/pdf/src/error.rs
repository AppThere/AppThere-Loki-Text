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
}

impl From<std::io::Error> for PdfError {
    fn from(e: std::io::Error) -> Self {
        PdfError::Io(e.to_string())
    }
}
