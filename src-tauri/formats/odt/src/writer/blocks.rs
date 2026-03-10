//! ODT block and inline XML writers.
//!
//! Provides [`write_blocks`] and [`write_inlines_with_style`] for emitting
//! ODF-compatible XML from the document's block/inline tree. These are used
//! by both the FODT and the content.xml writers.

use std::io::Cursor;

use common_core::{Block, Inline};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

/// An alias for the XML writer type used throughout the ODT writers.
pub type XmlWriter = Writer<Cursor<Vec<u8>>>;

/// Writes a slice of blocks as ODF XML.
///
/// Handles all block types: paragraphs, headings, lists, tables, images,
/// page breaks, etc. Inline content is written using [`write_inlines_with_style`].
///
/// # Errors
///
/// Returns a `String` error if any XML writing operation fails.
pub fn write_blocks(blocks: &[Block], writer: &mut XmlWriter) -> Result<(), String> {
    for block in blocks {
        write_single_block(block, writer)?;
    }
    Ok(())
}

/// Writes a single block element.
fn write_single_block(block: &Block, writer: &mut XmlWriter) -> Result<(), String> {
    match block {
        Block::Paragraph {
            style_name,
            content,
            ..
        } => write_paragraph(style_name.as_deref(), content, writer),
        Block::Heading {
            level,
            style_name,
            content,
            ..
        } => write_heading(*level, style_name.as_deref(), content, writer),
        Block::PageBreak => write_page_break(writer),
        Block::BulletList { content } | Block::OrderedList { content } => {
            write_list(content, writer)
        }
        Block::ListItem { content } => write_list_item(content, writer),
        Block::Table { content } => write_table(content, writer),
        Block::TableRow { content } => write_table_row(content, writer),
        Block::TableCell { content, .. } => write_table_cell("table:table-cell", content, writer),
        Block::TableHeader { content, .. } => {
            write_table_cell("table:table-header-cell", content, writer)
        }
        Block::Image { src, .. } => write_image(src, writer),
        Block::Blockquote { content } => write_blocks(content, writer),
        Block::HorizontalRule => writer
            .write_event(Event::Empty(BytesStart::new("text:p")))
            .map_err(|e| e.to_string()),
    }
}

fn write_paragraph(
    style_name: Option<&str>,
    content: &[Inline],
    writer: &mut XmlWriter,
) -> Result<(), String> {
    let mut p = BytesStart::new("text:p");
    if let Some(s) = style_name {
        p.push_attribute(("text:style-name", s));
    }
    writer
        .write_event(Event::Start(p))
        .map_err(|e| e.to_string())?;
    write_inlines_with_style(content, writer)?;
    writer
        .write_event(Event::End(BytesEnd::new("text:p")))
        .map_err(|e| e.to_string())
}

fn write_heading(
    level: u32,
    style_name: Option<&str>,
    content: &[Inline],
    writer: &mut XmlWriter,
) -> Result<(), String> {
    let mut h = BytesStart::new("text:h");
    if let Some(s) = style_name {
        h.push_attribute(("text:style-name", s));
    }
    h.push_attribute(("text:outline-level", level.to_string().as_str()));
    writer
        .write_event(Event::Start(h))
        .map_err(|e| e.to_string())?;
    write_inlines_with_style(content, writer)?;
    writer
        .write_event(Event::End(BytesEnd::new("text:h")))
        .map_err(|e| e.to_string())
}

fn write_page_break(writer: &mut XmlWriter) -> Result<(), String> {
    let mut p = BytesStart::new("text:p");
    p.push_attribute(("text:style-name", "PageBreak"));
    writer
        .write_event(Event::Empty(p))
        .map_err(|e| e.to_string())
}

fn write_list(items: &[Block], writer: &mut XmlWriter) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("text:list")))
        .map_err(|e| e.to_string())?;
    for item in items {
        write_single_block(item, writer)?;
    }
    writer
        .write_event(Event::End(BytesEnd::new("text:list")))
        .map_err(|e| e.to_string())
}

