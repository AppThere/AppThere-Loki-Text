use common_core::{Metadata, StyleDefinition};
use std::collections::HashMap;

fn main() {
    let old_xml = r#"<?xml version="1.0" encoding="UTF-8"?><office:document-styles xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:loki="https://appthere.com/loki/ns"><office:font-face-decls></office:font-face-decls><office:styles><style:style style:name="Standard" style:family="paragraph"><style:paragraph-properties fo:margin-top="0in"/></style:style></office:styles><office:automatic-styles><style:style style:name="T1" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style></office:automatic-styles><office:master-styles></office:master-styles></office:document-styles>"#;
    let mut styles = HashMap::new();
    styles.insert(
        "Custom".to_string(),
        StyleDefinition {
            name: "Custom".into(),
            family: common_core::StyleFamily::Paragraph,
            parent: None,
            next: None,
            display_name: None,
            attributes: HashMap::new(),
            text_transform: None,
            outline_level: None,
            autocomplete: None,
            font_colour: None,
            background_colour: None,
        },
    );

    let styles_xml =
        odt_format::writer::styles_writer::styles_to_xml(&styles, &None, &None, &None).unwrap();
    println!("STYLES XML:\n{}\n", styles_xml);

    let blocks = vec![];
    let metadata = Metadata::default();

    let updated = odt_format::writer::fodt::update_fodt(
        old_xml,
        &blocks,
        &styles,
        &metadata,
        "",
        &styles_xml,
        "",
    )
    .unwrap();

    println!("UPDATED:\n{}", updated);
}
