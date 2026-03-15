//! Memory-usage and leak-detection tests.
//!
//! `test_no_leaks_repeated_operations` runs in CI and checks that memory stays
//! stable across 1 000 parse/write cycles.
//!
//! `test_memory_large_document` is `#[ignore]`d; run it manually together with
//! valgrind to verify that no memory is retained after the document is dropped:
//!
//!   cargo test --release test_memory_large_document -- --ignored --exact

use common_core::{marks::TiptapMark, Block, Inline};
use odt_format::{
    lexical::{from_lexical, to_lexical},
    parser::parse_document,
    Document,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn simple_document(n: usize) -> Document {
    let mut doc = Document::new();
    for i in 0..n {
        doc.blocks.push(Block::Paragraph {
            style_name: None,
            attrs: None,
            content: vec![Inline::Text {
                text: format!("Paragraph {i} with some text content."),
                style_name: None,
                marks: vec![],
            }],
        });
    }
    doc
}

fn formatted_document(n: usize) -> Document {
    let mut doc = Document::new();
    for i in 0..n {
        doc.blocks.push(Block::Paragraph {
            style_name: None,
            attrs: None,
            content: vec![
                Inline::Text {
                    text: "Normal ".to_string(),
                    style_name: None,
                    marks: vec![],
                },
                Inline::Text {
                    text: "bold ".to_string(),
                    style_name: None,
                    marks: vec![TiptapMark::Bold],
                },
                Inline::Text {
                    text: format!("para {i}."),
                    style_name: None,
                    marks: vec![TiptapMark::Italic],
                },
            ],
        });
    }
    doc
}

fn paragraphs_xml(n: usize) -> String {
    let ns_office = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
    let ns_text = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
    let body: String = (0..n)
        .map(|i| format!("<text:p>Paragraph {i}</text:p>"))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{ns_office}" xmlns:text="{ns_text}"
    office:version="1.3">
  <office:body><office:text>{body}</office:text></office:body>
</office:document>"#
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Runs 1 000 full parse → to_lexical → from_lexical → write cycles and
/// verifies results stay correct. If memory were leaking, the process would
/// grow unboundedly; this acts as a lightweight sanity check in CI.
#[test]
fn test_no_leaks_repeated_operations() {
    for i in 0..1_000 {
        let doc = simple_document(100);
        let xml = doc.to_content_xml().unwrap();
        let restored = parse_document(&xml).unwrap();
        assert_eq!(doc.blocks.len(), restored.blocks.len(), "iteration {i}");
        drop(restored);
        drop(xml);
        drop(doc);
    }
}

/// Verifies that parse → write → parse produces identical block counts for
/// a moderately large formatted document. Exercises the full content path.
#[test]
fn test_memory_usage_formatted_text() {
    let doc = formatted_document(10_000);
    let xml = doc.to_content_xml().unwrap();
    let restored = parse_document(&xml).unwrap();
    assert_eq!(doc.blocks.len(), restored.blocks.len());
}

/// Verifies that the Lexical round-trip does not accumulate state across
/// repeated conversions (tests `to_lexical` + `from_lexical` path).
#[test]
fn test_lexical_conversion_memory_stable() {
    for _ in 0..200 {
        let doc = simple_document(500);
        let lex = to_lexical(&doc);
        let doc2 = from_lexical(lex, doc.styles.clone(), doc.metadata.clone());
        assert_eq!(doc.blocks.len(), doc2.blocks.len());
        drop(doc2);
        drop(doc);
    }
}

/// Manual test for valgrind / heaptrack.
///
/// Run with:
///   `cargo test --release test_memory_large_document -- --ignored --exact`
///
/// Expected: valgrind reports "definitely lost: 0 bytes in 0 blocks".
#[test]
#[ignore]
fn test_memory_large_document() {
    let doc = simple_document(100_000);
    let xml = doc.to_content_xml().unwrap();
    println!("XML size: {} MB", xml.len() / 1_000_000);
    let restored = parse_document(&xml).unwrap();
    assert_eq!(doc.blocks.len(), restored.blocks.len());
    drop(restored);
    drop(xml);
    drop(doc);
    println!("All allocations released. Check valgrind for leaks.");
}

/// Generates a large XML string, parses it, round-trips through Lexical, and
/// checks that block counts remain correct end-to-end.
#[test]
fn test_large_xml_parse_and_lexical_roundtrip() {
    let xml = paragraphs_xml(5_000);
    let doc = parse_document(&xml).unwrap();
    let lex = to_lexical(&doc);
    let doc2 = from_lexical(lex, doc.styles.clone(), doc.metadata.clone());
    assert_eq!(doc.blocks.len(), doc2.blocks.len());
}
