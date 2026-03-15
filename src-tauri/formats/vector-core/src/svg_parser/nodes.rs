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

use super::parse_common;
use crate::object::{EllipseObject, LineObject, PathObject, RectObject};

pub(crate) fn parse_rect(node: &roxmltree::Node) -> Option<RectObject> {
    let a = |k: &str| {
        node.attribute(k)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    Some(RectObject {
        common: parse_common(node),
        x: a("x"),
        y: a("y"),
        width: a("width"),
        height: a("height"),
        rx: a("rx"),
        ry: a("ry"),
    })
}

pub(crate) fn parse_ellipse(node: &roxmltree::Node, is_circle: bool) -> Option<EllipseObject> {
    let a = |k: &str| {
        node.attribute(k)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    let (rx, ry) = if is_circle {
        let r = a("r");
        (r, r)
    } else {
        (a("rx"), a("ry"))
    };
    Some(EllipseObject {
        common: parse_common(node),
        cx: a("cx"),
        cy: a("cy"),
        rx,
        ry,
    })
}

pub(crate) fn parse_line(node: &roxmltree::Node) -> Option<LineObject> {
    let a = |k: &str| {
        node.attribute(k)
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    Some(LineObject {
        common: parse_common(node),
        x1: a("x1"),
        y1: a("y1"),
        x2: a("x2"),
        y2: a("y2"),
    })
}

pub(crate) fn parse_path(node: &roxmltree::Node) -> Option<PathObject> {
    let d = node.attribute("d")?.to_string();
    Some(PathObject {
        common: parse_common(node),
        d,
    })
}
