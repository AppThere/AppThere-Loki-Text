//! Shared test-document generators for benchmarks.
//!
//! Provides both XML string generators (for parse benchmarks) and
//! in-memory `Document` builders (for write / conversion benchmarks).
//!
//! This file is compiled as a module into each bench binary via `mod generators;`.
//! Not every item is used in every binary, so dead_code warnings are suppressed.
#![allow(dead_code)]

use std::collections::HashMap;

use common_core::{marks::TiptapMark, Block, Inline, StyleDefinition, StyleFamily};
use odt_format::Document;

// ── Namespace constants ───────────────────────────────────────────────────────

pub const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
pub const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
pub const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
pub const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
pub const NS_TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";

// ── XML generators ────────────────────────────────────────────────────────────

/// Wrap `body` in a minimal FODT document with the standard namespaces.
pub fn fodt(styles: &str, body: &str) -> String {
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

/// `n` plain-text paragraphs.
pub fn paragraphs_xml(n: usize) -> String {
    let body: String = (0..n)
        .map(|i| format!("<text:p>Paragraph {i} with sample text content.</text:p>"))
        .collect();
    fodt("", &body)
}

/// `n` paragraphs each containing a bold and an italic span.
pub fn formatted_xml(n: usize) -> String {
    let styles = r#"
        <style:style style:name="B" style:family="text">
            <style:text-properties fo:font-weight="bold"/>
        </style:style>
        <style:style style:name="I" style:family="text">
            <style:text-properties fo:font-style="italic"/>
        </style:style>"#;
    let body: String = (0..n)
        .map(|i| {
            format!(
                r#"<text:p>Plain <text:span text:style-name="B">bold</text:span> \
and <text:span text:style-name="I">italic</text:span> para {i}.</text:p>"#
            )
        })
        .collect();
    fodt(styles, &body)
}

/// A table of `rows × cols` cells.
pub fn table_xml(rows: usize, cols: usize) -> String {
    let mut body = String::from("<table:table>");
    for r in 0..rows {
        body.push_str("<table:table-row>");
        for c in 0..cols {
            body.push_str(&format!(
                "<table:table-cell><text:p>R{r}C{c}</text:p></table:table-cell>"
            ));
        }
        body.push_str("</table:table-row>");
    }
    body.push_str("</table:table>");
    fodt("", &body)
}

/// A flat bullet list with `items` items.
pub fn list_xml(items: usize) -> String {
    let mut body = String::from("<text:list>");
    for i in 0..items {
        body.push_str(&format!(
            "<text:list-item><text:p>Item {i}</text:p></text:list-item>"
        ));
    }
    body.push_str("</text:list>");
    fodt("", &body)
}

/// `n_paras` paragraphs cycling through `n_styles` named paragraph styles.
pub fn many_styles_xml(n_paras: usize, n_styles: usize) -> String {
    let styles: String = (0..n_styles)
        .map(|i| {
            format!(
                r#"<style:style style:name="S{i}" style:family="paragraph">
                    <style:paragraph-properties fo:text-align="left"/>
                   </style:style>"#
            )
        })
        .collect();
    let body: String = (0..n_paras)
        .map(|i| {
            let s = i % n_styles;
            format!(r#"<text:p text:style-name="S{s}">Paragraph {i}</text:p>"#)
        })
        .collect();
    fodt(&styles, &body)
}

// ── Document builders ─────────────────────────────────────────────────────────

/// `n` plain-text paragraph blocks.
pub fn simple_document(n: usize) -> Document {
    let mut doc = Document::new();
    for i in 0..n {
        doc.blocks.push(Block::Paragraph {
            style_name: Some("Standard".to_string()),
            attrs: None,
            content: vec![Inline::Text {
                text: format!("Paragraph {i} with sample text."),
                style_name: None,
                marks: vec![],
            }],
        });
    }
    doc
}

/// `n` paragraphs each with plain + bold + italic inlines.
pub fn formatted_document(n: usize) -> Document {
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
                    text: "bold".to_string(),
                    style_name: None,
                    marks: vec![TiptapMark::Bold],
                },
                Inline::Text {
                    text: format!(" para {i}."),
                    style_name: None,
                    marks: vec![TiptapMark::Italic],
                },
            ],
        });
    }
    doc
}

/// `n_paras` paragraphs and `n_styles` named paragraph styles in the registry.
pub fn styled_document(n_paras: usize, n_styles: usize) -> Document {
    let mut doc = Document::new();
    for i in 0..n_styles {
        doc.styles.insert(
            format!("S{i}"),
            StyleDefinition {
                name: format!("S{i}"),
                family: StyleFamily::Paragraph,
                parent: None,
                next: None,
                display_name: Some(format!("Style {i}")),
                attributes: HashMap::new(),
                text_transform: None,
                outline_level: None,
                autocomplete: None,
            },
        );
    }
    for i in 0..n_paras {
        doc.blocks.push(Block::Paragraph {
            style_name: Some(format!("S{}", i % n_styles)),
            attrs: None,
            content: vec![Inline::Text {
                text: format!("Paragraph {i}"),
                style_name: None,
                marks: vec![],
            }],
        });
    }
    doc
}
