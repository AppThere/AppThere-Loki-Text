use odt_logic::{Document, TiptapNode, StyleDefinition, TiptapResponse, Metadata};
use std::collections::HashMap;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn sync_document(tiptap_json: TiptapNode, styles: HashMap<String, StyleDefinition>, metadata: Metadata) -> Document {
    Document::from_tiptap(tiptap_json, styles, metadata)
}

#[tauri::command]
async fn save_document(path: String, tiptap_json: TiptapNode, styles: HashMap<String, StyleDefinition>, metadata: Metadata    ) -> Result<(), String> {
    let doc = odt_logic::Document::from_tiptap(tiptap_json, styles, metadata);
    
    // ODT (ZIP) vs FODT (flat XML) - use different XML generation methods
    if path.ends_with(".odt") {
        // Use proper ODT format with office:document-content
        let xml_content = doc.to_content_xml()?;
        let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o755);
        
        // 1. mimetype (must be first, uncompressed)
        zip.start_file("mimetype", options).map_err(|e| e.to_string())?;
        use std::io::Write;
        zip.write_all(b"application/vnd.oasis.opendocument.text").map_err(|e| e.to_string())?;

        // 2. content.xml (deflated)
        let deflated_options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o755);
        
        zip.start_file("content.xml", deflated_options).map_err(|e| e.to_string())?;
        zip.write_all(xml_content.as_bytes()).map_err(|e| e.to_string())?;

        // 3. styles.xml
        let styles_xml = doc.styles_to_xml()?;
        zip.start_file("styles.xml", deflated_options).map_err(|e| e.to_string())?;
        zip.write_all(styles_xml.as_bytes()).map_err(|e| e.to_string())?;

        // 4. meta.xml (required for valid ODT)
        let meta_xml = doc.to_meta_xml()?;
        zip.start_file("meta.xml", deflated_options).map_err(|e| e.to_string())?;
        zip.write_all(meta_xml.as_bytes()).map_err(|e| e.to_string())?;

        // 5. META-INF/manifest.xml
        zip.add_directory("META-INF", options).map_err(|e| e.to_string())?;
        zip.start_file("META-INF/manifest.xml", deflated_options).map_err(|e| e.to_string())?;
        let manifest = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
 <manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.text"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;
        zip.write_all(manifest.as_bytes()).map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;
    } else {
        // Fallback to FODT (flat XML) - use office:document format
        let xml_content = doc.to_xml()?;
        std::fs::write(&path, xml_content).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn open_document(path: String) -> Result<TiptapResponse, String> {
    let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
    
    // Try to open as zip
    let (xml, styles_xml) = if let Ok(mut archive) = zip::ZipArchive::new(&file) {
        let content = if let Ok(mut content_file) = archive.by_name("content.xml") {
            let mut s = String::new();
            use std::io::Read;
            content_file.read_to_string(&mut s).map_err(|e| e.to_string())?;
            s
        } else {
             return Err("Invalid ODT: content.xml not found".to_string());
        };

        let styles = if let Ok(mut styles_file) = archive.by_name("styles.xml") {
            let mut s = String::new();
            use std::io::Read;
            styles_file.read_to_string(&mut s).map_err(|e| e.to_string())?;
            Some(s)
        } else {
            None
        };

        (content, styles)
    } else {
        // Not a zip, try reading as raw XML (FODT)
        (std::fs::read_to_string(&path).map_err(|e| e.to_string())?, None)
    };

    let mut doc = Document::from_xml(&xml)?;
    
    if let Some(styles_content) = styles_xml {
        match doc.add_styles_from_xml(&styles_content) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Warning: Failed to parse styles.xml: {}", e);
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
    path: String,
    tiptap_json: TiptapNode,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
    font_paths: Vec<String>,
) -> Result<(), String> {
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
    let epub_doc = epub_logic::EpubDocument::from_tiptap(tiptap_json, styles, metadata, fonts);
    
    // Create ZIP archive
    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    
    // 1. mimetype (MUST be first, uncompressed)
    let options = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Stored);
    
    zip.start_file("mimetype", options).map_err(|e| e.to_string())?;
    use std::io::Write;
    zip.write_all(b"application/epub+zip").map_err(|e| e.to_string())?;
    
    // 2. META-INF/container.xml
    let deflated = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    zip.add_directory("META-INF", options).map_err(|e| e.to_string())?;
    zip.start_file("META-INF/container.xml", deflated).map_err(|e| e.to_string())?;
    let container = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
    zip.write_all(container.as_bytes()).map_err(|e| e.to_string())?;
    
    // 2.1 META-INF/com.apple.ibooks.display-options.xml (Apple Books Font workaround)
    zip.start_file("META-INF/com.apple.ibooks.display-options.xml", deflated).map_err(|e| e.to_string())?;
    let apple_options = r#"<?xml version="1.0" encoding="UTF-8"?>
<display_options>
  <platform name="*">
    <option name="specified-fonts">true</option>
  </platform>
</display_options>"#;
    zip.write_all(apple_options.as_bytes()).map_err(|e| e.to_string())?;
    
    // 3. OEBPS directory structure
    zip.add_directory("OEBPS", options).map_err(|e| e.to_string())?;
    zip.add_directory("OEBPS/Text", options).map_err(|e| e.to_string())?;
    zip.add_directory("OEBPS/Styles", options).map_err(|e| e.to_string())?;
    if !epub_doc.fonts.is_empty() {
        zip.add_directory("OEBPS/Fonts", options).map_err(|e| e.to_string())?;
    }
    
    // 4. content.opf
    zip.start_file("OEBPS/content.opf", deflated).map_err(|e| e.to_string())?;
    zip.write_all(epub_doc.to_package_opf().as_bytes()).map_err(|e| e.to_string())?;
    
    // 5. nav.xhtml
    zip.start_file("OEBPS/nav.xhtml", deflated).map_err(|e| e.to_string())?;
    zip.write_all(epub_doc.to_nav_xhtml().as_bytes()).map_err(|e| e.to_string())?;
    
    // 6. styles.css
    zip.start_file("OEBPS/Styles/styles.css", deflated).map_err(|e| e.to_string())?;
    zip.write_all(epub_doc.to_css().as_bytes()).map_err(|e| e.to_string())?;
    
    // 7. Content sections
    for section in &epub_doc.sections {
        let filename = format!("OEBPS/Text/{}.xhtml", section.id);
        zip.start_file(&filename, deflated).map_err(|e| e.to_string())?;
        zip.write_all(epub_doc.section_to_xhtml(section).as_bytes()).map_err(|e| e.to_string())?;
    }
    
    // 8. Fonts
    for font in &epub_doc.fonts {
        let filename = format!("OEBPS/Fonts/{}", font.filename);
        zip.start_file(&filename, deflated).map_err(|e| e.to_string())?;
        zip.write_all(&font.data).map_err(|e| e.to_string())?;
    }
    
    zip.finish().map_err(|e| e.to_string())?;
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
