//! Write-then-parse integration tests.
//!
//! Each test verifies the content.xml writer path:
//!
//!   Block(s) → to_content_xml → parse_document → Block(s)
//!
//! These guard against silent drops or corruption introduced by the
//! content.xml writer (as distinct from the FODT / Lexical path).

use common_core::{block::CellAttrs, Block, Inline};
use odt_format::{parser::parse_document, writer::content::to_content_xml};

// ── Image ─────────────────────────────────────────────────────────────────────

/// Image block with a data: URI src must survive write → parse intact.
#[test]
fn image_round_trips_through_content_xml() {
    let src = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
    let blocks = vec![Block::Image {
        src: src.to_string(),
        alt: None,
        title: None,
    }];
    let xml = to_content_xml(&blocks).expect("to_content_xml failed");
    let doc = parse_document(&xml).expect("parse_document failed");

    assert_eq!(doc.blocks.len(), 1, "expected exactly one block");
    if let Block::Image { src: s, alt, .. } = &doc.blocks[0] {
        assert_eq!(s, src, "src changed after round-trip");
        assert_eq!(alt, &None, "alt changed after round-trip");
    } else {
        panic!("expected Block::Image, got {:?}", doc.blocks[0]);
    }
}

// ── Table ─────────────────────────────────────────────────────────────────────

fn make_para(text: &str) -> Block {
    Block::Paragraph {
        style_name: None,
        attrs: None,
        content: vec![Inline::Text {
            text: text.to_string(),
            marks: vec![],
            style_name: None,
        }],
    }
}

fn make_cell(text: &str) -> Block {
    Block::TableCell {
        attrs: None,
        content: vec![make_para(text)],
    }
}

/// A 2×2 table must survive write → parse with structure and text intact.
#[test]
fn table_2x2_round_trips_through_content_xml() {
    let blocks = vec![Block::Table {
        content: vec![
            Block::TableRow {
                content: vec![make_cell("R1C1"), make_cell("R1C2")],
            },
            Block::TableRow {
                content: vec![make_cell("R2C1"), make_cell("R2C2")],
            },
        ],
    }];

    let xml = to_content_xml(&blocks).expect("to_content_xml failed");
    let doc = parse_document(&xml).expect("parse_document failed");

    assert_eq!(doc.blocks.len(), 1, "expected one table block");
    let Block::Table { content: rows } = &doc.blocks[0] else {
        panic!("expected Block::Table");
    };
    assert_eq!(rows.len(), 2, "expected two rows");

    for (ri, row) in rows.iter().enumerate() {
        let Block::TableRow { content: cells } = row else {
            panic!("expected Block::TableRow at row {ri}");
        };
        assert_eq!(cells.len(), 2, "expected two cells in row {ri}");

        for (ci, cell) in cells.iter().enumerate() {
            let Block::TableCell { content: paras, .. } = cell else {
                panic!("expected Block::TableCell at [{ri}][{ci}]");
            };
            let Block::Paragraph { content: inlines, .. } = &paras[0] else {
                panic!("expected Paragraph in cell [{ri}][{ci}]");
            };
            let Inline::Text { text, .. } = &inlines[0] else {
                panic!("expected Text inline in cell [{ri}][{ci}]");
            };
            let expected = format!("R{}C{}", ri + 1, ci + 1);
            assert_eq!(text, &expected, "cell text mismatch at [{ri}][{ci}]");
        }
    }
}

/// A cell with col_span=2 must have colspan preserved through write → parse.
#[test]
fn table_cell_col_span_preserved_through_content_xml() {
    let spanned_cell = Block::TableCell {
        attrs: Some(CellAttrs {
            colspan: Some(2),
            rowspan: None,
            colwidth: None,
        }),
        content: vec![make_para("wide")],
    };
    let blocks = vec![Block::Table {
        content: vec![Block::TableRow {
            content: vec![spanned_cell],
        }],
    }];

    let xml = to_content_xml(&blocks).expect("to_content_xml failed");
    let doc = parse_document(&xml).expect("parse_document failed");

    let Block::Table { content: rows } = &doc.blocks[0] else {
        panic!("expected Block::Table");
    };
    let Block::TableRow { content: cells } = &rows[0] else {
        panic!("expected Block::TableRow");
    };
    let Block::TableCell { attrs, .. } = &cells[0] else {
        panic!("expected Block::TableCell");
    };
    let attrs = attrs.as_ref().expect("expected Some(CellAttrs)");
    assert_eq!(attrs.colspan, Some(2), "colspan not preserved");
}
