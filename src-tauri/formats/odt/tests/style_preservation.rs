//! Style preservation tests.
//!
//! Verifies that named styles (paragraph, text/character) survive the full
//! round-trip: ODT XML → `Document` → `LexicalDocument` → `Document` → ODT XML.
//!
//! Tests use inline FODT fixtures so the suite runs without any external files.

use odt_format::{
    lexical::{from_lexical, to_lexical},
    parser::parse_document,
    Document,
};

// ── Namespace constants ───────────────────────────────────────────────────────

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";

fn fodt(styles_xml: &str, body_xml: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}"
    office:version="1.3">
  <office:styles>{styles_xml}</office:styles>
  <office:body>
    <office:text>{body_xml}</office:text>
  </office:body>
</office:document>"#
    )
}

// ── Helper: full lexical round-trip ──────────────────────────────────────────

/// Parse XML → Document → LexicalDocument → Document.
/// Returns the final Document, which should be semantically identical.
fn lexical_round_trip(xml: &str) -> Document {
    let doc1 = parse_document(xml).expect("parse failed");
    let lex = to_lexical(&doc1);
    from_lexical(lex, doc1.styles.clone(), doc1.metadata.clone())
}

// ── Style count preservation ──────────────────────────────────────────────────

#[test]
fn named_styles_survive_round_trip() {
    let xml = fodt(
        r#"<style:style style:name="MyHeading" style:family="paragraph">
            <style:paragraph-properties fo:text-align="center"/>
           </style:style>
           <style:style style:name="MyBody" style:family="paragraph">
            <style:paragraph-properties fo:margin-left="1cm"/>
           </style:style>"#,
        r#"<text:p text:style-name="MyHeading">Title</text:p>
           <text:p text:style-name="MyBody">Body text</text:p>"#,
    );

    let doc1 = parse_document(&xml).unwrap();
    let doc2 = lexical_round_trip(&xml);

    // All named styles must survive
    for name in doc1.styles.keys() {
        assert!(doc2.styles.contains_key(name), "Style '{name}' was lost");
    }
    assert_eq!(
        doc1.styles.len(),
        doc2.styles.len(),
        "Style count changed ({} → {})",
        doc1.styles.len(),
        doc2.styles.len()
    );
}

#[test]
fn style_family_preserved() {
    let xml = fodt(
        r#"<style:style style:name="CodeStyle" style:family="text">
            <style:text-properties fo:font-family="monospace"/>
           </style:style>"#,
        r#"<text:p>Normal <text:span text:style-name="CodeStyle">code</text:span></text:p>"#,
    );

    let doc1 = parse_document(&xml).unwrap();
    let doc2 = lexical_round_trip(&xml);

    let orig = doc1.styles.get("CodeStyle").expect("style present in doc1");
    let restored = doc2.styles.get("CodeStyle").expect("CodeStyle lost after round-trip");

    assert_eq!(
        orig.family, restored.family,
        "style:family changed for CodeStyle"
    );
}

#[test]
fn paragraph_alignment_attribute_preserved() {
    let xml = fodt(
        r#"<style:style style:name="Centered" style:family="paragraph">
            <style:paragraph-properties fo:text-align="center"/>
           </style:style>"#,
        r#"<text:p text:style-name="Centered">Centered paragraph</text:p>"#,
    );

    let doc1 = parse_document(&xml).unwrap();
    let doc2 = lexical_round_trip(&xml);

    let orig = doc1.styles.get("Centered").unwrap();
    let restored = doc2.styles.get("Centered").expect("Centered style lost");

    // fo:text-align must survive
    let orig_align = orig.attributes.get("fo:text-align");
    let rest_align = restored.attributes.get("fo:text-align");
    assert_eq!(orig_align, rest_align, "fo:text-align changed for Centered");
}

#[test]
fn bold_italic_text_style_attributes_preserved() {
    let xml = fodt(
        r#"<style:style style:name="Emphasis" style:family="text">
            <style:text-properties fo:font-style="italic" fo:font-weight="bold"/>
           </style:style>"#,
        r#"<text:p><text:span text:style-name="Emphasis">Styled</text:span></text:p>"#,
    );

    let doc1 = parse_document(&xml).unwrap();
    let doc2 = lexical_round_trip(&xml);

    let orig = doc1.styles.get("Emphasis").unwrap();
    let restored = doc2.styles.get("Emphasis").expect("Emphasis style lost");

    for key in ["fo:font-style", "fo:font-weight"] {
        assert_eq!(
            orig.attributes.get(key),
            restored.attributes.get(key),
            "attribute '{key}' changed for Emphasis"
        );
    }
}

