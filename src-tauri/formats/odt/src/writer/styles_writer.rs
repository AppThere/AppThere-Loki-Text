//! ODT `styles.xml` writer.
//!
//! Generates the `<office:styles>` section and a standalone `styles.xml`
//! document from a document's named style definitions.

use std::collections::HashMap;
use std::io::Cursor;

use common_core::colour_management::Colour;
use common_core::{StyleDefinition, StyleFamily};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;

use crate::writer::namespaces::push_styles_doc_ns;

/// Generates a standalone `styles.xml` document string.
///
/// # Errors
///
/// Returns a `String` error if XML writing fails.
pub fn styles_to_xml(
    styles: &HashMap<String, StyleDefinition>,
    font_face_decls: &Option<String>,
    automatic_styles: &Option<String>,
    master_styles: &Option<String>,
) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| e.to_string())?;

    let mut root = BytesStart::new("office:document-styles");
    push_styles_doc_ns(&mut root);
    writer
        .write_event(Event::Start(root))
        .map_err(|e| e.to_string())?;

    write_preserved_section(&mut writer, font_face_decls)?;
    write_styles_section(&mut writer, styles)?;
    write_preserved_section(&mut writer, automatic_styles)?;
    write_preserved_section(&mut writer, master_styles)?;

    writer
        .write_event(Event::End(BytesEnd::new("office:document-styles")))
        .map_err(|e| e.to_string())?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.to_string())
}

/// Writes the `<office:styles>` section with all named style definitions.
///
/// Called by both [`styles_to_xml`] and the FODT writer.
pub fn write_styles_section(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    styles: &HashMap<String, StyleDefinition>,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("office:styles")))
        .map_err(|e| e.to_string())?;

    for (style_name, style_def) in styles {
        // Standard built-ins are handled explicitly below
        if style_name != "PageBreak"
            && style_name != "Strong"
            && style_name != "Emphasis"
            && style_name != "Underline"
            && style_name != "Strike"
            && style_name != "Superscript"
            && style_name != "Subscript"
        {
            write_style_definition(writer, style_name, style_def)?;
        }
    }

    write_builtin_styles(writer)?;

    writer
        .write_event(Event::End(BytesEnd::new("office:styles")))
        .map_err(|e| e.to_string())
}

fn write_style_definition(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    style_name: &str,
    style_def: &StyleDefinition,
) -> Result<(), String> {
    let mut style_elem = BytesStart::new("style:style");
    style_elem.push_attribute(("style:name", style_name));
    style_elem.push_attribute(("style:family", style_def.family.to_odf_str()));

    if let Some(ref parent) = style_def.parent {
        style_elem.push_attribute(("style:parent-style-name", parent.as_str()));
    }
    if let Some(ref next) = style_def.next {
        style_elem.push_attribute(("style:next-style-name", next.as_str()));
    }
    if let Some(ref display_name) = style_def.display_name {
        style_elem.push_attribute(("style:display-name", display_name.as_str()));
    }
    if let Some(level) = style_def.outline_level {
        style_elem.push_attribute(("style:outline-level", level.to_string().as_str()));
    }
    if style_def.autocomplete == Some(true) {
        style_elem.push_attribute(("loki:autocomplete", "true"));
    }

    writer
        .write_event(Event::Start(style_elem))
        .map_err(|e| e.to_string())?;

    if style_def.family == StyleFamily::Paragraph {
        write_paragraph_properties(writer, &style_def.attributes)?;
    }
    write_text_properties(
        writer,
        &style_def.attributes,
        &style_def.text_transform,
        style_def.font_colour.as_ref(),
        style_def.background_colour.as_ref(),
    )?;

    writer
        .write_event(Event::End(BytesEnd::new("style:style")))
        .map_err(|e| e.to_string())
}

/// Writes `<style:paragraph-properties>` from the style's attribute map.
fn write_paragraph_properties(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    attributes: &HashMap<String, String>,
) -> Result<(), String> {
    let mut para_props = BytesStart::new("style:paragraph-properties");
    for (key, value) in attributes {
        if is_paragraph_property(key) {
            let coerced = coerce_line_height(key, value);
            para_props.push_attribute((key.as_str(), coerced.as_str()));
        }
    }
    writer
        .write_event(Event::Empty(para_props))
        .map_err(|e| e.to_string())
}

