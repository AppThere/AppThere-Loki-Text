//! ODT `meta.xml` writer.
//!
//! Generates `meta.xml` (or the `<office:meta>` section in FODT) from a
//! document's [`Metadata`] struct.

use std::io::Cursor;

use common_core::Metadata;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

/// Generates a standalone `meta.xml` document string.
///
/// # Errors
///
/// Returns a `String` error if XML writing fails.
pub fn to_meta_xml(metadata: &Metadata) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| e.to_string())?;

    let mut doc_meta = BytesStart::new("office:document-meta");
    doc_meta.push_attribute(("xmlns:office", "urn:oasis:names:tc:opendocument:xmlns:office:1.0"));
    doc_meta.push_attribute(("xmlns:meta", "urn:oasis:names:tc:opendocument:xmlns:meta:1.0"));
    doc_meta.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
    doc_meta.push_attribute(("xmlns:xlink", "http://www.w3.org/1999/xlink"));
    doc_meta.push_attribute(("xmlns:grddl", "http://www.w3.org/2003/g/data-view#"));
    doc_meta.push_attribute(("office:version", "1.3"));
    writer.write_event(Event::Start(doc_meta)).map_err(|e| e.to_string())?;

    writer
        .write_event(Event::Start(BytesStart::new("office:meta")))
        .map_err(|e| e.to_string())?;

    write_meta_elements(&mut writer, metadata)?;

    writer.write_event(Event::End(BytesEnd::new("office:meta"))).map_err(|e| e.to_string())?;
    writer
        .write_event(Event::End(BytesEnd::new("office:document-meta")))
        .map_err(|e| e.to_string())?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.to_string())
}

/// Writes the individual `<dc:*>` and `<meta:*>` child elements.
///
/// Called by both [`to_meta_xml`] and the FODT writer to avoid duplication.
pub fn write_meta_elements(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    metadata: &Metadata,
) -> Result<(), String> {
    write_text_element(writer, "meta:generator", metadata.generator.as_deref().unwrap_or("AppThere Loki"))?;

    if let Some(title) = &metadata.title {
        write_text_element(writer, "dc:title", title)?;
    }
    if let Some(desc) = &metadata.description {
        write_text_element(writer, "dc:description", desc)?;
    }
    if let Some(subject) = &metadata.subject {
        write_text_element(writer, "dc:subject", subject)?;
    }
    if let Some(creator) = &metadata.creator {
        write_text_element(writer, "dc:creator", creator)?;
    }
    if let Some(date) = &metadata.creation_date {
        write_text_element(writer, "meta:creation-date", date)?;
    }
    if let Some(id) = &metadata.identifier {
        write_text_element(writer, "dc:identifier", id)?;
    }
    if let Some(lang) = &metadata.language {
        write_text_element(writer, "dc:language", lang)?;
    }

    Ok(())
}

/// Writes a single XML element with a text node: `<tag>text</tag>`.
fn write_text_element(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    tag: &str,
    text: &str,
) -> Result<(), String> {
    writer.write_event(Event::Start(BytesStart::new(tag))).map_err(|e| e.to_string())?;
    writer.write_event(Event::Text(BytesText::new(text))).map_err(|e| e.to_string())?;
    writer.write_event(Event::End(BytesEnd::new(tag))).map_err(|e| e.to_string())
}
