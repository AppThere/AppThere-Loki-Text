use odt_logic::{Document, Metadata, StyleDefinition, TiptapNode, TiptapResponse};
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
};
use tauri::{AppHandle, Emitter, Runtime};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn save_document<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    tiptap_json: String,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
) -> CommandResult<Option<Vec<u8>>> {
    app.emit("debug_log", format!("Saving document to {}", path))
        .ok();

    // Deserialize Tiptap JSON
    let json_node: TiptapNode =
        serde_json::from_str(&tiptap_json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let doc = Document::from_tiptap(json_node, styles, metadata);

    let bytes = if path.ends_with(".fodt") {
        doc.to_xml()?.into_bytes()
    } else {
        // ODT Generation (ZIP)
        let mut buffer = Cursor::new(Vec::new());
        write_odt_zip(&mut buffer, &doc)?;
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

fn write_odt_zip<W: Write + std::io::Seek>(writer: W, doc: &Document) -> Result<(), String> {
    let mut zip = ZipWriter::new(writer);

    // 1. mimetype (MUST be first, uncompressed)
    let options_mimetype = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    zip.start_file("mimetype", options_mimetype)
        .map_err(|e| e.to_string())?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")
        .map_err(|e| e.to_string())?;

    // 2. META-INF/manifest.xml
    let options_deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.add_directory("META-INF", options_deflated)
        .map_err(|e| e.to_string())?;
    zip.start_file("META-INF/manifest.xml", options_deflated)
        .map_err(|e| e.to_string())?;
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
 <manifest:file-entry manifest:full-path="/" manifest:version="1.3" manifest:media-type="application/vnd.oasis.opendocument.text"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
    zip.write_all(manifest.as_bytes())
        .map_err(|e| e.to_string())?;

    // 3. content.xml
    zip.start_file("content.xml", options_deflated)
        .map_err(|e| e.to_string())?;
    zip.write_all(doc.to_content_xml()?.as_bytes())
        .map_err(|e| e.to_string())?;

    // 4. styles.xml
    zip.start_file("styles.xml", options_deflated)
        .map_err(|e| e.to_string())?;
    zip.write_all(doc.styles_to_xml()?.as_bytes())
        .map_err(|e| e.to_string())?;

    // 5. meta.xml
    zip.start_file("meta.xml", options_deflated)
        .map_err(|e| e.to_string())?;
    zip.write_all(doc.to_meta_xml()?.as_bytes())
        .map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_document<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    file_content: Option<Vec<u8>>,
) -> CommandResult<TiptapResponse> {
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
        let xml_content = String::from_utf8(bytes).map_err(|e| {
            format!("Navalozh: Failed to decode text file (not UTF-8): {}", e)
        })?;
        Document::from_xml(&xml_content)?
    };

    Ok(TiptapResponse {
        content: doc.to_tiptap(),
        styles: doc.styles,
        metadata: doc.metadata,
    })
}
