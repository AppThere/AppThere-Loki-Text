// Copyright 2024 AppThere
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Text document layout and PDF content stream generation.
//!
//! # Architecture
//!
//! Two-pass design:
//! 1. **Pass 1** — `collect_used_glyphs`: walks all blocks/inlines and groups
//!    Unicode characters by the font variant they require.
//! 2. **Pass 2** — `emit_blocks`: generates the PDF content stream using
//!    pre-embedded font subsets from Pass 1.

mod layout;
mod measure;
mod operators;

use std::collections::HashMap;

use common_core::block::Block;
use common_core::inline::Inline;
use common_core::marks::TiptapMark;
use common_core::style::StyleDefinition;

use crate::error::PdfError;
use crate::fonts::subset::{FontSubset, UsedGlyphs};
use layout::{LayoutState, break_words, wrap_words};
use measure::space_width;
use operators::{write_horizontal_rule, write_text_run};

// ---------------------------------------------------------------------------
// Font key and resolution helpers
// ---------------------------------------------------------------------------

/// (family_name_lowercase, bold, italic)
pub type FontKey = (String, bool, bool);

/// Default font family used when a style does not specify one.
const DEFAULT_FONT_FAMILY: &str = "public sans";

/// Extract the font key for an inline based on block style + marks.
fn inline_font_key(
    marks: &[TiptapMark],
    style_name: Option<&str>,
    styles: &HashMap<String, StyleDefinition>,
    block_style_name: Option<&str>,
) -> FontKey {
    let is_bold = marks.iter().any(|m| matches!(m, TiptapMark::Bold));
    let is_italic = marks.iter().any(|m| matches!(m, TiptapMark::Italic));

    // Determine font family: inline style → block style → default.
    let family = style_name
        .and_then(|n| styles.get(n))
        .and_then(|s| s.attributes.get("fo:font-family").or_else(|| s.attributes.get("style:font-name")))
        .or_else(|| {
            block_style_name
                .and_then(|n| styles.get(n))
                .and_then(|s| s.attributes.get("fo:font-family").or_else(|| s.attributes.get("style:font-name")))
        })
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_FONT_FAMILY);

    (family.to_lowercase(), is_bold, is_italic)
}

// ---------------------------------------------------------------------------
// Pass 1: collect used glyphs
// ---------------------------------------------------------------------------

/// Walk all blocks and collect the set of Unicode characters used, grouped by
/// the font variant (family + bold + italic) they require.
pub fn collect_used_glyphs(
    blocks: &[Block],
    styles: &HashMap<String, StyleDefinition>,
) -> HashMap<FontKey, UsedGlyphs> {
    let mut result: HashMap<FontKey, UsedGlyphs> = HashMap::new();
    for block in blocks {
        collect_from_block(block, styles, &mut result);
    }
    result
}

fn collect_from_block(
    block: &Block,
    styles: &HashMap<String, StyleDefinition>,
    out: &mut HashMap<FontKey, UsedGlyphs>,
) {
    match block {
        Block::Paragraph { style_name, content, .. }
        | Block::Heading { style_name, content, .. } => {
            let sname = style_name.as_deref();
            for inline in content {
                collect_from_inline(inline, sname, styles, out);
            }
        }
        Block::BulletList { content }
        | Block::OrderedList { content }
        | Block::ListItem { content }
        | Block::Blockquote { content }
        | Block::Table { content }
        | Block::TableRow { content }
        | Block::TableHeader { content, .. }
        | Block::TableCell { content, .. } => {
            for child in content {
                collect_from_block(child, styles, out);
            }
        }
        Block::HorizontalRule | Block::PageBreak | Block::Image { .. } => {}
    }
}

fn collect_from_inline(
    inline: &Inline,
    block_style: Option<&str>,
    styles: &HashMap<String, StyleDefinition>,
    out: &mut HashMap<FontKey, UsedGlyphs>,
) {
    if let Inline::Text { text, style_name, marks } = inline {
        let key = inline_font_key(marks, style_name.as_deref(), styles, block_style);
        out.entry(key).or_default().extend(text.chars());
    }
}

// ---------------------------------------------------------------------------
// Pass 2: emit blocks to content stream
// ---------------------------------------------------------------------------

