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

use serde::{Deserialize, Serialize};
use crate::canvas::Canvas;
use crate::layer::Layer;
use common_core::Metadata;

/// A complete vector image document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDocument {
    pub canvas: Canvas,
    pub layers: Vec<Layer>,
    pub metadata: Metadata,
}

impl VectorDocument {
    /// Create a new document with one default layer named "Layer 1".
    pub fn new(canvas: Canvas) -> Self {
        VectorDocument {
            canvas,
            layers: vec![Layer::new("Layer 1")],
            metadata: Metadata::default(),
        }
    }

    pub fn blank_a4() -> Self {
        VectorDocument::new(Canvas::a4_portrait())
    }

    pub fn blank_letter() -> Self {
        VectorDocument::new(Canvas::letter_portrait())
    }
}
