//! Inline XML writers for ODT output.
//!
//! Provides two flavours of inline serialization:
//!
//! - [`write_inlines_with_style`]: uses named `text:span` style names (FODT / styles.xml path)
//! - [`write_inlines_with_marks`]: uses mark-derived style names (content.xml path)

use std::io::Cursor;

use common_core::marks::TiptapMark;
use common_core::Inline;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

/// Shared XML writer type used by all ODT writer modules.
pub type XmlWriter = Writer<Cursor<Vec<u8>>>;

/// Writes inline content using style names (FODT / styles.xml path).
///
/// Wraps styled text runs in `text:span` elements with the ODT style name.
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
            // Footnote references are not yet supported in ODT output; skip.
            Inline::FootnoteRef { .. } => {}
        }
    }
    Ok(())
}

/// Writes inline content using marks (content.xml path).
///
/// Wraps text runs that carry marks in `text:span` / `text:a` elements,
/// opening tags first, then the text, then closing tags in reverse order.
pub fn write_inlines_with_marks(inlines: &[Inline], writer: &mut XmlWriter) -> Result<(), String> {
    for inline in inlines {
        match inline {
            Inline::Text { text, marks, .. } => {
                write_mark_open_tags(marks, writer)?;
                writer
                    .write_event(Event::Text(BytesText::new(text)))
                    .map_err(|e| e.to_string())?;
                write_mark_close_tags(marks, writer)?;
            }
            Inline::LineBreak => {
                writer
                    .write_event(Event::Empty(BytesStart::new("text:line-break")))
                    .map_err(|e| e.to_string())?;
            }
            // Footnote references are not yet supported in ODT output; skip.
            Inline::FootnoteRef { .. } => {}
        }
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn write_mark_open_tags(marks: &[TiptapMark], writer: &mut XmlWriter) -> Result<(), String> {
    for mark in marks {
        match mark {
            TiptapMark::Bold => open_span("Strong", writer)?,
            TiptapMark::Italic => open_span("Emphasis", writer)?,
            TiptapMark::Underline => open_span("Underline", writer)?,
            TiptapMark::Strike => open_span("Strike", writer)?,
            TiptapMark::Superscript => open_span("Superscript", writer)?,
            TiptapMark::Subscript => open_span("Subscript", writer)?,
            TiptapMark::NamedSpanStyle { attrs } => {
                if let Some(name) = &attrs.style_name {
                    open_span(name, writer)?;
                }
            }
            TiptapMark::Link { attrs } => {
                let mut a = BytesStart::new("text:a");
                a.push_attribute(("xlink:type", "simple"));
                a.push_attribute(("xlink:href", attrs.href.as_str()));
                if let Some(t) = &attrs.target {
                    a.push_attribute(("office:target-frame-name", t.as_str()));
                }
                writer
                    .write_event(Event::Start(a))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

fn write_mark_close_tags(marks: &[TiptapMark], writer: &mut XmlWriter) -> Result<(), String> {
    for mark in marks.iter().rev() {
        match mark {
            TiptapMark::Link { .. } => {
                writer
                    .write_event(Event::End(BytesEnd::new("text:a")))
                    .map_err(|e| e.to_string())?;
            }
            TiptapMark::NamedSpanStyle { attrs } => {
                if attrs.style_name.is_some() {
                    writer
                        .write_event(Event::End(BytesEnd::new("text:span")))
                        .map_err(|e| e.to_string())?;
                }
            }
            _ => {
                writer
                    .write_event(Event::End(BytesEnd::new("text:span")))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

fn open_span(style: &str, writer: &mut XmlWriter) -> Result<(), String> {
    let mut span = BytesStart::new("text:span");
    span.push_attribute(("text:style-name", style));
    writer
        .write_event(Event::Start(span))
        .map_err(|e| e.to_string())
}
