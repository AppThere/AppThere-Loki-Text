//! ODT style parser.
//!
//! Parses `style:style` and `style:default-style` elements from ODT XML
//! into [`StyleDefinition`] structs and a style-to-marks lookup map.

use std::collections::HashMap;

use common_core::{StyleDefinition, StyleFamily, TiptapMark};

use crate::namespaces::ns_prefix;

/// Parses all named styles from an ODT document root.
///
/// Scans `office:styles` and `office:automatic-styles` sections for
/// `style:style` elements and builds two complementary data structures:
/// - A `StyleDefinition` map for the frontend
/// - A lightweight `style_map` used during block/inline parsing
///
/// # Arguments
///
/// * `root` - The document root element.
/// * `ns_office` - The `office:` namespace URI.
/// * `ns_style` - The `style:` namespace URI.
/// * `ns_fo` - The `fo:` namespace URI.
/// * `ns_text` - The `text:` namespace URI.
/// * `ns_loki` - The Loki custom namespace URI.
///
/// # Returns
///
/// A tuple of `(style_definitions, style_map)` where `style_map` maps
/// style names to `(family_str, marks)` for use during inline parsing.
pub fn parse_styles(
    root: roxmltree::Node,
    ns_office: &str,
    ns_style: &str,
    ns_fo: &str,
    ns_text: &str,
    ns_loki: &str,
) -> (HashMap<String, StyleDefinition>, HashMap<String, (String, Vec<TiptapMark>)>) {
    let mut style_map = build_default_style_map();
    let mut style_definitions = HashMap::new();

    let style_nodes = root
        .children()
        .filter(|n| {
            n.has_tag_name((ns_office, "styles"))
                || n.has_tag_name((ns_office, "automatic-styles"))
                || n.has_tag_name((ns_office, "font-face-decls"))
        })
        .flat_map(|n| n.children())
        .filter(|n| n.has_tag_name((ns_style, "style")));

    for style_node in style_nodes {
        if let Some(name) = style_node.attribute((ns_style, "name")) {
            let def = parse_single_style(style_node, name, ns_style, ns_fo, ns_text, ns_loki);
            let marks = extract_marks_from_style(style_node, ns_style, ns_fo);
            style_map.insert(name.to_string(), (def.family.to_odf_str().to_string(), marks));
            style_definitions.insert(name.to_string(), def);
        }
    }

    parse_default_styles(root, ns_office, ns_style, ns_fo, ns_text, &mut style_definitions);
    link_styles_to_defaults(&mut style_definitions);

    (style_definitions, style_map)
}

/// Parses style properties from `office:styles` in a standalone styles.xml.
///
/// Called when loading a `.odt` ZIP file's `styles.xml`. Populates the
/// document's `styles` map and preserves raw XML sections.
///
/// # Arguments
///
/// * `styles_node` - The `<office:styles>` element.
/// * `ns_style` - The `style:` namespace URI.
/// * `ns_fo` - The `fo:` namespace URI.
/// * `ns_text` - The `text:` namespace URI.
/// * `ns_loki` - The Loki custom namespace URI.
pub fn parse_styles_node(
    styles_node: roxmltree::Node,
    ns_style: &str,
    ns_fo: &str,
    ns_text: &str,
    ns_loki: &str,
) -> Result<HashMap<String, StyleDefinition>, String> {
    let mut styles = HashMap::new();

    for style_node in styles_node.children() {
        if !style_node.is_element() {
            continue;
        }
        if style_node.has_tag_name((ns_style, "style"))
            || style_node.has_tag_name((ns_style, "default-style"))
        {
            let is_default = style_node.has_tag_name((ns_style, "default-style"));
            let style_name = if is_default {
                default_style_name(style_node, ns_style)
            } else {
                style_node
                    .attribute((ns_style, "name"))
                    .ok_or("Style missing style:name attribute")?
                    .to_string()
            };
            let def = parse_single_style(style_node, &style_name, ns_style, ns_fo, ns_text, ns_loki);
            styles.insert(style_name, def);
        }
    }

    Ok(styles)
}

/// Builds a default style_map with well-known built-in style mappings.
fn build_default_style_map() -> HashMap<String, (String, Vec<TiptapMark>)> {
    let mut m = HashMap::new();
    m.insert("Strong".to_string(), ("text".to_string(), vec![TiptapMark::Bold]));
    m.insert("Emphasis".to_string(), ("text".to_string(), vec![TiptapMark::Italic]));
    m
}

/// Returns a synthetic name for a `style:default-style` element.
fn default_style_name(node: roxmltree::Node, ns_style: &str) -> String {
    match node.attribute((ns_style, "family")).unwrap_or("") {
        "paragraph" => "_Default_Paragraph".to_string(),
        "text" => "_Default_Text".to_string(),
        f => format!("_Default_{f}"),
    }
}

