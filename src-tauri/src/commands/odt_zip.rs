//! Canonical ODT ZIP writer shared by the fs and session command modules.

use std::io::{Write};

use odt_format::Document;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.3">
 <manifest:file-entry manifest:full-path="/" manifest:version="1.3" manifest:media-type="application/vnd.oasis.opendocument.text"/>
 <manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
 <manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
</manifest:manifest>"#;

/// Write a complete ODT ZIP archive to `writer`.
///
/// Entry order follows the ODF specification:
/// 1. `mimetype` — uncompressed, must be first
/// 2. `META-INF/manifest.xml` — deflated
/// 3. `content.xml` — deflated
/// 4. `styles.xml` — deflated
/// 5. `meta.xml` — deflated
pub fn write_odt_zip<W: Write + std::io::Seek>(
    writer: W,
    doc: &Document,
) -> Result<(), String> {
    let mut zip = ZipWriter::new(writer);

    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    // 1. mimetype (MUST be first, uncompressed)
    zip.start_file("mimetype", stored).map_err(|e| e.to_string())?;
    zip.write_all(b"application/vnd.oasis.opendocument.text")
        .map_err(|e| e.to_string())?;

    // 2. META-INF/manifest.xml
    zip.add_directory("META-INF", deflated).map_err(|e| e.to_string())?;
    zip.start_file("META-INF/manifest.xml", deflated).map_err(|e| e.to_string())?;
    zip.write_all(MANIFEST.as_bytes()).map_err(|e| e.to_string())?;

    // 3. content.xml
    zip.start_file("content.xml", deflated).map_err(|e| e.to_string())?;
    zip.write_all(doc.to_content_xml()?.as_bytes()).map_err(|e| e.to_string())?;

    // 4. styles.xml
    zip.start_file("styles.xml", deflated).map_err(|e| e.to_string())?;
    zip.write_all(doc.styles_to_xml()?.as_bytes()).map_err(|e| e.to_string())?;

    // 5. meta.xml
    zip.start_file("meta.xml", deflated).map_err(|e| e.to_string())?;
    zip.write_all(doc.to_meta_xml()?.as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}
