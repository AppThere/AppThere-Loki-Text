//! Internal helpers for ODT style parsing.
//!
//! Used exclusively by the [`super::styles`] module.

use std::collections::HashMap;

use common_core::{StyleDefinition, StyleFamily, TiptapMark};

use crate::namespaces::ns_prefix;

/// Parses a single `style:style` or `style:default-style` into a [`StyleDefinition`].
pub(super) fn parse_single_style(
    style_node: roxmltree::Node,
    name: &str,
    ns_style: &str,
    ns_fo: &str,
    ns_text: &str,
    ns_loki: &str,
) -> StyleDefinition {
    let family_str = style_node
        .attribute((ns_style, "family"))
        .unwrap_or("paragraph");
    let family = match family_str {
        "paragraph" => StyleFamily::Paragraph,
        "text" => StyleFamily::Text,
        _ => StyleFamily::Text,
    };
    let parent = style_node
        .attribute((ns_style, "parent-style-name"))
        .map(|s| s.to_string());
    let display_name = style_node
        .attribute((ns_style, "display-name"))
        .map(|s| s.to_string());
    let next = style_node
        .attribute((ns_style, "next-style-name"))
        .map(|s| s.to_string());
    let outline_level = style_node
        .attribute((ns_style, "outline-level"))
        .and_then(|s| s.parse().ok());
    let autocomplete = style_node
        .attribute((ns_loki, "autocomplete"))
        .map(|s| s == "true");

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
pub(super) fn collect_style_attributes(
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
pub(super) fn extract_marks_from_style(
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
pub(super) fn parse_default_styles(
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

        style_definitions.insert(
            name.to_string(),
            StyleDefinition {
                name: name.to_string(),
                family,
                parent: None,
                next: None,
                display_name: Some("Default".to_string()),
                attributes: attrs,
                text_transform: None,
                outline_level: None,
                autocomplete: None,
            },
        );
    }
}
