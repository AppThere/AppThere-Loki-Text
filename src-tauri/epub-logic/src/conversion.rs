use base64::Engine as _;

use common_core::{Block, BlockAttrs, CellAttrs, Inline, TiptapMark, TiptapNode};

use crate::ImageAsset;

// ---------------------------------------------------------------------------
// TiptapNode → Block conversion
// ---------------------------------------------------------------------------

/// Convert a `TiptapNode` tree node into a `Block`.
/// Returns `None` for leaf nodes (Text, HardBreak, Doc).
pub(crate) fn tiptap_node_to_block(node: TiptapNode) -> Option<Block> {
    match node {
        TiptapNode::Paragraph { attrs, content } => {
            let style_name = attrs.as_ref().and_then(|a| a.style_name.clone());
            let block_attrs = attrs.map(|a| BlockAttrs {
                text_align: a.text_align,
                indent: a.indent,
            });
            Some(Block::Paragraph {
                style_name,
                attrs: block_attrs,
                content: tiptap_content_to_inlines(content.unwrap_or_default()),
            })
        }
        TiptapNode::Heading { attrs, content } => {
            let style_name = attrs.as_ref().and_then(|a| a.style_name.clone());
            let level = attrs.as_ref().and_then(|a| a.level).unwrap_or(1);
            let block_attrs = attrs.map(|a| BlockAttrs {
                text_align: a.text_align,
                indent: a.indent,
            });
            Some(Block::Heading {
                level,
                style_name,
                attrs: block_attrs,
                content: tiptap_content_to_inlines(content.unwrap_or_default()),
            })
        }
        TiptapNode::Image { attrs } => Some(Block::Image {
            src: attrs.src,
            alt: attrs.alt,
            title: attrs.title,
        }),
        TiptapNode::BulletList { content } => Some(Block::BulletList {
            content: content
                .into_iter()
                .filter_map(tiptap_node_to_block)
                .collect(),
        }),
        TiptapNode::OrderedList { content } => Some(Block::OrderedList {
            content: content
                .into_iter()
                .filter_map(tiptap_node_to_block)
                .collect(),
        }),
        TiptapNode::ListItem { content } => Some(Block::ListItem {
            content: content
                .into_iter()
                .filter_map(tiptap_node_to_block)
                .collect(),
        }),
        TiptapNode::Blockquote { content } => Some(Block::Blockquote {
            content: content
                .into_iter()
                .filter_map(tiptap_node_to_block)
                .collect(),
        }),
        TiptapNode::Table { content } => Some(Block::Table {
            content: content
                .into_iter()
                .filter_map(tiptap_node_to_block)
                .collect(),
        }),
        TiptapNode::TableRow { content } => Some(Block::TableRow {
            content: content
                .into_iter()
                .filter_map(tiptap_node_to_block)
                .collect(),
        }),
        TiptapNode::TableHeader { attrs, content } => Some(Block::TableHeader {
            attrs,
            content: content
                .into_iter()
                .filter_map(tiptap_node_to_block)
                .collect(),
        }),
        TiptapNode::TableCell { attrs, content } => Some(Block::TableCell {
            attrs,
            content: content
                .into_iter()
                .filter_map(tiptap_node_to_block)
                .collect(),
        }),
        TiptapNode::HorizontalRule => Some(Block::HorizontalRule),
        TiptapNode::PageBreak => Some(Block::PageBreak),
        _ => None,
    }
}

pub(crate) fn tiptap_content_to_inlines(nodes: Vec<TiptapNode>) -> Vec<Inline> {
    let mut inlines = Vec::new();
    for node in nodes {
        match node {
            TiptapNode::Text { text, marks } => {
                inlines.push(Inline::Text {
                    text,
                    style_name: None,
                    marks: marks.unwrap_or_default(),
                });
            }
            TiptapNode::HardBreak => {
                inlines.push(Inline::LineBreak);
            }
            _ => {}
        }
    }
    inlines
}

// ---------------------------------------------------------------------------
// Image asset extraction from Block tree
// ---------------------------------------------------------------------------

/// Walk the block tree and decode every `Block::Image` whose `src` is a
/// data URI.  Returns one `ImageAsset` per decoded image, in document order.
/// File-path and URL srcs are silently skipped (see NEEDS HUMAN REVIEW in
/// the audit summary).
pub(crate) fn extract_images_from_blocks(blocks: &[Block]) -> Vec<ImageAsset> {
    let mut assets = Vec::new();
    let mut counter = 0usize;
    for block in blocks {
        extract_images_from_block(block, &mut assets, &mut counter);
    }
    assets
}

fn extract_images_from_block(
    block: &Block,
    assets: &mut Vec<ImageAsset>,
    counter: &mut usize,
) {
    match block {
        Block::Image { src, .. } => {
            if let Some(asset) = parse_data_uri_image(src, *counter) {
                *counter += 1;
                assets.push(asset);
            }
        }
        Block::Paragraph { content, .. } | Block::Heading { content, .. } => {
            // Inline images are not in the Block::Image path, nothing to do.
            let _ = content;
        }
        Block::BulletList { content }
        | Block::OrderedList { content }
        | Block::ListItem { content }
        | Block::Blockquote { content }
        | Block::Table { content }
        | Block::TableRow { content } => {
            for child in content {
                extract_images_from_block(child, assets, counter);
            }
        }
        Block::TableHeader { content, .. } | Block::TableCell { content, .. } => {
            for child in content {
                extract_images_from_block(child, assets, counter);
            }
        }
        Block::HorizontalRule | Block::PageBreak => {}
    }
}

/// Attempt to decode a `data:<mime>;base64,<payload>` URI.
/// Returns `None` for non-data URIs or malformed payloads.
fn parse_data_uri_image(src: &str, index: usize) -> Option<ImageAsset> {
    let rest = src.strip_prefix("data:")?;
    let comma = rest.find(',')?;
    let header = &rest[..comma];
    let payload = &rest[comma + 1..];

    // Header must contain ";base64" for us to decode it.
    let mime = header.strip_suffix(";base64")?;

    let data = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .ok()?;

    let ext = mime_to_ext(mime);
    let filename = format!("image-{:03}.{}", index, ext);

    Some(ImageAsset {
        original_src: src.to_string(),
        filename,
        data,
        media_type: mime.to_string(),
    })
}

fn mime_to_ext(mime: &str) -> &str {
    match mime {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "image/bmp" => "bmp",
        _ => "bin",
    }
}

// Silence unused-import warnings when the CellAttrs type flows through here
// via tiptap_node_to_block but is never directly referenced in this file.
const _: Option<CellAttrs> = None;
const _: Option<TiptapMark> = None;
