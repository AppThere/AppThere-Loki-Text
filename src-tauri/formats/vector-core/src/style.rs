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
use crate::colour::Colour;

/// Describes how a shape is filled or stroked.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Paint {
    None,
    Solid { colour: Colour },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub paint: Paint,
    /// Stroke width in pixels.
    pub width: f64,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f64,
    pub dash_array: Vec<f64>,
    pub dash_offset: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStyle {
    pub fill: Paint,
    pub stroke: StrokeStyle,
    /// Overall opacity, 0.0–1.0.
    pub opacity: f64,
    pub fill_opacity: f64,
    pub stroke_opacity: f64,
}

impl StrokeStyle {
    pub fn none() -> Self {
        StrokeStyle {
            paint: Paint::None,
            width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 4.0,
            dash_array: vec![],
            dash_offset: 0.0,
        }
    }

    pub fn solid_black_1px() -> Self {
        StrokeStyle {
            paint: Paint::Solid { colour: Colour::black() },
            width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 4.0,
            dash_array: vec![],
            dash_offset: 0.0,
        }
    }
}

impl ObjectStyle {
    /// Black fill, no stroke.
    pub fn default_fill() -> Self {
        ObjectStyle {
            fill: Paint::Solid { colour: Colour::black() },
            stroke: StrokeStyle::none(),
            opacity: 1.0,
            fill_opacity: 1.0,
            stroke_opacity: 1.0,
        }
    }

    /// No fill, black 1px stroke.
    pub fn default_stroke() -> Self {
        ObjectStyle {
            fill: Paint::None,
            stroke: StrokeStyle::solid_black_1px(),
            opacity: 1.0,
            fill_opacity: 1.0,
            stroke_opacity: 1.0,
        }
    }
}