/// Writes `<style:text-properties>` from the style's attribute map and typed colour fields.
///
/// When `font_colour` is `Some`, it takes precedence over any `fo:color` /
/// `loki:colour` entries in `attributes`. When `background_colour` is `Some`,
/// it takes precedence over `fo:background-color` in `attributes`.
fn write_text_properties(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    attributes: &HashMap<String, String>,
    text_transform: &Option<String>,
    font_colour: Option<&Colour>,
    background_colour: Option<&Colour>,
) -> Result<(), String> {
    use crate::loki_ext::{colour_to_attr, colour_to_odf_string, needs_loki_attr, LOKI_COLOUR_KEY};

    // Pre-compute typed colour attribute strings so they outlive the loop.
    let typed_font: Option<(String, Option<String>)> = font_colour.map(|c| {
        let hex = colour_to_odf_string(c);
        let loki = if needs_loki_attr(c) {
            colour_to_attr(c)
        } else {
            None
        };
        (hex, loki)
    });
    let typed_bg: Option<String> = background_colour.map(colour_to_odf_string);

    let mut text_props = BytesStart::new("style:text-properties");
    let mut has_props = false;

    for (key, value) in attributes {
        if is_text_property(key) {
            // Skip keys that will be emitted from typed fields to avoid duplicates.
            if typed_font.is_some() && (key == "fo:color" || key == LOKI_COLOUR_KEY) {
                continue;
            }
            if typed_bg.is_some() && key == "fo:background-color" {
                continue;
            }
            text_props.push_attribute((key.as_str(), value.as_str()));
            has_props = true;
        }
    }

    if let Some((ref hex, ref loki)) = typed_font {
        text_props.push_attribute(("fo:color", hex.as_str()));
        has_props = true;
        if let Some(ref json) = loki {
            text_props.push_attribute((LOKI_COLOUR_KEY, json.as_str()));
        }
    }
    if let Some(ref hex) = typed_bg {
        text_props.push_attribute(("fo:background-color", hex.as_str()));
        has_props = true;
    }

    if let Some(transform) = text_transform {
        text_props.push_attribute(("fo:text-transform", transform.as_str()));
        has_props = true;
    }

    if has_props {
        writer
            .write_event(Event::Empty(text_props))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Returns `true` if `key` is a paragraph-property attribute.
fn is_paragraph_property(key: &str) -> bool {
    crate::writer::styles_utils::is_paragraph_property(key)
}

/// Returns `true` if `key` is a text-property attribute.
fn is_text_property(key: &str) -> bool {
    crate::writer::styles_utils::is_text_property(key)
}

/// Normalizes unitless line-height values to ODF percent format.
fn coerce_line_height(key: &str, value: &str) -> String {
    crate::writer::styles_utils::coerce_line_height(key, value)
}

fn write_builtin_styles(writer: &mut Writer<Cursor<Vec<u8>>>) -> Result<(), String> {
    let builtins = [
        ("PageBreak", "paragraph", Some(("fo:break-before", "page"))),
        ("Strong", "text", Some(("fo:font-weight", "bold"))),
        ("Emphasis", "text", Some(("fo:font-style", "italic"))),
        (
            "Underline",
            "text",
            Some(("style:text-underline-style", "solid")),
        ),
        (
            "Strike",
            "text",
            Some(("style:text-line-through-style", "solid")),
        ),
        (
            "Superscript",
            "text",
            Some(("style:text-position", "super 58%")),
        ),
        (
            "Subscript",
            "text",
            Some(("style:text-position", "sub 58%")),
        ),
    ];

    for (name, family, prop) in builtins {
        let mut style = BytesStart::new("style:style");
        style.push_attribute(("style:name", name));
        style.push_attribute(("style:family", family));
        if name == "PageBreak" {
            style.push_attribute(("style:parent-style-name", "Standard"));
        }
        writer
            .write_event(Event::Start(style))
            .map_err(|e| e.to_string())?;

        if let Some((k, v)) = prop {
            let prop_name = if family == "paragraph" {
                "style:paragraph-properties"
            } else {
                "style:text-properties"
            };
            let mut props = BytesStart::new(prop_name);
            props.push_attribute((k, v));
            if name == "Underline" {
                props.push_attribute(("style:text-underline-width", "auto"));
                props.push_attribute(("style:text-underline-color", "font-color"));
            }
            writer
                .write_event(Event::Empty(props))
                .map_err(|e| e.to_string())?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("style:style")))
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Writes a preserved raw XML section (or nothing if absent).
fn write_preserved_section(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    content: &Option<String>,
) -> Result<(), String> {
    if let Some(ref xml) = content {
        writer
            .write_event(Event::Text(quick_xml::events::BytesText::from_escaped(xml)))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
