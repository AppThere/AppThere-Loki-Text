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

//! Iteration helpers for walking document objects and colours.

use common_core::colour_management::Colour;
use vector_core::document::VectorDocument;
use vector_core::object::VectorObject;
use vector_core::style::Paint;

pub(super) fn for_each_object<F>(document: &VectorDocument, mut f: F)
where
    F: FnMut(&VectorObject, &str),
{
    for layer in &document.layers {
        for obj in &layer.objects {
            let loc = format!("layer '{}' / object '{}'", layer.name, obj.id().0);
            visit_object(obj, &loc, &mut f);
        }
    }
}

fn visit_object<F>(obj: &VectorObject, location: &str, f: &mut F)
where
    F: FnMut(&VectorObject, &str),
{
    f(obj, location);
    if let VectorObject::Group(g) = obj {
        for child in &g.children {
            let loc = format!("{} / '{}'", location, child.id().0);
            visit_object(child, &loc, f);
        }
    }
}

pub(super) fn for_each_colour<F>(document: &VectorDocument, mut f: F)
where
    F: FnMut(&Colour, &str),
{
    for_each_object(document, |obj, location| {
        let style = &obj.common().style;
        if let Paint::Solid { colour } = &style.fill {
            f(colour, location);
        }
        if let Paint::Solid { colour } = &style.stroke.paint {
            f(colour, location);
        }
    });
}