/// Parses a single `style:style` or `style:default-style` into a [`StyleDefinition`].
fn parse_single_style(
    style_node: roxmltree::Node,
    name: &str,
    ns_style: &str,
    ns_fo: &str,
    ns_text: &str,
    ns_loki: &str,
) -> StyleDefinition {
    let family_str = style_node.attribute((ns_style, "family")).unwrap_or("paragraph");
    let family = match family_str {
        "paragraph" => StyleFamily::Paragraph,
        "text" => StyleFamily::Text,
        _ => StyleFamily::Text,
    };
    let parent = style_node.attribute((ns_style, "parent-style-name")).map(|s| s.to_string());
    let display_name = style_node.attribute((ns_style, "display-name")).map(|s| s.to_string());
    let next = style_node.attribute((ns_style, "next-style-name")).map(|s| s.to_string());
    let outline_level = style_node.attribute((ns_style, "outline-level")).and_then(|s| s.parse().ok());
    let autocomplete = style_node.attribute((ns_loki, "autocomplete")).map(|s| s == "true");

    let attributes = collect_style_attributes(style_node, ns_style, ns_fo, ns_text);
    let text_transform = attributes.get("fo:text-transform").cloned();

    StyleDefinition {
        name: name.to_string(),
        family,
        parent,
        next,
        display_name,
        attributes,
        text_transform,
        outline_level,
        autocomplete,
    }
}

/// Collects all `fo:`, `style:`, and `text:` attributes from style property nodes.
fn collect_style_attributes(
    style_node: roxmltree::Node,
    ns_style: &str,
    ns_fo: &str,
    ns_text: &str,
) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    for prop_node in style_node.children() {
        if prop_node.has_tag_name((ns_style, "text-properties"))
            || prop_node.has_tag_name((ns_style, "paragraph-properties"))
        {
            for attr in prop_node.attributes() {
                let prefix = attr.namespace().map(ns_prefix).unwrap_or("");
                let key = format!("{}{}", prefix, attr.name());
                attrs.insert(key, attr.value().to_string());
            }
        }
        // Also collect from all other property children for legacy compat
        for attr in prop_node.attributes() {
            if let Some(ns) = attr.namespace() {
                if ns == ns_fo || ns == ns_style || ns == ns_text {
                    let prefix = ns_prefix(ns);
                    let key = format!("{}{}", prefix, attr.name());
                    attrs.entry(key).or_insert_with(|| attr.value().to_string());
                }
            }
        }
    }
    attrs
}

/// Extracts TiptapMark values from a style's text-properties.
fn extract_marks_from_style(
    style_node: roxmltree::Node,
    ns_style: &str,
    ns_fo: &str,
) -> Vec<TiptapMark> {
    let mut marks = Vec::new();
    for prop_node in style_node.children() {
        if prop_node.has_tag_name((ns_style, "text-properties")) {
            if prop_node.attribute((ns_fo, "font-weight")) == Some("bold") {
                marks.push(TiptapMark::Bold);
            }
            if prop_node.attribute((ns_fo, "font-style")) == Some("italic") {
                marks.push(TiptapMark::Italic);
            }
            if prop_node
                .attribute((ns_style, "text-underline-style"))
                .is_some_and(|u| u != "none")
            {
                marks.push(TiptapMark::Underline);
            }
        }
    }
    marks
}

/// Parses `style:default-style` elements from `office:styles`.
fn parse_default_styles(
    root: roxmltree::Node,
    ns_office: &str,
    ns_style: &str,
    ns_fo: &str,
    ns_text: &str,
    style_definitions: &mut HashMap<String, StyleDefinition>,
) {
    let default_style_nodes = root
        .children()
        .filter(|n| n.has_tag_name((ns_office, "styles")))
        .flat_map(|n| n.children())
        .filter(|n| n.has_tag_name((ns_style, "default-style")));

    for ds_node in default_style_nodes {
        let family_str = ds_node.attribute((ns_style, "family")).unwrap_or("");
        let (family, name) = match family_str {
            "paragraph" => (StyleFamily::Paragraph, "_Default_Paragraph"),
            "text" => (StyleFamily::Text, "_Default_Text"),
            _ => continue,
        };

        let mut attrs = HashMap::new();
        for prop_node in ds_node.children() {
            if prop_node.has_tag_name((ns_style, "text-properties"))
                || prop_node.has_tag_name((ns_style, "paragraph-properties"))
            {
                for attr in prop_node.attributes() {
                    let prefix = attr.namespace().map(ns_prefix).unwrap_or("");
                    let key = format!("{}{}", prefix, attr.name());
                    attrs.insert(key, attr.value().to_string());
                }
            }
        }
        let _ = (ns_fo, ns_text); // used by ns_prefix transitively

        style_definitions.insert(name.to_string(), StyleDefinition {
            name: name.to_string(),
            family,
            parent: None,
            next: None,
            display_name: Some("Default".to_string()),
            attributes: attrs,
            text_transform: None,
            outline_level: None,
            autocomplete: None,
        });
    }
}

/// Links styles that have no parent to the appropriate default style.
pub fn link_styles_to_defaults(style_definitions: &mut HashMap<String, StyleDefinition>) {
    for style in style_definitions.values_mut() {
        if style.parent.is_none() {
            match style.family {
                StyleFamily::Paragraph if style.name != "_Default_Paragraph" => {
                    style.parent = Some("_Default_Paragraph".to_string());
                }
                StyleFamily::Text if style.name != "_Default_Text" => {
                    style.parent = Some("_Default_Text".to_string());
                }
                _ => {}
            }
        }
    }
}

