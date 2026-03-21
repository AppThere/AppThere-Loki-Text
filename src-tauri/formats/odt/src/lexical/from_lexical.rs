//! Convert a [`LexicalDocument`] from the frontend into an ODT [`Document`].
//!
//! Translates each Lexical custom node type into the corresponding internal
//! [`Block`] or [`Inline`] value. Text format bitmasks are decoded into
//! [`TiptapMark`] slices; link wrapper nodes are flattened into `Link` marks.

use std::collections::HashMap;

use common_core::block::CellAttrs;
use common_core::lexical::{
    LexicalDocument, LexicalNode, FORMAT_BOLD, FORMAT_ITALIC, FORMAT_STRIKETHROUGH,
    FORMAT_SUBSCRIPT, FORMAT_SUPERSCRIPT, FORMAT_UNDERLINE,
};
use common_core::marks::{LinkAttrs, TiptapAttrsInline, TiptapMark};
use common_core::{Block, BlockAttrs, Inline, Metadata, StyleDefinition};

use crate::lexical::style_has_break_before;
use crate::Document;

/// Converts a [`LexicalDocument`] to an ODT [`Document`].
///
/// # Examples
///
/// ```
/// use odt_format::lexical::from_lexical;
/// use common_core::{LexicalDocument, LexicalRoot, Metadata};
/// use std::collections::HashMap;
///
/// let lex = LexicalDocument {
///     root: LexicalRoot {
///         children: vec![],
///         direction: None,
///         format: String::new(),
///         indent: 0,
///         node_type: "root".to_string(),
///         version: 1,
///     },
/// };
/// let doc = from_lexical(lex, HashMap::new(), Metadata::default());
/// assert!(doc.blocks.is_empty());
/// ```
pub fn from_lexical(
    lex: LexicalDocument,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
) -> Document {
    // Use a peekable iterator so we can skip PageBreak nodes that were
    // synthesised by to_lexical for style-based breaks.  A PageBreak node
    // immediately followed by a paragraph/heading whose style already requests
    // fo:break-before is redundant: the style itself will produce the break on
    // export, and writing an explicit PageBreak paragraph as well would result
    // in a double page-break in other ODF consumers.
    let mut iter = lex.root.children.into_iter().peekable();
    let mut blocks = Vec::new();
    while let Some(node) = iter.next() {
        if matches!(node, LexicalNode::PageBreak { .. }) {
            if let Some(next) = iter.peek() {
                let next_style: Option<&str> = match next {
                    LexicalNode::ParagraphStyle { style_name, .. } => {
                        style_name.as_deref()
                    }
                    LexicalNode::HeadingStyle { style_name, .. } => {
                        style_name.as_deref()
                    }
                    _ => None,
                };
                if next_style
                    .map(|s| style_has_break_before(s, &styles))
                    .unwrap_or(false)
                {
                    // Drop this PageBreak; the following block's style handles it.
                    continue;
                }
            }
        }
        if let Some(block) = node_to_block(node) {
            blocks.push(block);
        }
    }
    Document {
        blocks,
        styles,
        metadata,
        font_face_decls: None,
        automatic_styles: None,
        master_styles: None,
    }
}

