use std::collections::HashMap;

use common_core::{Block, CellAttrs, StyleDefinition};

use crate::ImageAsset;

use super::html::{block_to_html, FootnoteSeqMap};

/// Returns `true` when every cell in a `TableRow` block is a `TableHeader`.
pub(crate) fn is_header_row(row: &Block) -> bool {
    match row {
        Block::TableRow { content } => {
            !content.is_empty()
                && content
                    .iter()
                    .all(|c| matches!(c, Block::TableHeader { .. }))
        }
        _ => false,
    }
}

/// Render a `Table` block to XHTML, splitting leading header rows into
/// `<thead>` and remaining rows into `<tbody>`.
pub(crate) fn render_table(
    rows: &[Block],
    styles: &HashMap<String, StyleDefinition>,
    images: &[ImageAsset],
    fn_seq: &FootnoteSeqMap,
) -> String {
    let header_end = rows.iter().take_while(|r| is_header_row(r)).count();
    let mut html = String::from("  <table>\n");

    if header_end > 0 {
        html.push_str("    <thead>\n");
        for row in &rows[..header_end] {
            html.push_str(&block_to_html(row, styles, images, fn_seq));
        }
        html.push_str("    </thead>\n");
    }

    html.push_str("    <tbody>\n");
    for row in &rows[header_end..] {
        html.push_str(&block_to_html(row, styles, images, fn_seq));
    }
    html.push_str("    </tbody>\n");
    html.push_str("  </table>\n");
    html
}

/// Render a single table cell (`<th>` or `<td>`) with optional spanning
/// attributes expressed as inline HTML attributes.
pub(crate) fn render_table_cell(
    tag: &str,
    attrs: Option<&CellAttrs>,
    content: &[Block],
    styles: &HashMap<String, StyleDefinition>,
    images: &[ImageAsset],
    fn_seq: &FootnoteSeqMap,
) -> String {
    let mut attr_str = String::new();
    if let Some(a) = attrs {
        if let Some(n) = a.colspan.filter(|&n| n > 1) {
            attr_str.push_str(&format!(" colspan=\"{}\"", n));
        }
        if let Some(n) = a.rowspan.filter(|&n| n > 1) {
            attr_str.push_str(&format!(" rowspan=\"{}\"", n));
        }
    }
    let mut html = format!("        <{}{}>\n", tag, attr_str);
    for b in content {
        html.push_str(&block_to_html(b, styles, images, fn_seq));
    }
    html.push_str(&format!("        </{}>\n", tag));
    html
}
