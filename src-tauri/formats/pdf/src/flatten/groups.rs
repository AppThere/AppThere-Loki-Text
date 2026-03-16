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

//! Object grouping: determines which objects need transparency flattening.

use vector_core::object::VectorObject;

/// Returns `true` if the object has any opacity value less than 1.0.
pub fn is_transparent(obj: &VectorObject) -> bool {
    let s = &obj.common().style;
    s.opacity < 1.0 - f64::EPSILON
        || s.fill_opacity < 1.0 - f64::EPSILON
        || s.stroke_opacity < 1.0 - f64::EPSILON
}

/// Returns `true` if any object in the slice (or any nested group child)
/// has opacity < 1.0.
pub fn layer_has_transparency(objects: &[VectorObject]) -> bool {
    objects.iter().any(|obj| {
        if is_transparent(obj) {
            return true;
        }
        if let VectorObject::Group(g) = obj {
            layer_has_transparency(&g.children)
        } else {
            false
        }
    })
}
