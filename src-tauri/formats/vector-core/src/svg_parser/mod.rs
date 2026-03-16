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

use crate::canvas::{Canvas, ViewBox};
use crate::document::VectorDocument;
use crate::layer::Layer;
use crate::object::{CommonProps, GroupObject, VectorObject};
use crate::transform::{parse_svg_transform, Transform};
use crate::units::{LengthUnit, UnitConverter};
use common_core::colour_management::{DocumentColourSettings, SwatchLibrary};
use common_core::Metadata;

mod nodes;
mod style_parse;

use nodes::{parse_ellipse, parse_line, parse_path, parse_rect};
use style_parse::parse_style;

/// Loki SVG extension namespace URI.
const LOKI_NS: &str = "http://appthere.com/ns/loki/1.0";

/// Parse an SVG string into a VectorDocument.
pub fn parse(svg: &str) -> Result<VectorDocument, String> {
    let doc = roxmltree::Document::parse(svg).map_err(|e| format!("SVG parse error: {e}"))?;

    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return Err("Root element is not <svg>".into());
    }

    let canvas = parse_canvas(&root)?;
    let mut layers: Vec<Layer> = Vec::new();
    let mut default_layer = Layer::new("Layer 1");

    for child in root.children().filter(|n| n.is_element()) {
        let tag = child.tag_name().name();
        if tag == "g" {
            if let Some(label) = child
                .attribute(("http://www.inkscape.org/namespaces/inkscape", "label"))
                .or_else(|| child.attribute("inkscape:label"))
            {
                let mut layer = Layer {
                    id: child.attribute("id").unwrap_or("layer").to_string(),
                    name: label.to_string(),
                    visible: child.attribute("display") != Some("none"),
                    locked: false,
                    objects: Vec::new(),
                };
                parse_children(&child, &mut layer.objects);
                layers.push(layer);
                continue;
            }
        }
        parse_node(&child, &mut default_layer.objects);
    }

    if !default_layer.objects.is_empty() || layers.is_empty() {
        layers.insert(0, default_layer);
    }

    let colour_settings = parse_loki_colour_settings(&root);

    Ok(VectorDocument {
        canvas,
        layers,
        metadata: Metadata::default(),
        colour_settings,
        swatch_library: SwatchLibrary::new(),
    })
}

/// Parse the `loki:colour-settings` attribute from the SVG root element.
///
/// Returns `DocumentColourSettings::default()` when the attribute is absent or
/// cannot be deserialised, so older SVG files are handled transparently.
fn parse_loki_colour_settings(root: &roxmltree::Node) -> DocumentColourSettings {
    let json = root
        .attribute((LOKI_NS, "colour-settings"))
        .or_else(|| root.attribute("loki:colour-settings"));
    json.and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

fn parse_canvas(root: &roxmltree::Node) -> Result<Canvas, String> {
    let c96 = UnitConverter::new(96.0);
    let width = c96
        .parse_length(root.attribute("width").unwrap_or("800"))
        .unwrap_or(800.0);
    let height = c96
        .parse_length(root.attribute("height").unwrap_or("600"))
        .unwrap_or(600.0);
    let viewbox = root.attribute("viewBox").and_then(parse_viewbox);
    Ok(Canvas {
        width,
        height,
        display_unit: LengthUnit::Px,
        dpi: 96.0,
        viewbox,
    })
}

fn parse_viewbox(s: &str) -> Option<ViewBox> {
    let nums: Vec<f64> = s
        .split_whitespace()
        .filter_map(|p| p.parse().ok())
        .collect();
    if nums.len() == 4 {
        Some(ViewBox {
            x: nums[0],
            y: nums[1],
            width: nums[2],
            height: nums[3],
        })
    } else {
        None
    }
}

pub(crate) fn parse_children(node: &roxmltree::Node, out: &mut Vec<VectorObject>) {
    for child in node.children().filter(|n| n.is_element()) {
        parse_node(&child, out);
    }
}

fn parse_node(node: &roxmltree::Node, out: &mut Vec<VectorObject>) {
    match node.tag_name().name() {
        "rect" => {
            if let Some(o) = parse_rect(node) {
                out.push(VectorObject::Rect(o));
            }
        }
        "ellipse" => {
            if let Some(o) = parse_ellipse(node, false) {
                out.push(VectorObject::Ellipse(o));
            }
        }
        "circle" => {
            if let Some(o) = parse_ellipse(node, true) {
                out.push(VectorObject::Ellipse(o));
            }
        }
        "line" => {
            if let Some(o) = parse_line(node) {
                out.push(VectorObject::Line(o));
            }
        }
        "path" => {
            if let Some(o) = parse_path(node) {
                out.push(VectorObject::Path(o));
            }
        }
        "g" => {
            let mut children = Vec::new();
            parse_children(node, &mut children);
            out.push(VectorObject::Group(GroupObject {
                common: parse_common(node),
                children,
            }));
        }
        _ => {}
    }
}

pub(crate) fn parse_common(node: &roxmltree::Node) -> CommonProps {
    use crate::object::ObjectId;
    let id = ObjectId(node.attribute("id").unwrap_or("obj").to_string());
    let label = node
        .attribute("inkscape:label")
        .or_else(|| node.attribute(("http://www.inkscape.org/namespaces/inkscape", "label")))
        .map(str::to_string);
    let transform = node
        .attribute("transform")
        .map(parse_svg_transform)
        .unwrap_or_else(Transform::identity);
    let style = parse_style(node);
    let visible =
        node.attribute("display") != Some("none") && node.attribute("visibility") != Some("hidden");
    CommonProps {
        id,
        label,
        style,
        transform,
        visible,
        locked: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect id="r1" x="10" y="10" width="50" height="30"/>
            <ellipse id="e1" cx="60" cy="60" rx="20" ry="15"/>
            <path id="p1" d="M 0 0 L 100 100"/>
        </svg>"#;
        let doc = parse(svg).unwrap();
        let layer = &doc.layers[0];
        assert_eq!(layer.objects.len(), 3);
        assert!(matches!(layer.objects[0], VectorObject::Rect(_)));
        assert!(matches!(layer.objects[1], VectorObject::Ellipse(_)));
        assert!(matches!(layer.objects[2], VectorObject::Path(_)));
    }

    #[test]
    fn test_parse_rect_properties() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect id="r1" x="5" y="10" width="80" height="40"/>
        </svg>"#;
        let doc = parse(svg).unwrap();
        if let VectorObject::Rect(r) = &doc.layers[0].objects[0] {
            assert_eq!(r.x, 5.0);
            assert_eq!(r.y, 10.0);
        } else {
            panic!("expected Rect");
        }
    }

    #[test]
    fn test_parse_unit_suffixes() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="25.4mm" height="1in"></svg>"#;
        let doc = parse(svg).unwrap();
        assert!(
            (doc.canvas.width - 96.0).abs() < 0.01,
            "width={}",
            doc.canvas.width
        );
        assert!(
            (doc.canvas.height - 96.0).abs() < 0.01,
            "height={}",
            doc.canvas.height
        );
    }

    #[test]
    fn test_parse_circle() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <circle id="c1" cx="50" cy="50" r="25"/>
        </svg>"#;
        let doc = parse(svg).unwrap();
        if let VectorObject::Ellipse(e) = &doc.layers[0].objects[0] {
            assert_eq!(e.rx, 25.0);
            assert_eq!(e.ry, 25.0);
        } else {
            panic!("expected Ellipse from circle");
        }
    }
}