// ── Paragraph style names in blocks ──────────────────────────────────────────

#[test]
fn paragraph_style_name_survives_round_trip() {
    use common_core::Block;

    let xml = fodt(
        r#"<style:style style:name="BodyText" style:family="paragraph"/>"#,
        r#"<text:p text:style-name="BodyText">Hello</text:p>"#,
    );

    let doc2 = lexical_round_trip(&xml);

    assert_eq!(doc2.blocks.len(), 1);
    if let Block::Paragraph { style_name, .. } = &doc2.blocks[0] {
        assert_eq!(
            style_name.as_deref(),
            Some("BodyText"),
            "paragraph style name lost"
        );
    } else {
        panic!("expected Paragraph block");
    }
}

#[test]
fn heading_style_name_survives_round_trip() {
    use common_core::Block;

    let xml = fodt(
        r#"<style:style style:name="Heading 1" style:family="paragraph"
                style:outline-level="1"/>"#,
        r#"<text:h text:style-name="Heading 1" text:outline-level="1">Chapter</text:h>"#,
    );

    let doc2 = lexical_round_trip(&xml);

    assert_eq!(doc2.blocks.len(), 1);
    if let Block::Heading { level, style_name, .. } = &doc2.blocks[0] {
        assert_eq!(*level, 1);
        assert_eq!(
            style_name.as_deref(),
            Some("Heading 1"),
            "heading style name lost"
        );
    } else {
        panic!("expected Heading block");
    }
}

// ── Multiple styles coexist ───────────────────────────────────────────────────

#[test]
fn multiple_paragraph_styles_all_preserved() {
    let xml = fodt(
        r#"<style:style style:name="Standard"   style:family="paragraph"/>
           <style:style style:name="Body_Text"  style:family="paragraph"/>
           <style:style style:name="Preformat"  style:family="paragraph">
             <style:text-properties fo:font-family="monospace"/>
           </style:style>"#,
        r#"<text:p text:style-name="Standard">A</text:p>
           <text:p text:style-name="Body_Text">B</text:p>
           <text:p text:style-name="Preformat">C</text:p>"#,
    );

    let doc1 = parse_document(&xml).unwrap();
    let doc2 = lexical_round_trip(&xml);

    for name in ["Standard", "Body_Text", "Preformat"] {
        assert!(doc1.styles.contains_key(name), "{name} missing from doc1");
        assert!(doc2.styles.contains_key(name), "{name} missing after round-trip");
    }
}

// ── Empty styles section ──────────────────────────────────────────────────────

#[test]
fn no_styles_round_trip_is_stable() {
    let xml = fodt(
        "",
        r#"<text:p>Plain paragraph without any named style</text:p>"#,
    );

    let doc1 = parse_document(&xml).unwrap();
    let doc2 = lexical_round_trip(&xml);

    assert_eq!(doc1.blocks.len(), doc2.blocks.len());
    assert_eq!(doc1.styles.len(), doc2.styles.len());
}

// ── Style metadata fields ─────────────────────────────────────────────────────

#[test]
fn style_display_name_preserved() {
    use common_core::StyleFamily;

    let xml = fodt(
        r#"<style:style style:name="MyStyle" style:display-name="My Custom Style"
                style:family="paragraph"/>"#,
        r#"<text:p text:style-name="MyStyle">text</text:p>"#,
    );

    let doc1 = parse_document(&xml).unwrap();
    let doc2 = lexical_round_trip(&xml);

    let orig = doc1.styles.get("MyStyle").unwrap();
    let restored = doc2.styles.get("MyStyle").expect("MyStyle lost");

    assert_eq!(orig.display_name, restored.display_name, "display_name changed");
    assert_eq!(StyleFamily::Paragraph, restored.family);
}

#[test]
fn style_parent_name_preserved() {
    let xml = fodt(
        r#"<style:style style:name="Parent" style:family="paragraph"/>
           <style:style style:name="Child" style:family="paragraph"
                style:parent-style-name="Parent"/>"#,
        r#"<text:p text:style-name="Child">derived</text:p>"#,
    );

    let doc1 = parse_document(&xml).unwrap();
    let doc2 = lexical_round_trip(&xml);

    let orig = doc1.styles.get("Child").unwrap();
    let restored = doc2.styles.get("Child").expect("Child style lost");

    assert_eq!(orig.parent, restored.parent, "parent style name changed");
}
