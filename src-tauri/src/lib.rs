use odt_logic::{Document, TiptapNode, StyleDefinition, TiptapResponse, Metadata};
use std::collections::HashMap;
use log::{info, error, debug};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn sync_document(tiptap_json: String, styles: HashMap<String, StyleDefinition>, metadata: Metadata) -> Result<Document, String> {
    info!("Synchronizing document...");
    let json_node: TiptapNode = serde_json::from_str(&tiptap_json).map_err(|e| e.to_string())?;
    Ok(Document::from_tiptap(json_node, styles, metadata))
}

#[tauri::command]
async fn save_document(
    app: tauri::AppHandle,
    path: String,
    tiptap_json: String,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata
) -> Result<Option<Vec<u8>>, String> {
    use tauri::Emitter;
    let _ = app.emit("debug_log", format!("Starting save operation to: {}", path));

    let json_node: TiptapNode = serde_json::from_str(&tiptap_json).map_err(|e| e.to_string())?;
    let doc = odt_logic::Document::from_tiptap(json_node, styles, metadata);
    
    // ODT (ZIP) vs FODT (flat XML)
    if path.ends_with(".odt") {
        let _ = app.emit("debug_log", "Generating XML content...".to_string());
        let xml_content = doc.to_content_xml()?;
        let styles_xml = doc.styles_to_xml()?;
        let meta_xml = doc.to_meta_xml()?;
        
        let _ = app.emit("debug_log", format!("XML generated. Content size: {}", xml_content.len()));

        // 1. Build ZIP in memory first (Atomic Save Strategy)
        let _ = app.emit("debug_log", "Building ZIP in memory...".to_string());
        
        let mut buffer = std::io::Cursor::new(Vec::new());
        write_odt_to_zip(&mut buffer, &xml_content, &styles_xml, &meta_xml).map_err(|e| {
             let _ = app.emit("debug_log", format!("Failed to build ZIP in memory: {}", e));
             e
        })?;

        let zip_data = buffer.into_inner();
        let _ = app.emit("debug_log", format!("ZIP constructed. Total size: {} bytes", zip_data.len()));

        if zip_data.is_empty() {
            let msg = "Internal Error: Generated ZIP is empty.";
            let _ = app.emit("debug_log", msg.to_string());
            return Err(msg.to_string());
        }

        // HYBRID STRATEGY:
        // If content:// URI, return bytes to frontend.
        if path.starts_with("content://") {
             let _ = app.emit("debug_log", "Detected content:// URI. Returning bytes to frontend...".to_string());
             return Ok(Some(zip_data));
        }

        // 2. Write to disk (Standard File System)
        #[cfg(target_os = "android")]
        let use_direct_write = true;
        #[cfg(not(target_os = "android"))]
        let use_direct_write = false;

        if use_direct_write {
            let _ = app.emit("debug_log", format!("Writing {} bytes directly to disk (Android)...", zip_data.len()));
            match std::fs::write(&path, &zip_data) {
                Ok(_) => {
                     // Verify size
                     let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                     let _ = app.emit("debug_log", format!("Success! Written to disk. Verified size: {}", size));
                     if size == 0 {
                         let _ = app.emit("debug_log", "WARNING: Verified size is 0 bytes!".to_string());
                     }
                },
                Err(e) => {
                    let _ = app.emit("debug_log", format!("Write failed: {}", e));
                    return Err(format!("Write failed: {}", e));
                }
            }
        } else {
            // Desktop safe save (Atomic Rename)
            let temp_path = format!("{}.tmp", path);
            let _ = app.emit("debug_log", format!("Writing to temp file: {}", temp_path));
            
            if let Err(e) = std::fs::write(&temp_path, &zip_data) {
                let _ = app.emit("debug_log", format!("Temp write failed: {}", e));
                return Err(e.to_string());
            }

            let _ = app.emit("debug_log", "Renaming temp file to target...".to_string());
            if let Err(e) = std::fs::rename(&temp_path, &path) {
                 let _ = app.emit("debug_log", format!("Rename failed, trying fallback copy... {}", e));
                 if let Err(c_err) = std::fs::copy(&temp_path, &path) {
                     let _ = app.emit("debug_log", format!("Fallback copy failed: {}", c_err));
                     return Err(c_err.to_string());
                 }
            }
            let _ = app.emit("debug_log", "Save complete.".to_string());
        }
    } else {
        // FODT
        let _ = app.emit("debug_log", format!("Saving FODT to: {}", path));
        let xml_content = doc.to_xml()?;
        
        if path.starts_with("content://") {
             let _ = app.emit("debug_log", "Detected content:// URI. Returning bytes to frontend...".to_string());
             return Ok(Some(xml_content.into_bytes()));
        }
        
        std::fs::write(&path, xml_content).map_err(|e| e.to_string())?;
    }

    Ok(None)
}

