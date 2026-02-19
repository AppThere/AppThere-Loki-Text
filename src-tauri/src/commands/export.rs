use odt_logic::{Metadata, StyleDefinition, TiptapNode};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Runtime};

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn save_epub<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    tiptap_json: String,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
    font_paths: Vec<String>,
) -> CommandResult<Option<Vec<u8>>> {
    app.emit("debug_log", format!("Exporting EPUB to: {}", path)).ok();

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
    
    // Write EPUB
    if path.starts_with("content://") {
        app.emit("debug_log", "Detected content:// URI. Generating EPUB in memory...".to_string()).ok();
        let mut buffer = std::io::Cursor::new(Vec::new());
        write_epub_zip(&mut buffer, &epub_doc)?;
        let bytes = buffer.into_inner();
        Ok(Some(bytes))
    } else {
        let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        write_epub_zip(file, &epub_doc)?;
        Ok(None)
    }
}

fn write_epub_zip<W: std::io::Write + std::io::Seek>(
    writer: W,
    epub_doc: &epub_logic::EpubDocument
) -> Result<(), String> {
    use std::io::Write;
    let mut zip_writer = zip::ZipWriter::new(writer);
    
    // 1. mimetype (MUST be first, uncompressed)
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);

    zip_writer.start_file("mimetype", options).map_err(|e| e.to_string())?;
    zip_writer.write_all(b"application/epub+zip").map_err(|e| e.to_string())?;
    
    // 2. META-INF/container.xml
    let deflated_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
        
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
    
    // 5. OEBPS/Text/... (Content)
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
