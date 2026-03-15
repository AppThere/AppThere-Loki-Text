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

use crate::style::ObjectStyle;
use crate::transform::Transform;
use serde::{Deserialize, Serialize};

/// A unique identifier for a vector object (UUID string).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub String);

/// Properties shared by all vector objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonProps {
    pub id: ObjectId,
    pub label: Option<String>,
    pub style: ObjectStyle,
    pub transform: Transform,
    pub visible: bool,
    pub locked: bool,
}

impl CommonProps {
    pub fn new(id: impl Into<String>) -> Self {
        CommonProps {
            id: ObjectId(id.into()),
            label: None,
            style: ObjectStyle::default_fill(),
            transform: Transform::identity(),
            visible: true,
            locked: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RectObject {
    #[serde(flatten)]
    pub common: CommonProps,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    /// Corner radius X (0 = sharp).
    pub rx: f64,
    /// Corner radius Y (0 = sharp).
    pub ry: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EllipseObject {
    #[serde(flatten)]
    pub common: CommonProps,
    pub cx: f64,
    pub cy: f64,
    pub rx: f64,
    pub ry: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineObject {
    #[serde(flatten)]
    pub common: CommonProps,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathObject {
    #[serde(flatten)]
    pub common: CommonProps,
    /// SVG path data string (canonical representation).
    pub d: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupObject {
    #[serde(flatten)]
    pub common: CommonProps,
    pub children: Vec<VectorObject>,
}

/// All vector object types. Uses an internal `type` tag for JSON serialisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VectorObject {
    Rect(RectObject),
    Ellipse(EllipseObject),
    Line(LineObject),
    Path(PathObject),
    Group(GroupObject),
}

impl VectorObject {
    pub fn id(&self) -> &ObjectId {
        &self.common().id
    }

    pub fn common(&self) -> &CommonProps {
        match self {
            VectorObject::Rect(o) => &o.common,
            VectorObject::Ellipse(o) => &o.common,
            VectorObject::Line(o) => &o.common,
            VectorObject::Path(o) => &o.common,
            VectorObject::Group(o) => &o.common,
        }
    }

    pub fn common_mut(&mut self) -> &mut CommonProps {
        match self {
            VectorObject::Rect(o) => &mut o.common,
            VectorObject::Ellipse(o) => &mut o.common,
            VectorObject::Line(o) => &mut o.common,
            VectorObject::Path(o) => &mut o.common,
            VectorObject::Group(o) => &mut o.common,
        }
    }
}
