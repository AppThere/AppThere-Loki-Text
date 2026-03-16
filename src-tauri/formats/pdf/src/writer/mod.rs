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

//! PDF/X writer — converts a validated `VectorDocument` to PDF bytes.

pub mod colour;
pub mod content;
pub mod image;
pub mod metadata;
pub mod page;
pub mod path_ops;
pub mod resources;
pub(crate) mod text;
mod vector;

pub use text::write_text_pdf;
pub use vector::write_pdf_x;
