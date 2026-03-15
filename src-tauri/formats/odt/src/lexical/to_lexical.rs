//! Convert [`Document`] to a [`LexicalDocument`] for the frontend editor.
//!
//! Translates the internal block/inline model into the JSON structure that the
//! Lexical editor expects, matching the custom node types registered in the
//! TypeScript frontend (`paragraph-style`, `heading-style`, etc.).

use common_core::block::CellAttrs;
use common_core::lexical::{
    LexicalDocument, LexicalNode, LexicalRoot, FORMAT_BOLD, FORMAT_ITALIC, FORMAT_STRIKETHROUGH,
    FORMAT_SUBSCRIPT, FORMAT_SUPERSCRIPT, FORMAT_UNDERLINE,
};
use common_core::marks::TiptapMark;
use common_core::{Block, Inline};

use crate::Document;

/// Converts an ODT [`Document`] to a [`LexicalDocument`] for the frontend.
///
/// # Examples
///
/// ```
/// use odt_format::lexical::to_lexical;
/// use odt_format::Document;
///
/// let doc = Document::new();
/// let lex = to_lexical(&doc);
/// assert_eq!(lex.root.node_type, "root");
/// ```
pub fn to_lexical(doc: &Document) -> LexicalDocument {
    let children = doc.blocks.iter().map(block_to_node).collect();
    LexicalDocument {
        root: LexicalRoot {
            children,
            direction: None,
            format: String::new(),
            indent: 0,
            node_type: "root".to_string(),
            version: 1,
        },
    }
}

/// Converts a single [`Block`] to a [`LexicalNode`].
pub fn block_to_node(block: &Block) -> LexicalNode {
    match block {
        Block::Paragraph {
            style_name,
            attrs,
            content,
        } => LexicalNode::ParagraphStyle {
            style_name: style_name.clone().unwrap_or_else(|| "Standard".to_string()),
            children: inlines_to_nodes(content),
            direction: None,
            format: attrs
                .as_ref()
                .and_then(|a| a.text_align.clone())
                .unwrap_or_default(),
            indent: attrs.as_ref().and_then(|a| a.indent).unwrap_or(0),
            version: 1,
        },
        Block::Heading {
            level,
            style_name,
            attrs,
            content,
        } => LexicalNode::HeadingStyle {
            tag: format!("h{}", level.min(&6)),
            style_name: style_name.clone(),
            children: inlines_to_nodes(content),
            direction: None,
            format: attrs
                .as_ref()
                .and_then(|a| a.text_align.clone())
                .unwrap_or_default(),
            indent: attrs.as_ref().and_then(|a| a.indent).unwrap_or(0),
            version: 1,
        },
        Block::Image { src, alt, title } => LexicalNode::Image {
            src: src.clone(),
            alt_text: alt.clone().or_else(|| title.clone()).unwrap_or_default(),
            version: 1,
        },
        Block::BulletList { content } => LexicalNode::List {
            list_type: "bullet".to_string(),
            start: 1,
            tag: "ul".to_string(),
            children: content.iter().map(block_to_node).collect(),
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        },
        Block::OrderedList { content } => LexicalNode::List {
            list_type: "number".to_string(),
            start: 1,
            tag: "ol".to_string(),
            children: content.iter().map(block_to_node).collect(),
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        },
        Block::ListItem { content } => LexicalNode::ListItem {
            value: 1,
            children: content.iter().map(block_to_node).collect(),
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        },
        Block::Blockquote { content } => LexicalNode::Quote {
            children: content.iter().map(block_to_node).collect(),
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        },
        Block::Table { content } => LexicalNode::Table {
            children: content.iter().map(block_to_node).collect(),
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        },
        Block::TableRow { content } => LexicalNode::TableRow {
            children: content.iter().map(block_to_node).collect(),
            direction: None,
            format: String::new(),
            indent: 0,
            version: 1,
        },
        Block::TableHeader { attrs, content } => table_cell_node(attrs, content, true),
        Block::TableCell { attrs, content } => table_cell_node(attrs, content, false),
        Block::HorizontalRule => {
            // Represent as empty paragraph – Lexical has no native HR block
            LexicalNode::ParagraphStyle {
                style_name: "Horizontal Line".to_string(),
                children: vec![],
                direction: None,
                format: String::new(),
                indent: 0,
                version: 1,
            }
        }
        Block::PageBreak => LexicalNode::PageBreak { version: 1 },
    }
}

fn table_cell_node(attrs: &Option<CellAttrs>, content: &[Block], is_header: bool) -> LexicalNode {
    LexicalNode::TableCell {
        col_span: attrs.as_ref().and_then(|a| a.colspan).unwrap_or(1),
        row_span: attrs.as_ref().and_then(|a| a.rowspan).unwrap_or(1),
        header_state: if is_header { 1 } else { 0 },
        children: content.iter().map(block_to_node).collect(),
        direction: None,
        format: String::new(),
        indent: 0,
        version: 1,
    }
}

/// Converts a slice of [`Inline`] values to Lexical inline nodes.
///
/// Link marks are hoisted into `LexicalNode::Link` wrappers.
pub fn inlines_to_nodes(inlines: &[Inline]) -> Vec<LexicalNode> {
    let mut out = Vec::with_capacity(inlines.len());
    for inline in inlines {
        match inline {
            Inline::Text {
                text,
                marks,
                style_name,
            } => {
                let link = marks.iter().find_map(|m| {
                    if let TiptapMark::Link { attrs } = m {
                        Some(attrs.clone())
                    } else {
                        None
                    }
                });
                let mut format: u32 = 0;
                let mut span_style: Option<String> = style_name.clone();
                for mark in marks {
                    match mark {
                        TiptapMark::Bold => format |= FORMAT_BOLD,
                        TiptapMark::Italic => format |= FORMAT_ITALIC,
                        TiptapMark::Strike => format |= FORMAT_STRIKETHROUGH,
                        TiptapMark::Underline => format |= FORMAT_UNDERLINE,
                        TiptapMark::Subscript => format |= FORMAT_SUBSCRIPT,
                        TiptapMark::Superscript => format |= FORMAT_SUPERSCRIPT,
                        TiptapMark::NamedSpanStyle { attrs } => {
                            span_style = attrs.style_name.clone();
                        }
                        TiptapMark::Link { .. } => {}
                    }
                }
                let text_node = LexicalNode::Text {
                    text: text.clone(),
                    format,
                    style: String::new(),
                    mode: "normal".to_string(),
                    detail: 0,
                    style_name: span_style,
                    version: 1,
                };
                if let Some(link_attrs) = link {
                    out.push(LexicalNode::Link {
                        url: link_attrs.href.clone(),
                        target: link_attrs.target.clone(),
                        rel: Some("noopener noreferrer".to_string()),
                        children: vec![text_node],
                        direction: None,
                        format: String::new(),
                        indent: 0,
                        version: 1,
                    });
                } else {
                    out.push(text_node);
                }
            }
            Inline::LineBreak => out.push(LexicalNode::LineBreak { version: 1 }),
        }
    }
    out
}

#[cfg(test)]
#[path = "to_lexical_tests.rs"]
mod tests;
