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

//! `loki-pdf` — PDF/X validator and writer for AppThere Loki.
//!
//! # Usage
//!
//! 1. Call [`conformance::validate`] to check a [`VectorDocument`] against
//!    the chosen [`PdfExportSettings`].
//! 2. If the report is conformant, call [`writer::write_pdf_x`] to produce
//!    PDF bytes.
//!
//! The writer internally re-validates and will refuse to produce output for
//! a non-conformant document.

pub mod conformance;
pub mod error;
pub mod export_settings;
pub(crate) mod flatten;
pub(crate) mod preexport;
pub mod writer;

// Re-export the most commonly used types.
pub use conformance::{validate, ConformanceReport, ConformanceViolation};
pub use error::PdfError;
pub use export_settings::{PdfExportSettings, PdfXStandard};
pub use writer::write_pdf_x;
