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

//! Paragraph height measurement and content stream emission.

use common_core::block::Block;
use common_core::inline::Inline;
use common_core::style::StyleDefinition;
use std::collections::HashMap;

use super::collector::FontKey;
use super::layout::{break_words, wrap_words_with_indent, LayoutState};
use super::measure::space_width;
use super::operators::write_text_run;
use super::style_props::{resolve_paragraph_props, TextAlign};
use crate::fonts::subset::FontSubset;

/// Compute the total vertical height a block occupies on a page.
pub(super) fn block_height(
    block: &Block,
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    usable_width: f64,
) -> f64 {
    match block {
        Block::Paragraph { .. } | Block::Heading { .. } => {
            let (content, style_name, level) = unpack_para_or_heading(block);
            let props = resolve_paragraph_props(style_name, styles, level);
            let key = (
                props.font_family.to_lowercase(),
                if props.bold { 700 } else { 400 },
                props.italic,
            );
            if let Some((_, subset)) = font_map.get(&key).or_else(|| font_map.values().next()) {
                let full_text = collect_text(content);
                let font_size = props.font_size;
                let sw = space_width(&subset.bytes, font_size);
                let line_height = font_size * props.line_height_factor;
                let base_w = usable_width - props.margin_left - props.margin_right;
                let total_lines = count_wrapped_lines(
                    &full_text,
                    subset,
                    font_size,
                    sw,
                    base_w,
                    props.text_indent,
                );
                props.space_before + (total_lines as f64 * line_height) + props.space_after
            } else {
                12.0
            }
        }
        Block::HorizontalRule => 12.0,
        _ => 0.0,
    }
}

/// Emit the content lines of a paragraph or heading to the PDF stream.
///
/// Returns the number of lines written. Stops early and sets `overflowed` if
/// the page bottom is reached before all lines are placed.
pub(super) fn emit_para_content(
    content: &[Inline],
    props: super::style_props::ParagraphProps,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    state: &mut LayoutState,
    overflowed: &mut bool,
    out: &mut String,
    start_line_idx: usize,
) -> usize {
    let full_text = collect_text(content);
    let key = (
        props.font_family.to_lowercase(),
        if props.bold { 700 } else { 400 },
        props.italic,
    );
    let (pdf_name, subset) = match font_map
        .get(&key)
        .or_else(|| font_map.get(&(props.font_family.to_lowercase(), 400, false)))
        .or_else(|| font_map.values().next())
    {
        Some(v) => v,
        None => return 0,
    };

    let font_size = props.font_size;
    let sw = space_width(&subset.bytes, font_size);
    let line_height = font_size * props.line_height_factor;
    let base_w = state.usable_width - props.margin_left - props.margin_right;
    let line_count = full_text.lines().count();

    let mut lines_emitted = 0;
    let mut total_lines_processed = 0;

    for (p_idx, line_text) in full_text.lines().enumerate() {
        let is_last_chunk = p_idx == line_count - 1;
        let is_first_chunk = p_idx == 0 && start_line_idx == 0;
        let first_w = if is_first_chunk {
            base_w - props.text_indent
        } else {
            base_w
        };

        let words = break_words(line_text, subset, font_size);
        let wrapped = wrap_words_with_indent(words, sw, first_w, base_w);
        let total_wrapped = wrapped.len();

        for (l_idx, line_words) in wrapped.into_iter().enumerate() {
            if total_lines_processed < start_line_idx {
                total_lines_processed += 1;
                continue;
            }
            if state.current_y_from_top + line_height > state.page_height - state.bottom_margin {
                *overflowed = true;
                return lines_emitted;
            }

            let is_first_line = is_first_chunk && l_idx == 0;
            let is_last_line = is_last_chunk && l_idx == total_wrapped - 1;
            let line_w = if is_first_line { first_w } else { base_w };
            let text_w: f64 = line_words.iter().map(|w| w.width).sum();
            let spaces = if line_words.len() > 1 {
                (line_words.len() - 1) as f64
            } else {
                0.0
            };
            let total_w = text_w + spaces * sw;

            let (x_offset, word_spacing) = compute_alignment(
                props.text_align,
                is_last_line,
                line_words.len(),
                line_w,
                total_w,
                spaces,
                sw,
            );

            let line_str: String = line_words
                .iter()
                .enumerate()
                .map(|(i, w)| {
                    if i == 0 {
                        w.text.clone()
                    } else {
                        format!(" {}", w.text)
                    }
                })
                .collect();

            let x = state.left_margin
                + props.margin_left
                + (if is_first_line {
                    props.text_indent
                } else {
                    0.0
                })
                + x_offset;
            let y = state.pdf_y() - font_size;
            write_text_run(
                &line_str,
                subset,
                pdf_name,
                font_size,
                x,
                y,
                0.0,
                0.0,
                0.0,
                word_spacing,
                out,
            );
            state.advance(line_height);
            lines_emitted += 1;
            total_lines_processed += 1;
        }
    }
    lines_emitted
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Collect all inline text into a single string, treating LineBreak as `\n`.
fn collect_text(content: &[Inline]) -> String {
    content
        .iter()
        .map(|i| match i {
            Inline::Text { text, .. } => text.as_str(),
            Inline::LineBreak => "\n",
            Inline::FootnoteRef { .. } => "",
        })
        .collect()
}

/// Count the total number of wrapped lines for a block of text.
fn count_wrapped_lines(
    full_text: &str,
    subset: &FontSubset,
    font_size: f64,
    sw: f64,
    base_w: f64,
    text_indent: f64,
) -> usize {
    let mut total = 0;
    for (p_idx, line_text) in full_text.lines().enumerate() {
        let first_w = if p_idx == 0 {
            base_w - text_indent
        } else {
            base_w
        };
        let words = break_words(line_text, subset, font_size);
        total += wrap_words_with_indent(words, sw, first_w, base_w).len();
    }
    total
}

/// Compute x-axis alignment offset and word spacing for a line.
fn compute_alignment(
    align: TextAlign,
    is_last_line: bool,
    word_count: usize,
    line_usable_w: f64,
    total_w: f64,
    spaces: f64,
    _sw: f64,
) -> (f64, f64) {
    match align {
        TextAlign::Center => ((line_usable_w - total_w) / 2.0, 0.0),
        TextAlign::Right => (line_usable_w - total_w, 0.0),
        TextAlign::Justify if !is_last_line && word_count > 1 => {
            (0.0, (line_usable_w - total_w) / spaces)
        }
        _ => (0.0, 0.0),
    }
}

/// Destructure a `Paragraph` or `Heading` block into `(content, style_name, level)`.
pub(super) fn unpack_para_or_heading(block: &Block) -> (&[Inline], Option<&str>, Option<u32>) {
    match block {
        Block::Paragraph {
            content,
            style_name,
            ..
        } => (content, style_name.as_deref(), None),
        Block::Heading {
            content,
            style_name,
            level,
            ..
        } => (content, style_name.as_deref(), Some(*level)),
        _ => unreachable!("unpack_para_or_heading called on non-paragraph/heading block"),
    }
}
