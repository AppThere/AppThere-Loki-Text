//! ODT `styles.xml` writer.
//!
//! Generates the `<office:styles>` section and a standalone `styles.xml`
//! document from a document's named style definitions.

use std::collections::HashMap;
use std::io::Cursor;

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
        write_style_definition(writer, style_name, style_def)?;
    }

    write_page_break_style(writer)?;

    writer
        .write_event(Event::End(BytesEnd::new("office:styles")))
        .map_err(|e| e.to_string())
}

/// Writes a single `<style:style>` element with all its properties.
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
    write_text_properties(writer, &style_def.attributes, &style_def.text_transform)?;

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

/// Writes `<style:text-properties>` from the style's attribute map.
fn write_text_properties(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    attributes: &HashMap<String, String>,
    text_transform: &Option<String>,
) -> Result<(), String> {
    let mut text_props = BytesStart::new("style:text-properties");
    let mut has_props = false;

    for (key, value) in attributes {
        if is_text_property(key) {
            text_props.push_attribute((key.as_str(), value.as_str()));
            has_props = true;
        }
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
    key.starts_with("fo:margin")
        || key.starts_with("fo:text-indent")
        || key.starts_with("fo:text-align")
        || key.starts_with("fo:orphans")
        || key.starts_with("fo:widows")
        || key.starts_with("fo:hyphenate")
        || key.starts_with("fo:break-")
        || key == "fo:line-height"
}

/// Returns `true` if `key` is a text-property attribute.
fn is_text_property(key: &str) -> bool {
    key.starts_with("fo:font")
        || key.starts_with("fo:color")
        || key.starts_with("fo:font-size")
        || key.starts_with("fo:font-weight")
        || key.starts_with("fo:font-style")
        || key.starts_with("fo:text-transform")
}

/// Normalizes unitless line-height values to ODF percent format.
///
/// ODF requires line-height to be a percentage or length. Unitless numbers
/// (e.g., `1.5`) are converted to percent (`150%`).
fn coerce_line_height(key: &str, value: &str) -> String {
    if key != "fo:line-height" {
        return value.to_string();
    }
    let val = value.trim();
    if val.chars().all(|c| c.is_ascii_digit() || c == '.') {
        if let Ok(num) = val.parse::<f32>() {
            return format!("{}%", (num * 100.0).round());
        }
    }
    value.to_string()
}

/// Writes the built-in `PageBreak` style required by Loki documents.
fn write_page_break_style(writer: &mut Writer<Cursor<Vec<u8>>>) -> Result<(), String> {
    let mut pb_style = BytesStart::new("style:style");
    pb_style.push_attribute(("style:name", "PageBreak"));
    pb_style.push_attribute(("style:family", "paragraph"));
    pb_style.push_attribute(("style:parent-style-name", "Standard"));
    writer
        .write_event(Event::Start(pb_style))
        .map_err(|e| e.to_string())?;

    let mut pb_props = BytesStart::new("style:paragraph-properties");
    pb_props.push_attribute(("fo:break-before", "page"));
    writer
        .write_event(Event::Empty(pb_props))
        .map_err(|e| e.to_string())?;

    writer
        .write_event(Event::End(BytesEnd::new("style:style")))
        .map_err(|e| e.to_string())
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