/// Tries to convert a [`LexicalNode`] to a [`Block`].
///
/// Returns `None` for inline-only node types (text, linebreak) that cannot
/// appear at block level.
pub fn node_to_block(node: LexicalNode) -> Option<Block> {
    match node {
        LexicalNode::ParagraphStyle {
            style_name,
            children,
            format,
            indent,
            ..
        } => Some(Block::Paragraph {
            style_name: style_name.filter(|s| !s.is_empty()),
            attrs: block_attrs(format, indent),
            content: children.into_iter().flat_map(node_to_inlines).collect(),
        }),
        LexicalNode::HeadingStyle {
            tag,
            style_name,
            children,
            format,
            indent,
            ..
        } => {
            let level = tag
                .strip_prefix('h')
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(1)
                .clamp(1, 9);
            Some(Block::Heading {
                level,
                style_name,
                attrs: block_attrs(format, indent),
                content: children.into_iter().flat_map(node_to_inlines).collect(),
            })
        }
        LexicalNode::Image { src, alt_text, .. } => Some(Block::Image {
            src,
            alt: if alt_text.is_empty() {
                None
            } else {
                Some(alt_text)
            },
            title: None,
        }),
        LexicalNode::Table { children, .. } => Some(Block::Table {
            content: children.into_iter().filter_map(node_to_block).collect(),
        }),
        LexicalNode::TableRow { children, .. } => Some(Block::TableRow {
            content: children.into_iter().filter_map(node_to_block).collect(),
        }),
        LexicalNode::TableCell {
            col_span,
            row_span,
            header_state,
            children,
            ..
        } => {
            let attrs = Some(CellAttrs {
                colspan: if col_span == 1 { None } else { Some(col_span) },
                rowspan: if row_span == 1 { None } else { Some(row_span) },
                colwidth: None,
            });
            let content = children.into_iter().filter_map(node_to_block).collect();
            if header_state == 1 {
                Some(Block::TableHeader { attrs, content })
            } else {
                Some(Block::TableCell { attrs, content })
            }
        }
        LexicalNode::List {
            list_type,
            children,
            ..
        } => {
            let items = children.into_iter().filter_map(node_to_block).collect();
            if list_type == "number" {
                Some(Block::OrderedList { content: items })
            } else {
                Some(Block::BulletList { content: items })
            }
        }
        LexicalNode::ListItem { children, .. } => Some(Block::ListItem {
            content: children.into_iter().filter_map(node_to_block).collect(),
        }),
        LexicalNode::Quote { children, .. } => Some(Block::Blockquote {
            content: children.into_iter().filter_map(node_to_block).collect(),
        }),
        LexicalNode::PageBreak { .. } => Some(Block::PageBreak),
        // Inline-only nodes cannot appear at block level
        LexicalNode::Text { .. } | LexicalNode::LineBreak { .. } | LexicalNode::Link { .. } => None,
    }
}

/// Converts a [`LexicalNode`] to zero or more [`Inline`] values.
///
/// Block-level nodes (paragraph, heading, etc.) are ignored here.
pub fn node_to_inlines(node: LexicalNode) -> Vec<Inline> {
    match node {
        LexicalNode::Text {
            text,
            format,
            style_name,
            ..
        } => {
            vec![Inline::Text {
                text,
                style_name: style_name.clone(),
                marks: decode_format(format, style_name),
            }]
        }
        LexicalNode::LineBreak { .. } => vec![Inline::LineBreak],
        LexicalNode::Link {
            url,
            target,
            children,
            ..
        } => {
            let link_mark = TiptapMark::Link {
                attrs: LinkAttrs { href: url, target },
            };
            children
                .into_iter()
                .flat_map(node_to_inlines)
                .map(|inline| match inline {
                    Inline::Text {
                        text,
                        style_name,
                        mut marks,
                    } => {
                        marks.push(link_mark.clone());
                        Inline::Text {
                            text,
                            style_name,
                            marks,
                        }
                    }
                    other => other,
                })
                .collect()
        }
        // Block-level nodes inside inline context → ignore
        _ => vec![],
    }
}

fn block_attrs(format: String, indent: u32) -> Option<BlockAttrs> {
    let text_align = if format.is_empty() {
        None
    } else {
        Some(format)
    };
    let indent_val = if indent == 0 { None } else { Some(indent) };
    if text_align.is_none() && indent_val.is_none() {
        None
    } else {
        Some(BlockAttrs {
            text_align,
            indent: indent_val,
        })
    }
}

pub(crate) fn decode_format(format: u32, style_name: Option<String>) -> Vec<TiptapMark> {
    let mut marks = Vec::new();
    if format & FORMAT_BOLD != 0 {
        marks.push(TiptapMark::Bold);
    }
    if format & FORMAT_ITALIC != 0 {
        marks.push(TiptapMark::Italic);
    }
    if format & FORMAT_STRIKETHROUGH != 0 {
        marks.push(TiptapMark::Strike);
    }
    if format & FORMAT_UNDERLINE != 0 {
        marks.push(TiptapMark::Underline);
    }
    if format & FORMAT_SUBSCRIPT != 0 {
        marks.push(TiptapMark::Subscript);
    }
    if format & FORMAT_SUPERSCRIPT != 0 {
        marks.push(TiptapMark::Superscript);
    }
    if let Some(name) = style_name {
        if !name.is_empty() {
            marks.push(TiptapMark::NamedSpanStyle {
                attrs: TiptapAttrsInline {
                    style_name: Some(name),
                },
            });
        }
    }
    marks
}

#[cfg(test)]
#[path = "from_lexical_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "from_lexical_page_break_tests.rs"]
mod page_break_tests;
