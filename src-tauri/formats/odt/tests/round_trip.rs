//! Round-trip regression tests.
//!
//! Each test verifies that a document structure survives the full pipeline:
//!
//!   ODT XML → parse → to_lexical → from_lexical → to_content_xml → parse
//!
//! Content (block count, text, formatting) must be identical before and after.
//! These tests guard against regressions introduced by parser or writer changes.

use common_core::{Block, Inline};
use odt_format::{
    lexical::{from_lexical, to_lexical},
    parser::parse_document,
};

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
const NS_TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";

fn fodt(styles: &str, body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}"
    xmlns:table="{NS_TABLE}" office:version="1.3">
  <office:styles>{styles}</office:styles>
  <office:body>
    <office:text>{body}</office:text>
  </office:body>
</office:document>"#
    )
}

/// Full parse → Lexical → parse round-trip.
fn round_trip(xml: &str) -> (Vec<Block>, Vec<Block>) {
    let doc1 = parse_document(xml).expect("initial parse failed");
    let lex = to_lexical(&doc1);
    let doc2 = from_lexical(lex, doc1.styles.clone(), doc1.metadata.clone());
    (doc1.blocks, doc2.blocks)
}

// ── Paragraphs ────────────────────────────────────────────────────────────────

#[test]
fn plain_paragraph_round_trips() {
    let (b1, b2) = round_trip(&fodt("", r#"<text:p>Hello world</text:p>"#));
    assert_eq!(b1.len(), b2.len());
    if let (Block::Paragraph { content: c1, .. }, Block::Paragraph { content: c2, .. }) =
        (&b1[0], &b2[0])
    {
        assert_eq!(c1.len(), c2.len());
        assert_inline_text(&c1[0], "Hello world");
        assert_inline_text(&c2[0], "Hello world");
    } else {
        panic!("expected Paragraph blocks");
    }
}

#[test]
fn multiple_paragraphs_count_preserved() {
    let (b1, b2) = round_trip(&fodt(
        "",
        r#"<text:p>A</text:p><text:p>B</text:p><text:p>C</text:p>"#,
    ));
    assert_eq!(b1.len(), 3);
    assert_eq!(b2.len(), 3);
}

#[test]
fn empty_paragraph_survives() {
    let (b1, b2) = round_trip(&fodt("", r#"<text:p/>"#));
    assert_eq!(b1.len(), b2.len());
    assert_eq!(b1.len(), 1);
}

// ── Headings ──────────────────────────────────────────────────────────────────

#[test]
fn heading_level_and_text_preserved() {
    let (b1, b2) = round_trip(&fodt(
        "",
        r#"<text:h text:outline-level="2">Section 2</text:h>"#,
    ));
    if let (Block::Heading { level: l1, .. }, Block::Heading { level: l2, .. }) = (&b1[0], &b2[0]) {
        assert_eq!(l1, l2, "heading level changed");
        assert_eq!(l1, &2u32);
    } else {
        panic!("expected Heading blocks");
    }
}

#[test]
fn mixed_paragraphs_and_headings_order_preserved() {
    let xml = fodt(
        "",
        r#"<text:h text:outline-level="1">Title</text:h>
           <text:p>Intro paragraph</text:p>
           <text:h text:outline-level="2">Subtitle</text:h>
           <text:p>Body paragraph</text:p>"#,
    );
    let (b1, b2) = round_trip(&xml);
    assert_eq!(b1.len(), b2.len(), "block count changed");

    let is_heading = |b: &Block| matches!(b, Block::Heading { .. });
    let is_para = |b: &Block| matches!(b, Block::Paragraph { .. });

    assert!(is_heading(&b2[0]), "b2[0] should be Heading");
    assert!(is_para(&b2[1]), "b2[1] should be Paragraph");
    assert!(is_heading(&b2[2]), "b2[2] should be Heading");
    assert!(is_para(&b2[3]), "b2[3] should be Paragraph");
}

// ── Inline formatting ─────────────────────────────────────────────────────────

#[test]
fn bold_text_format_preserved() {
    use common_core::marks::TiptapMark;

    let xml = fodt(
        r#"<style:style style:name="B" style:family="text">
            <style:text-properties fo:font-weight="bold"/>
           </style:style>"#,
        r#"<text:p><text:span text:style-name="B">Bold text</text:span></text:p>"#,
    );
    let (b1, b2) = round_trip(&xml);

    let inlines1 = paragraph_inlines(&b1[0]);
    let inlines2 = paragraph_inlines(&b2[0]);
    assert_eq!(inlines1.len(), inlines2.len());

    if let (Inline::Text { marks: m1, .. }, Inline::Text { marks: m2, .. }) =
        (&inlines1[0], &inlines2[0])
    {
        assert_eq!(
            m1.contains(&TiptapMark::Bold),
            m2.contains(&TiptapMark::Bold)
        );
    }
}

#[test]
fn mixed_plain_and_bold_inlines_count_preserved() {
    let xml = fodt(
        r#"<style:style style:name="B" style:family="text">
            <style:text-properties fo:font-weight="bold"/>
           </style:style>"#,
        r#"<text:p>
            plain
            <text:span text:style-name="B">bold</text:span>
            plain again
           </text:p>"#,
    );
    let (b1, b2) = round_trip(&xml);
    let c1 = paragraph_inlines(&b1[0]);
    let c2 = paragraph_inlines(&b2[0]);
    assert_eq!(c1.len(), c2.len(), "inline count changed");
}

// ── Lists ─────────────────────────────────────────────────────────────────────

#[test]
fn bullet_list_items_preserved() {
    let xml = fodt(
        "",
        r#"<text:list>
             <text:list-item><text:p>Item 1</text:p></text:list-item>
             <text:list-item><text:p>Item 2</text:p></text:list-item>
             <text:list-item><text:p>Item 3</text:p></text:list-item>
           </text:list>"#,
    );
    let (b1, b2) = round_trip(&xml);
    assert_eq!(b1.len(), b2.len(), "list block count changed");

    if let (Block::BulletList { content: c1 }, Block::BulletList { content: c2 }) = (&b1[0], &b2[0])
    {
        assert_eq!(c1.len(), c2.len(), "list item count changed");
    } else {
        panic!("expected BulletList blocks");
    }
}

// ── Images ────────────────────────────────────────────────────────────────────

#[test]
fn image_block_src_preserved() {
    let ns_draw = "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0";
    let ns_xlink = "http://www.w3.org/1999/xlink";

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:draw="{ns_draw}" xmlns:xlink="{ns_xlink}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}" office:version="1.3">
  <office:body>
    <office:text>
      <text:p>
        <draw:frame><draw:image xlink:href="images/photo.png"/></draw:frame>
      </text:p>
    </office:text>
  </office:body>
</office:document>"#
    );

    let (b1, b2) = round_trip(&xml);
    assert_eq!(b1.len(), b2.len());

    if let (Block::Image { src: s1, .. }, Block::Image { src: s2, .. }) = (&b1[0], &b2[0]) {
        assert_eq!(s1, s2, "image src changed");
    } else {
        panic!("expected Image blocks");
    }
}

// ── Page break ────────────────────────────────────────────────────────────────

#[test]
fn page_break_block_survives() {
    // The parser recognises <text:p text:style-name="PageBreak"/> as a page break.
    // The writer emits the same format, so the round-trip must preserve it.
    let xml = fodt(
        "",
        r#"<text:p>Before</text:p>
           <text:p text:style-name="PageBreak"/>
           <text:p>After</text:p>"#,
    );

    let (b1, b2) = round_trip(&xml);
    assert_eq!(b1.len(), b2.len(), "block count changed around page break");

    let has_page_break = |blocks: &[Block]| blocks.iter().any(|b| matches!(b, Block::PageBreak));
    assert!(has_page_break(&b1), "parser did not recognise PageBreak");
    assert!(has_page_break(&b2), "PageBreak lost after round-trip");
}

// ── Metadata ──────────────────────────────────────────────────────────────────

#[test]
fn document_metadata_preserved_through_round_trip() {
    let ns_dc = "http://purl.org/dc/elements/1.1/";
    let ns_meta = "urn:oasis:names:tc:opendocument:xmlns:meta:1.0";

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:dc="{ns_dc}" xmlns:meta="{ns_meta}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}" office:version="1.3">
  <office:meta>
    <dc:title>My Document</dc:title>
    <dc:creator>Jane Doe</dc:creator>
  </office:meta>
  <office:body>
    <office:text><text:p>Body</text:p></office:text>
  </office:body>
</office:document>"#
    );

    let doc1 = parse_document(&xml).unwrap();
    let lex = to_lexical(&doc1);
    let doc2 = from_lexical(lex, doc1.styles.clone(), doc1.metadata.clone());

    assert_eq!(doc1.metadata.title, doc2.metadata.title, "title changed");
    assert_eq!(
        doc1.metadata.creator, doc2.metadata.creator,
        "creator changed"
    );
}

// ── Page-break style round-trips ──────────────────────────────────────────────

/// A paragraph style that has `fo:break-before = "page"` should survive the
/// full round-trip without sprouting an extra explicit PageBreak block.
#[test]
fn style_with_break_before_round_trips_without_extra_page_break() {
    let style_xml = format!(
        r#"<style:style style:name="Chapter" style:family="paragraph"
                xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}">
            <style:paragraph-properties fo:break-before="page"/>
        </style:style>"#
    );
    let body = format!(
        r#"<text:p xmlns:text="{NS_TEXT}" text:style-name="Chapter">Chapter 1</text:p>"#
    );
    let (b1, b2) = round_trip(&fodt(&style_xml, &body));

    // The source and round-tripped documents must both contain exactly one
    // block (the paragraph) — no extra PageBreak blocks.
    assert_eq!(b1.len(), 1, "source must have 1 block");
    assert_eq!(b2.len(), 1, "round-tripped must have 1 block");
    assert!(matches!(b1[0], Block::Paragraph { .. }));
    assert!(matches!(b2[0], Block::Paragraph { .. }));
}

/// The Lexical representation of a document with a break-before style must
/// contain a PageBreak node before the styled paragraph.
#[test]
fn style_with_break_before_produces_page_break_node_in_lexical() {
    use common_core::lexical::LexicalNode;
    use odt_format::{lexical::to_lexical, parser::parse_document};

    let style_xml = format!(
        r#"<style:style style:name="Chapter" style:family="paragraph"
                xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}">
            <style:paragraph-properties fo:break-before="page"/>
        </style:style>"#
    );
    let body = format!(
        r#"<text:p xmlns:text="{NS_TEXT}" text:style-name="Chapter">Intro</text:p>"#
    );
    let doc = parse_document(&fodt(&style_xml, &body)).unwrap();
    let lex = to_lexical(&doc);

    assert_eq!(
        lex.root.children.len(),
        2,
        "expected [PageBreak, ParagraphStyle]"
    );
    assert!(
        matches!(lex.root.children[0], LexicalNode::PageBreak { .. }),
        "first node must be PageBreak"
    );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn paragraph_inlines(block: &Block) -> &[Inline] {
    match block {
        Block::Paragraph { content, .. } => content,
        _ => panic!("expected Paragraph"),
    }
}

fn assert_inline_text(inline: &Inline, expected: &str) {
    if let Inline::Text { text, .. } = inline {
        assert_eq!(text, expected, "inline text mismatch");
    } else {
        panic!("expected Text inline, got {inline:?}");
    }
}
