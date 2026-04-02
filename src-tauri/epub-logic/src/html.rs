use std::collections::HashMap;

use common_core::{Block, BlockAttrs, Inline, StyleDefinition, TiptapMark};

/// Mapping from footnote UUID to 1-based sequence number and source section id.
pub(crate) type FootnoteSeqMap = HashMap<String, (usize, String)>;

use crate::{table, ImageAsset};

// ---------------------------------------------------------------------------
// XML / XHTML escaping
// ---------------------------------------------------------------------------

/// Escape the five XML special characters.
pub(crate) fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ---------------------------------------------------------------------------
// Style attribute helpers
// ---------------------------------------------------------------------------

/// Build an inline `style="..."` attribute string from `BlockAttrs`.
/// Returns an empty string when no style properties are needed.
fn build_style_attr(attrs: Option<&BlockAttrs>) -> String {
    let Some(attrs) = attrs else {
        return String::new();
    };
    let mut props: Vec<String> = Vec::new();
    if let Some(ref align) = attrs.text_align {
        props.push(format!("text-align:{}", align));
    }
    if let Some(indent) = attrs.indent {
        if indent > 0 {
            props.push(format!("padding-left:{}em", indent));
        }
    }
    if props.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", props.join(";"))
    }
}

// ---------------------------------------------------------------------------
// Inline rendering (G3, G4, G8, G9)
// ---------------------------------------------------------------------------

/// Render a slice of `Inline` elements to an XHTML fragment.
pub(crate) fn inlines_to_html(inlines: &[Inline], fn_seq: &FootnoteSeqMap) -> String {
    let mut html = String::new();
    for inline in inlines {
        match inline {
            // G3: escape text content before wrapping with marks
            Inline::Text {
                text,
                marks,
                style_name,
            } => {
                let mut content = escape_xml(text);

                // Apply formatting marks (innermost first)
                for mark in marks {
                    content = match mark {
                        TiptapMark::Bold => format!("<strong>{}</strong>", content),
                        TiptapMark::Italic => format!("<em>{}</em>", content),
                        TiptapMark::Underline => format!("<u>{}</u>", content),
                        TiptapMark::Strike => format!("<s>{}</s>", content),
                        TiptapMark::Superscript => format!("<sup>{}</sup>", content),
                        TiptapMark::Subscript => format!("<sub>{}</sub>", content),
                        // G4: escape href
                        TiptapMark::Link { attrs } => {
                            format!("<a href=\"{}\">{}</a>", escape_xml(&attrs.href), content)
                        }
                        // G8: render named character style as CSS class span
                        TiptapMark::NamedSpanStyle { attrs } => {
                            if let Some(ref name) = attrs.style_name {
                                format!(
                                    "<span class=\"style-{}\">{}</span>",
                                    name.replace(' ', "-"),
                                    content
                                )
                            } else {
                                content
                            }
                        }
                    };
                }

                // G9: wrap in character-style span (outermost)
                if let Some(ref name) = style_name {
                    content = format!(
                        "<span class=\"style-{}\">{}</span>",
                        name.replace(' ', "-"),
                        content
                    );
                }

                html.push_str(&content);
            }
            Inline::LineBreak => {
                html.push_str("<br/>");
            }
            Inline::FootnoteRef { id } => {
                if let Some((seq, section_id)) = fn_seq.get(id) {
                    html.push_str(&format!(
                        "<a href=\"notes.xhtml#fn-{seq}\" epub:type=\"noteref\" \
                         role=\"doc-noteref\" id=\"fnref-{seq}\">\
                         <sup>{seq}</sup></a>"
                    ));
                    let _ = section_id; // used in backlinks inside notes.xhtml
                } else {
                    html.push_str("<sup>?</sup>");
                }
            }
        }
    }
    html
}

// ---------------------------------------------------------------------------
// Block rendering (G1, G2, G6, G7)
// ---------------------------------------------------------------------------

