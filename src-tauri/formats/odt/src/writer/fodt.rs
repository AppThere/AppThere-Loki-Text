//! FODT (flat ODT) writer and in-place updater.
//!
//! Provides [`to_xml`] to generate a complete single-file FODT document, and
//! [`update_fodt`] to update the body, meta, and styles sections of an
//! existing FODT file while preserving all other content verbatim.

use std::io::Cursor;

use common_core::{Block, Metadata};
use common_core::StyleDefinition;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::collections::HashMap;

use crate::writer::blocks::write_blocks;
use crate::writer::meta::write_meta_elements;
use crate::writer::namespaces::push_fodt_ns;
use crate::writer::styles_writer::write_styles_section;

/// Generates a complete FODT (flat XML ODT) document string.
///
/// Writes all sections: meta, font-face-decls, styles, automatic-styles,
/// master-styles, and body content.
///
/// # Errors
///
/// Returns a `String` error if XML writing fails.
pub fn to_xml(
    blocks: &[Block],
    styles: &HashMap<String, StyleDefinition>,
    metadata: &Metadata,
    font_face_decls: &Option<String>,
    automatic_styles: &Option<String>,
    master_styles: &Option<String>,
) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| e.to_string())?;

    let mut document = BytesStart::new("office:document");
    push_fodt_ns(&mut document);
    writer.write_event(Event::Start(document)).map_err(|e| e.to_string())?;

    // Write <office:meta>
    writer
        .write_event(Event::Start(BytesStart::new("office:meta")))
        .map_err(|e| e.to_string())?;
    write_meta_elements(&mut writer, metadata)?;
    writer
        .write_event(Event::End(BytesEnd::new("office:meta")))
        .map_err(|e| e.to_string())?;

    // Write preserved <office:font-face-decls>
    write_preserved(&mut writer, font_face_decls)?;

    // Write <office:styles>
    write_styles_section(&mut writer, styles)?;

    // Write preserved <office:automatic-styles>
    write_preserved_or_empty(&mut writer, automatic_styles, "office:automatic-styles")?;

    // Write preserved <office:master-styles>
    write_preserved(&mut writer, master_styles)?;

    // Write document body
    writer
        .write_event(Event::Start(BytesStart::new("office:body")))
        .map_err(|e| e.to_string())?;
    writer
        .write_event(Event::Start(BytesStart::new("office:text")))
        .map_err(|e| e.to_string())?;

    write_blocks(blocks, &mut writer)?;

    writer
        .write_event(Event::End(BytesEnd::new("office:text")))
        .map_err(|e| e.to_string())?;
    writer
        .write_event(Event::End(BytesEnd::new("office:body")))
        .map_err(|e| e.to_string())?;
    writer
        .write_event(Event::End(BytesEnd::new("office:document")))
        .map_err(|e| e.to_string())?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.to_string())
}

/// Updates the body, meta, and styles sections of an existing FODT file.
///
/// Streams the existing XML, replacing `office:text`, `office:meta`, and
/// `office:styles` content with freshly generated content while preserving
/// all other XML verbatim.
///
/// # Errors
///
/// Returns a `String` error if XML parsing or writing fails.
pub fn update_fodt(
    old_xml: &str,
    blocks: &[Block],
    styles: &HashMap<String, StyleDefinition>,
    metadata: &Metadata,
    content_xml: &str,
    styles_xml: &str,
    meta_xml: &str,
) -> Result<String, String> {
    let mut reader = Reader::from_str(old_xml);
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    let mut buf = Vec::new();
    let mut skip_depth = 0;
    let mut in_styles = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e))
                if e.name().as_ref() == b"office:text" && skip_depth == 0 =>
            {
                writer.write_event(Event::Start(e.clone())).map_err(|err| err.to_string())?;
                inject_inner_xml(&mut writer, content_xml, "<office:text>", "</office:text>")?;
                skip_depth = 1;
            }
            Ok(Event::Start(ref e))
                if e.name().as_ref() == b"office:meta" && skip_depth == 0 =>
            {
                writer.write_event(Event::Start(e.clone())).map_err(|err| err.to_string())?;
                inject_inner_xml(&mut writer, meta_xml, "<office:meta>", "</office:meta>")?;
                skip_depth = 1;
            }
            Ok(Event::Start(ref e))
                if e.name().as_ref() == b"office:styles" && skip_depth == 0 =>
            {
                writer.write_event(Event::Start(e.clone())).map_err(|err| err.to_string())?;
                inject_inner_xml(&mut writer, styles_xml, "<office:styles>", "</office:styles>")?;
                skip_depth = 1;
                in_styles = true;
            }
            Ok(Event::Start(ref _e)) if skip_depth > 0 => skip_depth += 1,
            Ok(Event::Empty(ref _e)) if skip_depth > 0 => {}
            Ok(Event::End(e)) if skip_depth > 0 => {
                skip_depth -= 1;
                if skip_depth == 0 {
                    writer.write_event(Event::End(e)).map_err(|err| err.to_string())?;
                    in_styles = false;
                }
            }
            Ok(Event::Eof) => break,
            Ok(e) => {
                if skip_depth == 0 {
                    writer.write_event(e).map_err(|err| err.to_string())?;
                }
            }
            Err(e) => return Err(e.to_string()),
        }
        buf.clear();
    }
    let _ = (blocks, styles, metadata, in_styles); // used indirectly

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.to_string())
}

/// Extracts the inner XML between `open_tag` and `close_tag` and streams it.
fn inject_inner_xml(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    xml: &str,
    open_tag: &str,
    close_tag: &str,
) -> Result<(), String> {
    if let Some(start_idx) = xml.find(open_tag) {
        if let Some(end_idx) = xml.rfind(close_tag) {
            let inner_xml = &xml[start_idx + open_tag.len()..end_idx];
            let mut inner_reader = Reader::from_str(inner_xml);
            let mut inner_buf = Vec::new();
            loop {
                match inner_reader.read_event_into(&mut inner_buf) {
                    Ok(Event::Eof) => break,
                    Ok(event) => writer.write_event(event).map_err(|e| e.to_string())?,
                    Err(e) => return Err(e.to_string()),
                }
                inner_buf.clear();
            }
        }
    }
    Ok(())
}

/// Writes a preserved raw XML section verbatim.
fn write_preserved(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    content: &Option<String>,
) -> Result<(), String> {
    if let Some(ref xml) = content {
        writer
            .write_event(Event::Text(BytesText::from_escaped(xml)))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Writes a preserved raw XML section, or an empty element if absent.
fn write_preserved_or_empty(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    content: &Option<String>,
    tag: &str,
) -> Result<(), String> {
    if let Some(ref xml) = content {
        writer
            .write_event(Event::Text(BytesText::from_escaped(xml)))
            .map_err(|e| e.to_string())?;
    } else {
        writer.write_event(Event::Start(BytesStart::new(tag))).map_err(|e| e.to_string())?;
        writer.write_event(Event::End(BytesEnd::new(tag))).map_err(|e| e.to_string())?;
    }
    Ok(())
}
