//! Session document commands.
//!
//! These commands serialise and deserialise ODT documents to/from raw bytes
//! without touching the filesystem. The frontend's `SessionManager` uses them
//! to autosave into the app-data session directory without overwriting the
//! user's original file.

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

use common_core::{LexicalDocument, Metadata, StyleDefinition};
use odt_format::{
    lexical::{from_lexical, to_lexical},
    Document,
};
use serde::Serialize;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

type CommandResult<T> = Result<T, String>;

/// Lexical editor state returned by `deserialize_document`.
#[derive(Serialize)]
pub struct SessionLexicalResponse {
    pub content: LexicalDocument,
    pub styles: HashMap<String, StyleDefinition>,
    pub metadata: Metadata,
}

/// Serialise a Lexical document to ODT bytes without writing to disk.
///
/// Used by the frontend `SessionManager` to produce bytes that are written
/// to the session directory, leaving the user's original file untouched.
#[tauri::command]
pub fn serialize_document(
    lexical_json: String,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
) -> CommandResult<Vec<u8>> {
    let lex: LexicalDocument =
        serde_json::from_str(&lexical_json).map_err(|e| format!("Invalid Lexical JSON: {e}"))?;
    let doc = from_lexical(lex, styles, metadata);

    let mut buf = Cursor::new(Vec::new());
    write_odt_zip(&mut buf, &doc)?;
    Ok(buf.into_inner())
}

/// Deserialise raw ODT bytes into a Lexical editor state.
///
/// Used by the frontend `SessionManager` to restore a previously serialised
/// session file.
#[tauri::command]
pub fn deserialize_document(file_content: Vec<u8>) -> CommandResult<SessionLexicalResponse> {
    let reader = Cursor::new(file_content);

    let doc = if reader.get_ref().starts_with(b"PK") {
        // ZIP-based ODT
        let mut archive =
            zip::ZipArchive::new(reader).map_err(|e| format!("Failed to open ODT zip: {e}"))?;

        let content_xml = {
            let mut f = archive
                .by_name("content.xml")
                .map_err(|e| format!("content.xml missing: {e}"))?;
            let mut s = String::new();
            f.read_to_string(&mut s)
                .map_err(|e| format!("Failed to read content.xml: {e}"))?;
            s
        };

        let mut doc = Document::from_xml(&content_xml)?;

        if let Ok(mut f) = archive.by_name("styles.xml") {
            let mut s = String::new();
            if f.read_to_string(&mut s).is_ok() {
                let _ = doc.add_styles_from_xml(&s);
            }
        }

        doc
    } else {
        // FODT (flat XML)
        let xml = String::from_utf8(reader.into_inner())
            .map_err(|e| format!("Not valid UTF-8: {e}"))?;
        Document::from_xml(&xml)?
    };

    Ok(SessionLexicalResponse {
        content: to_lexical(&doc),
        styles: doc.styles,
        metadata: doc.metadata,
    })
}

fn write_odt_zip<W: Write + std::io::Seek>(writer: W, doc: &Document) -> CommandResult<()> {
    let mut zip = ZipWriter::new(writer);

    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("mimetype", stored)
        .map_err(|e| e.to_string())?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")
        .map_err(|e| e.to_string())?;

    zip.add_directory("META-INF", deflated)
        .map_err(|e| e.to_string())?;
    zip.start_file("META-INF/manifest.xml", deflated)
        .map_err(|e| e.to_string())?;
    zip.write_all(MANIFEST.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("content.xml", deflated)
        .map_err(|e| e.to_string())?;
    zip.write_all(doc.to_content_xml()?.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("styles.xml", deflated)
        .map_err(|e| e.to_string())?;
    zip.write_all(doc.styles_to_xml()?.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.start_file("meta.xml", deflated)
        .map_err(|e| e.to_string())?;
    zip.write_all(doc.to_meta_xml()?.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
 <manifest:file-entry manifest:full-path="/" manifest:version="1.3" manifest:media-type="application/vnd.oasis.opendocument.text"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