/// Render a single `Block` to an XHTML fragment.
pub(crate) fn block_to_html(
    block: &Block,
    styles: &HashMap<String, StyleDefinition>,
    images: &[ImageAsset],
    fn_seq: &FootnoteSeqMap,
) -> String {
    match block {
        // ---- Paragraph (G6, G7) ----
        Block::Paragraph {
            style_name,
            attrs,
            content,
        } => {
            // Promote to heading tag when the named style has an outline level
            let mut tag = "p".to_string();
            if let Some(ref name) = style_name {
                if let Some(style) = styles.get(name) {
                    if let Some(level) = style.outline_level {
                        tag = format!("h{}", level.min(6));
                    }
                }
            }
            let class = style_name
                .as_ref()
                .map(|s| format!(" class=\"style-{}\"", s.replace(' ', "-")))
                .unwrap_or_default();
            let style_attr = build_style_attr(attrs.as_ref());
            format!(
                "  <{}{}{}>{}</{}>\n",
                tag,
                class,
                style_attr,
                inlines_to_html(content, fn_seq),
                tag
            )
        }

        // ---- Heading (G6, G7) ----
        Block::Heading {
            level,
            style_name,
            attrs,
            content,
        } => {
            let tag = format!("h{}", (*level).min(6));
            let class = style_name
                .as_ref()
                .map(|s| format!(" class=\"style-{}\"", s.replace(' ', "-")))
                .unwrap_or_default();
            let style_attr = build_style_attr(attrs.as_ref());
            format!(
                "  <{}{}{}>{}</{}>\n",
                tag,
                class,
                style_attr,
                inlines_to_html(content, fn_seq),
                tag
            )
        }

        // ---- Image (G1) ----
        Block::Image { src, alt, title } => {
            // Resolve to an OPS-relative path when the image was embedded.
            let img_src = images
                .iter()
                .find(|img| &img.original_src == src)
                .map(|img| format!("../Images/{}", img.filename))
                .unwrap_or_else(|| escape_xml(src));

            // alt is always present (empty string when absent) per EPUB spec.
            let alt_text = escape_xml(alt.as_deref().unwrap_or(""));
            let title_attr = title
                .as_ref()
                .map(|t| format!(" title=\"{}\"", escape_xml(t)))
                .unwrap_or_default();

            format!(
                "  <img src=\"{}\" alt=\"{}\"{}/>\n",
                img_src, alt_text, title_attr
            )
        }

        // ---- Lists ----
        Block::BulletList { content } => {
            let mut html = String::from("  <ul>\n");
            for item in content {
                html.push_str(&block_to_html(item, styles, images, fn_seq));
            }
            html.push_str("  </ul>\n");
            html
        }
        Block::OrderedList { content } => {
            let mut html = String::from("  <ol>\n");
            for item in content {
                html.push_str(&block_to_html(item, styles, images, fn_seq));
            }
            html.push_str("  </ol>\n");
            html
        }
        Block::ListItem { content } => {
            let mut html = String::from("    <li>");
            for b in content {
                html.push_str(block_to_html(b, styles, images, fn_seq).trim());
            }
            html.push_str("</li>\n");
            html
        }

        // ---- Blockquote ----
        Block::Blockquote { content } => {
            let mut html = String::from("  <blockquote>\n");
            for b in content {
                html.push_str(&block_to_html(b, styles, images, fn_seq));
            }
            html.push_str("  </blockquote>\n");
            html
        }

        // ---- Table (G2) ----
        Block::Table { content } => table::render_table(content, styles, images, fn_seq),

        Block::TableRow { content } => {
            let mut html = String::from("      <tr>\n");
            for cell in content {
                html.push_str(&block_to_html(cell, styles, images, fn_seq));
            }
            html.push_str("      </tr>\n");
            html
        }

        Block::TableHeader { attrs, content } => {
            table::render_table_cell("th", attrs.as_ref(), content, styles, images, fn_seq)
        }
        Block::TableCell { attrs, content } => {
            table::render_table_cell("td", attrs.as_ref(), content, styles, images, fn_seq)
        }

        Block::HorizontalRule => String::from("  <hr/>\n"),
        Block::PageBreak => String::new(),
    }
}