fn write_list_item(content: &[Block], writer: &mut XmlWriter) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("text:list-item")))
        .map_err(|e| e.to_string())?;
    write_blocks(content, writer)?;
    writer
        .write_event(Event::End(BytesEnd::new("text:list-item")))
        .map_err(|e| e.to_string())
}

fn write_table(content: &[Block], writer: &mut XmlWriter) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("table:table")))
        .map_err(|e| e.to_string())?;
    write_blocks(content, writer)?;
    writer
        .write_event(Event::End(BytesEnd::new("table:table")))
        .map_err(|e| e.to_string())
}

fn write_table_row(content: &[Block], writer: &mut XmlWriter) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new("table:table-row")))
        .map_err(|e| e.to_string())?;
    write_blocks(content, writer)?;
    writer
        .write_event(Event::End(BytesEnd::new("table:table-row")))
        .map_err(|e| e.to_string())
}

fn write_table_cell(
    tag: &'static str,
    content: &[Block],
    writer: &mut XmlWriter,
) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new(tag)))
        .map_err(|e| e.to_string())?;
    write_blocks(content, writer)?;
    writer
        .write_event(Event::End(BytesEnd::new(tag)))
        .map_err(|e| e.to_string())
}

fn write_image(src: &str, writer: &mut XmlWriter) -> Result<(), String> {
    let mut frame = BytesStart::new("draw:frame");
    frame.push_attribute(("draw:name", "Image"));
    writer
        .write_event(Event::Start(frame))
        .map_err(|e| e.to_string())?;
    let mut img = BytesStart::new("draw:image");
    img.push_attribute(("xlink:href", src));
    img.push_attribute(("xlink:type", "simple"));
    img.push_attribute(("xlink:show", "embed"));
    img.push_attribute(("xlink:actuate", "onLoad"));
    writer
        .write_event(Event::Empty(img))
        .map_err(|e| e.to_string())?;
    writer
        .write_event(Event::End(BytesEnd::new("draw:frame")))
        .map_err(|e| e.to_string())
}

/// Writes inline content using style names (for FODT and styles.xml output).
///
/// Wraps styled text runs in `text:span` elements with the ODT style name.
/// Used by the FODT writer where style names are the authoritative source.
pub fn write_inlines_with_style(inlines: &[Inline], writer: &mut XmlWriter) -> Result<(), String> {
    for inline in inlines {
        match inline {
            Inline::Text {
                text, style_name, ..
            } => {
                if let Some(s) = style_name {
                    let mut span = BytesStart::new("text:span");
                    span.push_attribute(("text:style-name", s.as_str()));
                    writer
                        .write_event(Event::Start(span))
                        .map_err(|e| e.to_string())?;
                    writer
                        .write_event(Event::Text(BytesText::new(text)))
                        .map_err(|e| e.to_string())?;
                    writer
                        .write_event(Event::End(BytesEnd::new("text:span")))
                        .map_err(|e| e.to_string())?;
                } else {
                    writer
                        .write_event(Event::Text(BytesText::new(text)))
                        .map_err(|e| e.to_string())?;
                }
            }
            Inline::LineBreak => {
                writer
                    .write_event(Event::Empty(BytesStart::new("text:line-break")))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// Writes inline content using marks (for content.xml output).
///
/// Wraps text runs that have any marks in `text:span` elements.
/// Used by the content.xml writer where marks (bold/italic) are the source.
pub fn write_inlines_with_marks(inlines: &[Inline], writer: &mut XmlWriter) -> Result<(), String> {
    for inline in inlines {
        match inline {
            Inline::Text { text, marks, .. } => {
                let has_marks = !marks.is_empty();
                if has_marks {
                    writer
                        .write_event(Event::Start(BytesStart::new("text:span")))
                        .map_err(|e| e.to_string())?;
                }
                writer
                    .write_event(Event::Text(BytesText::new(text)))
                    .map_err(|e| e.to_string())?;
                if has_marks {
                    writer
                        .write_event(Event::End(BytesEnd::new("text:span")))
                        .map_err(|e| e.to_string())?;
                }
            }
            Inline::LineBreak => {
                writer
                    .write_event(Event::Empty(BytesStart::new("text:line-break")))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
