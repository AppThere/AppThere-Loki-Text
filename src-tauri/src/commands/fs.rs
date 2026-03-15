use common_core::{LexicalDocument, Metadata, StyleDefinition};
use odt_format::{
    lexical::{from_lexical, to_lexical},
    Document,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
};
use tauri::{AppHandle, Emitter, Runtime};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use super::odt_zip::write_odt_zip;

/// Response payload for `open_document`: Lexical editor state + styles + metadata.
#[derive(Serialize)]
pub struct LexicalResponse {
    pub content: LexicalDocument,
    pub styles: HashMap<String, StyleDefinition>,
    pub metadata: Metadata,
}

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn save_document<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    lexical_json: String,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
    original_path: Option<String>,
    original_content: Option<Vec<u8>>,
) -> CommandResult<Option<Vec<u8>>> {
    app.emit("debug_log", format!("Saving document to {}", path))
        .ok();

    // Deserialize Lexical editor state
    let lex_doc: LexicalDocument =
        serde_json::from_str(&lexical_json).map_err(|e| format!("Invalid Lexical JSON: {}", e))?;

    let doc = from_lexical(lex_doc, styles, metadata);

    let mut original_bytes: Option<Vec<u8>> = original_content;
    if original_bytes.is_none() {
        if let Some(ref orig_path) = original_path {
            if !orig_path.starts_with("content://") {
                original_bytes = std::fs::read(orig_path).ok();
            }
        }
    }

    let bytes = if path.ends_with(".fodt") {
        if let Some(orig_bytes) = original_bytes {
            if let Ok(orig_xml) = String::from_utf8(orig_bytes) {
                if let Ok(updated) = doc.update_fodt(&orig_xml) {
                    updated.into_bytes()
                } else {
                    doc.to_xml()?.into_bytes()
                }
            } else {
                doc.to_xml()?.into_bytes()
            }
        } else {
            doc.to_xml()?.into_bytes()
        }
    } else {
        // ODT Generation (ZIP)
        let mut buffer = Cursor::new(Vec::new());
        if let Some(orig_bytes) = original_bytes {
            if update_odt_zip(&orig_bytes, &mut buffer, &doc).is_ok() {
                // Success
            } else {
                buffer = Cursor::new(Vec::new()); // Reset buffer
                write_odt_zip(&mut buffer, &doc)?;
            }
        } else {
            write_odt_zip(&mut buffer, &doc)?;
        }
        buffer.into_inner()
    };

    if path.starts_with("content://") {
        app.emit(
            "debug_log",
            format!("Returning {} bytes to frontend for writing", bytes.len()),
        )
        .ok();
        Ok(Some(bytes))
    } else {
        std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
        app.emit(
            "debug_log",
            format!("Saved {} bytes to disk: {}", bytes.len(), path),
        )
        .ok();
        Ok(None)
    }
}

