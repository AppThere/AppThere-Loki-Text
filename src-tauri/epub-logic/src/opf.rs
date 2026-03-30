use common_core::Metadata;

use crate::{html::escape_xml, ContentSection, FontAsset, ImageAsset};

/// Generate the OPF 3.0 package document (content.opf).
pub(crate) fn generate_package_opf(
    metadata: &Metadata,
    sections: &[ContentSection],
    fonts: &[FontAsset],
    images: &[ImageAsset],
) -> String {
    let mut opf = String::new();

    opf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    opf.push_str(
        "<package xmlns=\"http://www.idpf.org/2007/opf\" \
         version=\"3.0\" unique-identifier=\"uuid\">\n",
    );

    // ---- Metadata ----
    opf.push_str("  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n");

    let identifier = metadata
        .identifier
        .clone()
        .unwrap_or_else(|| format!("urn:uuid:{}", uuid::Uuid::new_v4()));
    opf.push_str(&format!(
        "    <dc:identifier id=\"uuid\">{}</dc:identifier>\n",
        escape_xml(&identifier)
    ));

    let title = metadata.title.as_deref().unwrap_or("Untitled");
    opf.push_str(&format!("    <dc:title>{}</dc:title>\n", escape_xml(title)));

    if let Some(creator) = &metadata.creator {
        opf.push_str(&format!(
            "    <dc:creator>{}</dc:creator>\n",
            escape_xml(creator)
        ));
    }

    let language = metadata.language.as_deref().unwrap_or("en");
    opf.push_str(&format!(
        "    <dc:language>{}</dc:language>\n",
        escape_xml(language)
    ));

    // G11: dc:date from creation_date
    if let Some(date) = &metadata.creation_date {
        opf.push_str(&format!("    <dc:date>{}</dc:date>\n", escape_xml(date)));
    }

    // G12: dc:description
    if let Some(desc) = &metadata.description {
        opf.push_str(&format!(
            "    <dc:description>{}</dc:description>\n",
            escape_xml(desc)
        ));
    }

    // G12: dc:subject
    if let Some(subject) = &metadata.subject {
        opf.push_str(&format!(
            "    <dc:subject>{}</dc:subject>\n",
            escape_xml(subject)
        ));
    }

    let modified = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    opf.push_str(&format!(
        "    <meta property=\"dcterms:modified\">{}</meta>\n",
        modified
    ));

    opf.push_str("  </metadata>\n");

    // ---- Manifest ----
    opf.push_str("  <manifest>\n");
    opf.push_str(
        "    <item id=\"nav\" href=\"nav.xhtml\" \
         media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n",
    );
    opf.push_str("    <item id=\"css\" href=\"Styles/styles.css\" media-type=\"text/css\"/>\n");

    for section in sections {
        opf.push_str(&format!(
            "    <item id=\"{}\" href=\"Text/{}.xhtml\" \
             media-type=\"application/xhtml+xml\"/>\n",
            section.id, section.id
        ));
    }

    for (idx, font) in fonts.iter().enumerate() {
        opf.push_str(&format!(
            "    <item id=\"font-{}\" href=\"Fonts/{}\" media-type=\"{}\"/>\n",
            idx,
            font.filename,
            font.format.media_type()
        ));
    }

    // G13: embedded images in manifest
    for image in images {
        opf.push_str(&format!(
            "    <item id=\"img-{}\" href=\"Images/{}\" media-type=\"{}\"/>\n",
            image.filename.replace('.', "-"),
            image.filename,
            image.media_type
        ));
    }

    opf.push_str("  </manifest>\n");

    // ---- Spine ----
    opf.push_str("  <spine>\n");
    for section in sections {
        opf.push_str(&format!("    <itemref idref=\"{}\"/>\n", section.id));
    }
    opf.push_str("  </spine>\n");

    opf.push_str("</package>\n");
    opf
}