fn write_odt_to_zip<W: std::io::Write + std::io::Seek>(
    writer: &mut W,
    xml_content: &str,
    styles_xml: &str,
    meta_xml: &str,
) -> Result<(), String> {
    let mut zip = zip::ZipWriter::new(writer);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);
    
    // 1. mimetype (must be first, uncompressed)
    zip.start_file("mimetype", options).map_err(|e| e.to_string())?;
    use std::io::Write;
    zip.write_all(b"application/vnd.oasis.opendocument.text").map_err(|e| e.to_string())?;

    // 2. content.xml (deflated)
    let deflated_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    
    zip.start_file("content.xml", deflated_options).map_err(|e| e.to_string())?;
    zip.write_all(xml_content.as_bytes()).map_err(|e| e.to_string())?;

    // 3. styles.xml
    zip.start_file("styles.xml", deflated_options).map_err(|e| e.to_string())?;
    zip.write_all(styles_xml.as_bytes()).map_err(|e| e.to_string())?;

    // 4. meta.xml
    zip.start_file("meta.xml", deflated_options).map_err(|e| e.to_string())?;
    zip.write_all(meta_xml.as_bytes()).map_err(|e| e.to_string())?;

    // 5. META-INF/manifest.xml
    zip.add_directory("META-INF", options).map_err(|e| e.to_string())?;
    zip.start_file("META-INF/manifest.xml", deflated_options).map_err(|e| e.to_string())?;
    let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
 <manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.text"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
    zip.write_all(manifest.as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| {
        e.to_string()
    })?;
    Ok(())
}

