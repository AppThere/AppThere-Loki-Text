//! ODT `content.xml` writer.
//!
//! Generates the `content.xml` file for ZIP-format `.odt` documents.
//! Styles are expected to be in a separate `styles.xml` file.

use std::io::Cursor;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;

use common_core::{Block, Inline};

use crate::writer::blocks::write_inlines_with_marks;
use crate::writer::namespaces::push_content_ns;

/// Generates the `content.xml` string for a ZIP-format ODT file.
///
/// This output contains only the document body (`office:body` /
/// `office:text`) and an empty `office:automatic-styles` placeholder.
/// Named styles live in `styles.xml`.
///
/// # Arguments
///
/// * `blocks` - The document's block content.
///
/// # Errors
///
/// Returns a `String` error if XML writing fails.
pub fn to_content_xml(blocks: &[Block]) -> Result<String, String> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| e.to_string())?;

    let mut document = BytesStart::new("office:document-content");
    push_content_ns(&mut document);
    writer
        .write_event(Event::Start(document))
        .map_err(|e| e.to_string())?;

    // Empty automatic-styles — named styles are in styles.xml
    writer
        .write_event(Event::Start(BytesStart::new("office:automatic-styles")))
        .map_err(|e| e.to_string())?;
    writer
        .write_event(Event::End(BytesEnd::new("office:automatic-styles")))
        .map_err(|e| e.to_string())?;

    writer
        .write_event(Event::Start(BytesStart::new("office:body")))
        .map_err(|e| e.to_string())?;
    writer
        .write_event(Event::Start(BytesStart::new("office:text")))
        .map_err(|e| e.to_string())?;

    write_blocks_content(blocks, &mut writer)?;

    writer
        .write_event(Event::End(BytesEnd::new("office:text")))
        .map_err(|e| e.to_string())?;
    writer
        .write_event(Event::End(BytesEnd::new("office:body")))
        .map_err(|e| e.to_string())?;
    writer
        .write_event(Event::End(BytesEnd::new("office:document-content")))
        .map_err(|e| e.to_string())?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| e.to_string())
}

/// Writes blocks for `content.xml`, using marks-based inline writing.
///
/// Unlike the FODT writer, `content.xml` uses mark-based spans rather than
/// style-name spans for inline content.
fn write_blocks_content(
    blocks: &[Block],
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<(), String> {
    for block in blocks {
        write_block_content(block, writer)?;
    }
    Ok(())
}

fn write_block_content(block: &Block, writer: &mut Writer<Cursor<Vec<u8>>>) -> Result<(), String> {
    match block {
        Block::Paragraph {
            style_name,
            content,
            ..
        } => {
            let mut p = BytesStart::new("text:p");
            if let Some(s) = style_name {
                p.push_attribute(("text:style-name", s.as_str()));
            }
            writer
                .write_event(Event::Start(p))
                .map_err(|e| e.to_string())?;
            write_inlines_content(content, writer)?;
            writer
                .write_event(Event::End(BytesEnd::new("text:p")))
                .map_err(|e| e.to_string())?;
        }
        Block::Heading {
            level,
            style_name,
            content,
            ..
        } => {
            let mut h = BytesStart::new("text:h");
            if let Some(s) = style_name {
                h.push_attribute(("text:style-name", s.as_str()));
            }
            h.push_attribute(("text:outline-level", level.to_string().as_str()));
            writer
                .write_event(Event::Start(h))
                .map_err(|e| e.to_string())?;
            write_inlines_content(content, writer)?;
            writer
                .write_event(Event::End(BytesEnd::new("text:h")))
                .map_err(|e| e.to_string())?;
        }
        Block::PageBreak => {
            let mut p = BytesStart::new("text:p");
            p.push_attribute(("text:style-name", "PageBreak"));
            writer
                .write_event(Event::Empty(p))
                .map_err(|e| e.to_string())?;
        }
        Block::BulletList { content } | Block::OrderedList { content } => {
            writer
                .write_event(Event::Start(BytesStart::new("text:list")))
                .map_err(|e| e.to_string())?;
            for item in content {
                write_block_content(item, writer)?;
            }
            writer
                .write_event(Event::End(BytesEnd::new("text:list")))
                .map_err(|e| e.to_string())?;
        }
        Block::ListItem { content } => {
            writer
                .write_event(Event::Start(BytesStart::new("text:list-item")))
                .map_err(|e| e.to_string())?;
            write_blocks_content(content, writer)?;
            writer
                .write_event(Event::End(BytesEnd::new("text:list-item")))
                .map_err(|e| e.to_string())?;
        }
        Block::Table { content } => {
            writer
                .write_event(Event::Start(BytesStart::new("table:table")))
                .map_err(|e| e.to_string())?;
            write_blocks_content(content, writer)?;
            writer
                .write_event(Event::End(BytesEnd::new("table:table")))
                .map_err(|e| e.to_string())?;
        }
        Block::TableRow { content } => {
            writer
                .write_event(Event::Start(BytesStart::new("table:table-row")))
                .map_err(|e| e.to_string())?;
            write_blocks_content(content, writer)?;
            writer
                .write_event(Event::End(BytesEnd::new("table:table-row")))
                .map_err(|e| e.to_string())?;
        }
        Block::TableCell { content, .. } | Block::TableHeader { content, .. } => {
            writer
                .write_event(Event::Start(BytesStart::new("table:table-cell")))
                .map_err(|e| e.to_string())?;
            write_blocks_content(content, writer)?;
            writer
                .write_event(Event::End(BytesEnd::new("table:table-cell")))
                .map_err(|e| e.to_string())?;
        }
        Block::Blockquote { content } => write_blocks_content(content, writer)?,
        Block::HorizontalRule => {
            writer
                .write_event(Event::Empty(BytesStart::new("text:p")))
                .map_err(|e| e.to_string())?;
        }
        Block::Image { .. } => {} // Images not supported in content.xml writer
    }
    Ok(())
}

/// Writes inlines for content.xml using marks (not style names).
fn write_inlines_content(
    inlines: &[Inline],
    writer: &mut Writer<Cursor<Vec<u8>>>,
) -> Result<(), String> {
    write_inlines_with_marks(inlines, writer)
}