/// Emit all blocks to the PDF content stream string.
///
/// `font_map` maps `FontKey → (pdf_resource_name, FontSubset)`.
///
/// # Phase 7 limitations
/// - Single-page output only. Overflowing content is clipped with a warning.
/// - Tables render as plain text (borders are omitted for Phase 7).
pub fn emit_blocks(
    blocks: &[Block],
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    page_width: f64,
    page_height: f64,
    margin: f64,
    out: &mut String,
) -> Result<(), PdfError> {
    let mut state = LayoutState::new(page_width, page_height, margin);
    let mut overflowed = false;

    for block in blocks {
        if overflowed {
            break;
        }
        emit_block(block, styles, font_map, &mut state, &mut overflowed, out);
    }

    if overflowed {
        eprintln!(
            "[loki-pdf] WARNING: Text document content overflowed one A4 page. \
             Remaining content was clipped. Multi-page support is Phase 8."
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Block-level rendering
// ---------------------------------------------------------------------------

fn emit_block(
    block: &Block,
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    state: &mut LayoutState,
    overflowed: &mut bool,
    out: &mut String,
) {
    match block {
        Block::Paragraph { style_name, content, .. } => {
            let (family, size, _align) = resolve_para_style(style_name.as_deref(), styles);
            emit_para_content(content, style_name.as_deref(), styles, font_map,
                &family, size, state, overflowed, out);
            // Paragraph spacing after.
            if state.advance(size * 0.4) && !*overflowed {
                *overflowed = true;
            }
        }
        Block::Heading { level, style_name, content, .. } => {
            let size = heading_size(*level);
            let (family, _, _) = resolve_para_style(style_name.as_deref(), styles);
            // Extra space before headings.
            if state.advance(size * 0.6) && !*overflowed { *overflowed = true; return; }
            emit_inline_line(content, style_name.as_deref(), styles, font_map,
                &family, size, true, state, overflowed, out);
            if state.advance(size * 0.4) && !*overflowed { *overflowed = true; }
        }
        Block::HorizontalRule => {
            let y = state.pdf_y();
            write_horizontal_rule(state.left_margin, y, state.usable_width, out);
            if state.advance(12.0) && !*overflowed { *overflowed = true; }
        }
        Block::PageBreak => {
            // Phase 7: single page — treat as end of content.
            *overflowed = true;
        }
        Block::BulletList { content } => {
            for item in content {
                emit_list_item(item, "•", styles, font_map, state, overflowed, out);
            }
        }
        Block::OrderedList { content } => {
            for (i, item) in content.iter().enumerate() {
                let bullet = format!("{}.", i + 1);
                emit_list_item(item, &bullet, styles, font_map, state, overflowed, out);
            }
        }
        Block::ListItem { content } => {
            for child in content {
                emit_block(child, styles, font_map, state, overflowed, out);
            }
        }
        Block::Table { content } | Block::Blockquote { content } => {
            for child in content {
                emit_block(child, styles, font_map, state, overflowed, out);
            }
        }
        Block::TableRow { content } | Block::TableHeader { content, .. } | Block::TableCell { content, .. } => {
            for child in content {
                emit_block(child, styles, font_map, state, overflowed, out);
            }
        }
        Block::Image { .. } => {
            eprintln!("[loki-pdf] WARNING: Image blocks are not yet supported in text PDF export (Phase 8).");
        }
    }
}

fn emit_list_item(
    block: &Block,
    bullet: &str,
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    state: &mut LayoutState,
    overflowed: &mut bool,
    out: &mut String,
) {
    let indent = 20.0;
    let font_size = 11.0;
    let family = DEFAULT_FONT_FAMILY.to_string();
    let key: FontKey = (family.clone(), false, false);
    if let Some((pdf_name, subset)) = font_map.get(&key).or_else(|| font_map.values().next()) {
        let x = state.left_margin;
        let y = state.pdf_y() - font_size;
        write_text_run(bullet, subset, pdf_name, font_size, x, y, 0.0, 0.0, 0.0, out);
    }
    // Indent inline content.
    let orig_left = state.left_margin;
    let orig_width = state.usable_width;
    // Temporarily borrow via direct mutation
    state.left_margin += indent;
    state.usable_width -= indent;
    emit_block(block, styles, font_map, state, overflowed, out);
    state.left_margin = orig_left;
    state.usable_width = orig_width;
}

// ---------------------------------------------------------------------------
// Inline rendering helpers
// ---------------------------------------------------------------------------

fn emit_para_content(
    content: &[Inline],
    block_style: Option<&str>,
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    default_family: &str,
    font_size: f64,
    state: &mut LayoutState,
    overflowed: &mut bool,
    out: &mut String,
) {
    // Concatenate all text in the paragraph and wrap as one flow.
    // For Phase 7, we treat the paragraph as a single run (ignoring per-inline
    // bold/italic mid-paragraph switching to keep complexity bounded).
    let full_text: String = content.iter().map(|i| match i {
        Inline::Text { text, .. } => text.as_str(),
        Inline::LineBreak => "\n",
    }).collect();

    // Determine font for this paragraph.
    let key = resolve_font_key_for_para(content, block_style, styles, default_family);
    let (pdf_name, subset) = match font_map.get(&key)
        .or_else(|| font_map.values().next()) {
        Some(v) => v,
        None => return,
    };

    let sw = space_width(&subset.bytes, font_size);
    let line_height = font_size * 1.35;

    for line_text in full_text.lines() {
        if line_text.is_empty() {
            if state.advance(line_height) && !*overflowed { *overflowed = true; return; }
            continue;
        }
        let words = break_words(line_text, subset, font_size);
        let wrapped_lines = wrap_words(words, sw, state.usable_width);

        for line_words in wrapped_lines {
            if *overflowed { return; }
            let line_str: String = line_words.iter().enumerate().map(|(i, w)| {
                if i == 0 { w.text.clone() } else { format!(" {}", w.text) }
            }).collect();
            let x = state.left_margin;
            let y = state.pdf_y() - font_size;
            write_text_run(&line_str, subset, pdf_name, font_size, x, y, 0.0, 0.0, 0.0, out);
            if state.advance(line_height) && !*overflowed { *overflowed = true; return; }
        }
    }
}

fn emit_inline_line(
    content: &[Inline],
    block_style: Option<&str>,
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    default_family: &str,
    font_size: f64,
    _bold: bool,
    state: &mut LayoutState,
    overflowed: &mut bool,
    out: &mut String,
) {
    let text: String = content.iter().map(|i| match i {
        Inline::Text { text, .. } => text.as_str(),
        Inline::LineBreak => " ",
    }).collect();
    let key: FontKey = (default_family.to_lowercase(), true, false);
    let (pdf_name, subset) = match font_map.get(&key)
        .or_else(|| font_map.get(&(default_family.to_lowercase(), false, false)))
        .or_else(|| font_map.values().next()) {
        Some(v) => v,
        None => return,
    };
    let line_height = font_size * 1.35;
    let x = state.left_margin;
    let y = state.pdf_y() - font_size;
    write_text_run(&text, subset, pdf_name, font_size, x, y, 0.0, 0.0, 0.0, out);
    if state.advance(line_height) && !*overflowed { *overflowed = true; }
    let _ = block_style;
    let _ = styles;
}

// ---------------------------------------------------------------------------
// Style resolution helpers
// ---------------------------------------------------------------------------

fn resolve_para_style(
    style_name: Option<&str>,
    styles: &HashMap<String, StyleDefinition>,
) -> (String, f64, String) {
    let style = style_name.and_then(|n| styles.get(n));
    let family = style
        .and_then(|s| s.attributes.get("fo:font-family").or_else(|| s.attributes.get("style:font-name")))
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| DEFAULT_FONT_FAMILY.to_string());
    let size = style
        .and_then(|s| s.attributes.get("fo:font-size"))
        .and_then(|v| parse_pt(v))
        .unwrap_or(11.0);
    let align = style
        .and_then(|s| s.attributes.get("fo:text-align"))
        .cloned()
        .unwrap_or_else(|| "left".to_string());
    (family, size, align)
}

fn resolve_font_key_for_para(
    content: &[Inline],
    block_style: Option<&str>,
    styles: &HashMap<String, StyleDefinition>,
    default_family: &str,
) -> FontKey {
    // Use the first text inline's style/marks to determine font key.
    if let Some(Inline::Text { style_name, marks, .. }) = content.first() {
        inline_font_key(marks, style_name.as_deref(), styles, block_style)
    } else {
        (default_family.to_lowercase(), false, false)
    }
}

fn heading_size(level: u32) -> f64 {
    match level {
        1 => 24.0,
        2 => 18.0,
        3 => 14.0,
        4 => 12.0,
        _ => 11.0,
    }
}

/// Parse a CSS-style point/pt value like "12pt" or "12".
fn parse_pt(s: &str) -> Option<f64> {
    s.trim_end_matches("pt").trim().parse::<f64>().ok()
}