#[tauri::command]
async fn open_document(
    app: tauri::AppHandle,
    path: String,
    file_content: Option<Vec<u8>>,
) -> Result<TiptapResponse, String> {
    use tauri::Emitter;
    let _ = app.emit("debug_log", format!("Opening document: {}", path));

    if let Some(bytes) = &file_content {
        let _ = app.emit("debug_log", format!("Opening from provided memory content ({} bytes)", bytes.len()));
    } else {
        let _ = app.emit("debug_log", "Opening from file system path".to_string());
    }

    // Helper to read from any Read+Seek (File or Cursor)
    fn read_content_from_reader<R: std::io::Read + std::io::Seek>(
        mut reader: R,
    ) -> Result<(String, Option<String>), String> {
        // Try as ZIP (ODT)
        if let Ok(mut archive) = zip::ZipArchive::new(&mut reader) {
            let content = if let Ok(mut content_file) = archive.by_name("content.xml") {
                let mut s = String::new();
                std::io::Read::read_to_string(&mut content_file, &mut s)
                    .map_err(|e| format!("Failed to read content.xml: {}", e))?;
                s
            } else {
                return Err("Invalid ODT: content.xml not found".to_string());
            };

            let styles = if let Ok(mut styles_file) = archive.by_name("styles.xml") {
                let mut s = String::new();
                std::io::Read::read_to_string(&mut styles_file, &mut s)
                    .map_err(|e| format!("Failed to read styles.xml: {}", e))?;
                Some(s)
            } else {
                None
            };
            Ok((content, styles))
        } else {
            // Not a ZIP, rewind and try as raw XML (FODT)
            reader.seek(std::io::SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            let mut s = String::new();
            std::io::Read::read_to_string(&mut reader, &mut s)
                .map_err(|e| format!("Failed to read FODT: {}", e))?;
            Ok((s, None))
        }
    }

    let (xml, styles_xml) = if let Some(bytes) = file_content {
        let cursor = std::io::Cursor::new(bytes);
        read_content_from_reader(cursor)?
    } else {
        let file = std::fs::File::open(&path)
            .map_err(|e| format!("Failed to open file at '{}': {}", path, e))?;
        read_content_from_reader(file)?
    };

    let _ = app.emit("debug_log", "Content parsed successfully. Converting to Tiptap...".to_string());

    let mut doc = Document::from_xml(&xml).map_err(|e| format!("Failed to parse ODF XML: {}", e))?;
    
    if let Some(styles_content) = styles_xml {
        match doc.add_styles_from_xml(&styles_content) {
            Ok(_) => {},
            Err(e) => {
                let _ = app.emit("debug_log", format!("Warning: Failed to parse styles.xml: {}", e));
            }
        }
    }
    
    Ok(TiptapResponse {
        content: doc.to_tiptap(),
        styles: doc.styles,
        metadata: doc.metadata,
    })
}

#[tauri::command]
async fn save_epub(
    app: tauri::AppHandle,
    path: String,
    tiptap_json: String,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
    font_paths: Vec<String>,
) -> Result<Option<Vec<u8>>, String> {
    use tauri::Emitter;
    let _ = app.emit("debug_log", format!("Exporting EPUB to: {}", path));

    let json_node: TiptapNode = serde_json::from_str(&tiptap_json).map_err(|e| e.to_string())?;
    // Load fonts
    let mut fonts = Vec::new();
    for font_path in font_paths {
        if let Ok(data) = std::fs::read(&font_path) {
            let filename = std::path::Path::new(&font_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("font.ttf")
                .to_string();
            
            // Extract family name from filename (e.g., "CourierPrime-Regular.ttf" -> "Courier Prime")
            let family_name = filename
                .split('.')
                .next()
                .unwrap_or("")
                .split('-')
                .next()
                .unwrap_or("Unknown")
                .to_string();
            
            fonts.push(epub_logic::FontAsset {
                family_name,
                filename,
                data,
                format: epub_logic::FontFormat::from_filename(&font_path),
            });
        }
    }
    
    // Create EPUB document
    let epub_doc = epub_logic::EpubDocument::from_tiptap(json_node, styles, metadata, fonts);
    
    // Helper to write EPUB to any Write+Seek
    fn write_epub_zip<W: std::io::Write + std::io::Seek>(
        writer: W,
        epub_doc: &epub_logic::EpubDocument
    ) -> Result<(), String> {
        use std::io::Write; // Import Write trait
        let mut zip_writer = zip::ZipWriter::new(writer);
        
        // 1. mimetype (MUST be first, uncompressed)
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip_writer.start_file("mimetype", options).map_err(|e| e.to_string())?;
        zip_writer.write_all(b"application/epub+zip").map_err(|e| e.to_string())?;
        
        // 2. META-INF/container.xml
        let deflated_options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
            
        zip_writer.add_directory("META-INF", options).map_err(|e| e.to_string())?;
        zip_writer.start_file("META-INF/container.xml", deflated_options).map_err(|e| e.to_string())?;
        let container = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
        zip_writer.write_all(container.as_bytes()).map_err(|e| e.to_string())?;
        
        // 3. EPUB/package.opf
        zip_writer.add_directory("OEBPS", options).map_err(|e| e.to_string())?;
        zip_writer.start_file("OEBPS/content.opf", deflated_options).map_err(|e| e.to_string())?;
        zip_writer.write_all(epub_doc.to_package_opf().as_bytes()).map_err(|e| e.to_string())?;
        
        // 4. OEBPS/nav.xhtml
        zip_writer.start_file("OEBPS/nav.xhtml", deflated_options).map_err(|e| e.to_string())?;
        zip_writer.write_all(epub_doc.to_nav_xhtml().as_bytes()).map_err(|e| e.to_string())?;
        
        // 5. OEBPS/Text/... (Content) - Reverting to loop logic!
        zip_writer.add_directory("OEBPS/Text", options).map_err(|e| e.to_string())?;
        for section in &epub_doc.sections {
            let filename = format!("OEBPS/Text/{}.xhtml", section.id);
            zip_writer.start_file(&filename, deflated_options).map_err(|e| e.to_string())?;
            zip_writer.write_all(epub_doc.section_to_xhtml(section).as_bytes()).map_err(|e| e.to_string())?;
        }
        
        // 6. OEBPS/Styles/styles.css
        zip_writer.add_directory("OEBPS/Styles", options).map_err(|e| e.to_string())?;
        zip_writer.start_file("OEBPS/Styles/styles.css", deflated_options).map_err(|e| e.to_string())?;
        zip_writer.write_all(epub_doc.to_css().as_bytes()).map_err(|e| e.to_string())?;
        
        // 7. OEBPS/Fonts
        if !epub_doc.fonts.is_empty() {
            zip_writer.add_directory("OEBPS/Fonts", options).map_err(|e| e.to_string())?;
            for font in &epub_doc.fonts {
                zip_writer.start_file(format!("OEBPS/Fonts/{}", font.filename), deflated_options).map_err(|e| e.to_string())?;
                zip_writer.write_all(&font.data).map_err(|e| e.to_string())?;
            }
        }
        
        zip_writer.finish().map_err(|e| e.to_string())?;
        Ok(())
    }

    if path.starts_with("content://") {
        let _ = app.emit("debug_log", "Detected content:// URI. Generating EPUB in memory...".to_string());
        let mut buffer = std::io::Cursor::new(Vec::new());
        write_epub_zip(&mut buffer, &epub_doc)?;
        let bytes = buffer.into_inner();
        return Ok(Some(bytes));
    } else {
        let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        write_epub_zip(file, &epub_doc)?;
        Ok(None)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("DEBUG: run() starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            sync_document,
            save_document,
            open_document,
            save_epub
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
