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

//! Format-agnostic vector document types for AppThere Loki Vector.
//!
//! This crate provides parsing, writing, and in-memory representation for
//! SVG-based vector documents. It targets feature parity with Inkscape/Illustrator
//! for professional vector image editing.

pub mod canvas;
pub mod convert;
pub mod document;
pub mod layer;
pub mod object;
pub mod style;
pub mod svg_parser;
pub mod svg_writer;
pub mod transform;
pub mod units;

pub use canvas::Canvas;
pub use common_core::colour_management::Colour;
pub use document::VectorDocument;
pub use layer::Layer;
pub use object::VectorObject;
pub use style::{Paint, StrokeStyle};
pub use transform::Transform;
pub use units::LengthUnit;