fn update_odt_zip<W: Write + std::io::Seek>(
    old_bytes: &[u8],
    writer: W,
    doc: &Document,
) -> Result<(), String> {
    let reader = Cursor::new(old_bytes.to_vec());
    let mut zip_in = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;
    let mut zip_out = ZipWriter::new(writer);

    let options_mimetype =
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let options_deflated =
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    // Write mimetype first (uncompressed) if it exists
    for i in 0..zip_in.len() {
        let mut file = zip_in.by_index(i).map_err(|e| e.to_string())?;
        if file.name() == "mimetype" {
            zip_out
                .start_file("mimetype", options_mimetype)
                .map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut zip_out).map_err(|e| e.to_string())?;
            break;
        }
    }

    // Write the rest
    for i in 0..zip_in.len() {
        let mut file = zip_in.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        if name == "mimetype" {
            continue;
        }

        zip_out
            .start_file(&name, options_deflated)
            .map_err(|e| e.to_string())?;

        if name == "content.xml" {
            let mut original = String::new();
            if file.read_to_string(&mut original).is_ok() {
                if let Ok(updated) = doc.update_fodt(&original) {
                    zip_out
                        .write_all(updated.as_bytes())
                        .map_err(|e| e.to_string())?;
                } else {
                    zip_out
                        .write_all(doc.to_content_xml()?.as_bytes())
                        .map_err(|e| e.to_string())?;
                }
            } else {
                zip_out
                    .write_all(doc.to_content_xml()?.as_bytes())
                    .map_err(|e| e.to_string())?;
            }
        } else if name == "styles.xml" {
            let mut original = String::new();
            if file.read_to_string(&mut original).is_ok() {
                if let Ok(updated) = doc.update_fodt(&original) {
                    zip_out
                        .write_all(updated.as_bytes())
                        .map_err(|e| e.to_string())?;
                } else {
                    zip_out
                        .write_all(doc.styles_to_xml()?.as_bytes())
                        .map_err(|e| e.to_string())?;
                }
            } else {
                zip_out
                    .write_all(doc.styles_to_xml()?.as_bytes())
                    .map_err(|e| e.to_string())?;
            }
        } else if name == "meta.xml" {
            zip_out
                .write_all(doc.to_meta_xml()?.as_bytes())
                .map_err(|e| e.to_string())?;
        } else {
            std::io::copy(&mut file, &mut zip_out).map_err(|e| e.to_string())?;
        }
    }

    zip_out.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_document<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    file_content: Option<Vec<u8>>,
) -> CommandResult<LexicalResponse> {
    app.emit("debug_log", format!("Opening document: {}", path))
        .ok();

    let bytes = if let Some(content) = file_content {
        app.emit("debug_log", "Using provided file content (Memory)")
            .ok();
        content
    } else {
        app.emit("debug_log", "Reading file from disk").ok();
        std::fs::read(&path).map_err(|e| {
            format!(
                "Failed to read file {}: {}. Tip: On Android, ensure file_content is passed for content:// URIs.",
                path, e
            )
        })?
    };

    let doc = if bytes.starts_with(b"PK") {
        // Zip archive (ODT)
        let reader = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(reader)
            .map_err(|e| format!("Navalozh: Failed to read zip archive: {}", e))?;

        // 1. Read content.xml (Body and Automatic Styles)
        let content_xml = {
            let mut content_file = archive
                .by_name("content.xml")
                .map_err(|e| format!("Navalozh: content.xml not found in ODT: {}", e))?;
            let mut content_xml = String::new();
            content_file
                .read_to_string(&mut content_xml)
                .map_err(|e| format!("Navalozh: Failed to read content.xml: {}", e))?;
            content_xml
        };

        let mut doc = Document::from_xml(&content_xml)?;

        // 2. Read styles.xml (Common Styles)
        {
            if let Ok(mut styles_file) = archive.by_name("styles.xml") {
                let mut styles_xml = String::new();
                if styles_file.read_to_string(&mut styles_xml).is_ok() {
                    let _ = doc.add_styles_from_xml(&styles_xml);
                }
            }
        }

        // 3. Read meta.xml (Metadata)
        {
            if let Ok(mut meta_file) = archive.by_name("meta.xml") {
                let mut meta_xml = String::new();
                if meta_file.read_to_string(&mut meta_xml).is_ok() {
                    if let Ok(meta_doc) = Document::from_xml(&meta_xml) {
                        if meta_doc.metadata.title.is_some() {
                            doc.metadata.title = meta_doc.metadata.title;
                        }
                        if meta_doc.metadata.creator.is_some() {
                            doc.metadata.creator = meta_doc.metadata.creator;
                        }
                        if meta_doc.metadata.description.is_some() {
                            doc.metadata.description = meta_doc.metadata.description;
                        }
                        if meta_doc.metadata.subject.is_some() {
                            doc.metadata.subject = meta_doc.metadata.subject;
                        }
                        if meta_doc.metadata.creation_date.is_some() {
                            doc.metadata.creation_date = meta_doc.metadata.creation_date;
                        }
                        if meta_doc.metadata.generator.is_some() {
                            doc.metadata.generator = meta_doc.metadata.generator;
                        }
                        if meta_doc.metadata.identifier.is_some() {
                            doc.metadata.identifier = meta_doc.metadata.identifier;
                        }
                        if meta_doc.metadata.language.is_some() {
                            doc.metadata.language = meta_doc.metadata.language;
                        }
                    }
                }
            }
        }

        doc
    } else {
        // Plain text / XML (FODT)
        let xml_content = String::from_utf8(bytes)
            .map_err(|e| format!("Navalozh: Failed to decode text file (not UTF-8): {}", e))?;
        Document::from_xml(&xml_content)?
    };

    Ok(LexicalResponse {
        content: to_lexical(&doc),
        styles: doc.styles,
        metadata: doc.metadata,
    })
}
