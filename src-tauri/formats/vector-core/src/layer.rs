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
use crate::object::VectorObject;

/// A layer in a vector document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub objects: Vec<VectorObject>,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| format!("layer-{}", d.subsec_nanos()))
            .unwrap_or_else(|_| "layer-1".to_string());
        Layer {
            id,
            name: name.into(),
            visible: true,
            locked: false,
            objects: vec![],
        }
    }
}
