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

use common_core::block::Block;
use common_core::inline::Inline;
use common_core::style::StyleDefinition;
use std::collections::HashMap;

use super::collector::FontKey;
use super::layout::{break_words, wrap_words_with_indent, LayoutState, PageContent};
use super::measure::space_width;
use super::operators::write_horizontal_rule;
use super::para::{block_height, emit_para_content, unpack_para_or_heading};
use super::style_props::{resolve_paragraph_props, ParagraphProps};
use crate::error::PdfError;
use crate::fonts::subset::FontSubset;

/// Emit all blocks to potentially multiple PDF page content streams.
pub struct LayoutResult {
    pub pages: Vec<PageContent>,
}

pub fn emit_blocks(
    blocks: &[Block],
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    page_width: f64,
    page_height: f64,
    margin: f64,
) -> Result<LayoutResult, PdfError> {
    let mut pages = Vec::new();
    let mut current_block_idx = 0;
    let mut current_line_offset = 0;

    while current_block_idx < blocks.len() {
        let mut state = LayoutState::new(page_width, page_height, margin);
        let mut overflowed = false;
        let mut content_stream = String::new();

        let mut page_end_block_idx = current_block_idx;
        let mut next_line_offset = 0;

        for (i, block) in blocks.iter().enumerate().skip(current_block_idx) {
            let start_offset = if i == current_block_idx {
                current_line_offset
            } else {
                0
            };

            // Look-ahead for break-before or keep-with-next
            if start_offset == 0 && state.current_y_from_top > state._top_margin {
                let props = get_block_props(block, styles);
                if props.break_before {
                    overflowed = true;
                    page_end_block_idx = i;
                    break;
                }

                if props.keep_with_next && i + 1 < blocks.len() {
                    let next_block = &blocks[i + 1];
                    let current_h = block_height(block, styles, font_map, state.usable_width);
                    let next_h = block_height(next_block, styles, font_map, state.usable_width);

                    if state.current_y_from_top + current_h + next_h
                        > state.page_height - state.bottom_margin
                    {
                        overflowed = true;
                        page_end_block_idx = i;
                        break;
                    }
                }
            }

            let lines_emitted = emit_block(
                block,
                styles,
                font_map,
                &mut state,
                &mut overflowed,
                &mut content_stream,
                start_offset,
            );

            page_end_block_idx = i;
            if overflowed {
                next_line_offset = start_offset + lines_emitted;
                if is_finished(block, next_line_offset, styles, font_map) {
                    next_line_offset = 0;
                    page_end_block_idx = i;
                } else {
                    break;
                }
                break;
            }
        }

        pages.push(PageContent { content_stream });

        if overflowed && next_line_offset > 0 {
            current_block_idx = page_end_block_idx;
            current_line_offset = next_line_offset;
        } else {
            current_block_idx = page_end_block_idx + 1;
            current_line_offset = 0;
        }
    }

    Ok(LayoutResult { pages })
}

fn is_finished(
    block: &Block,
    offset: usize,
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
) -> bool {
    match block {
        Block::PageBreak => true,
        Block::Paragraph { .. } | Block::Heading { .. } => {
            let (content, style_name, level) = unpack_para_or_heading(block);
            let props = resolve_paragraph_props(style_name, styles, level);
            let key = (props.font_family.to_lowercase(), 400, false);
            if let Some((_, subset)) = font_map.get(&key).or_else(|| font_map.values().next()) {
                let full_text: String = content
                    .iter()
                    .map(|i| match i {
                        Inline::Text { text, .. } => text.as_str(),
                        Inline::LineBreak => "\n",
                        Inline::FootnoteRef { .. } => "",
                    })
                    .collect();
                let font_size = props.font_size;
                let sw = space_width(&subset.bytes, font_size);

                let base_usable_width = (595.0 - 144.0) - props.margin_left - props.margin_right;
                let mut total_lines = 0;
                for (p_idx, line_text) in full_text.lines().enumerate() {
                    let first_line_width = if p_idx == 0 {
                        base_usable_width - props.text_indent
                    } else {
                        base_usable_width
                    };
                    let words = break_words(line_text, subset, font_size);
                    total_lines +=
                        wrap_words_with_indent(words, sw, first_line_width, base_usable_width)
                            .len();
                }
                offset >= total_lines
            } else {
                true
            }
        }
        _ => true,
    }
}

fn get_block_props(block: &Block, styles: &HashMap<String, StyleDefinition>) -> ParagraphProps {
    match block {
        Block::Paragraph { style_name, .. } => {
            resolve_paragraph_props(style_name.as_deref(), styles, None)
        }
        Block::Heading {
            level, style_name, ..
        } => resolve_paragraph_props(style_name.as_deref(), styles, Some(*level)),
        _ => ParagraphProps::default(),
    }
}

fn emit_block(
    block: &Block,
    styles: &HashMap<String, StyleDefinition>,
    font_map: &HashMap<FontKey, (String, FontSubset)>,
    state: &mut LayoutState,
    overflowed: &mut bool,
    out: &mut String,
    start_offset: usize,
) -> usize {
    match block {
        Block::Paragraph { .. } | Block::Heading { .. } => {
            let (content, style_name, level) = unpack_para_or_heading(block);
            let props = resolve_paragraph_props(style_name, styles, level);

            if start_offset == 0 {
                let h = block_height(block, styles, font_map, state.usable_width);
                let line_h = props.font_size * props.line_height_factor;
                let content_h = h - props.space_before - props.space_after;
                let total_lines = (content_h / line_h).max(1.0).round() as usize;
                let min_lines = if total_lines >= 2 { 2 } else { 1 };
                let min_h = props.space_before + (min_lines as f64 * line_h);

                if state.current_y_from_top + min_h > state.page_height - state.bottom_margin
                    && state.current_y_from_top > state._top_margin
                {
                    *overflowed = true;
                    return 0;
                }

                if props.space_before > 0.0 {
                    if state.advance(props.space_before) {
                        *overflowed = true;
                        return 0;
                    }
                }
            }

            let emitted = emit_para_content(
                content,
                props,
                font_map,
                state,
                overflowed,
                out,
                start_offset,
            );

            if !*overflowed {
                let (style_name, level) = match block {
                    Block::Paragraph { style_name, .. } => (style_name.as_deref(), None),
                    Block::Heading {
                        style_name, level, ..
                    } => (style_name.as_deref(), Some(*level)),
                    _ => unreachable!(),
                };
                let props = resolve_paragraph_props(style_name, styles, level);
                let after = props.space_after.max(if level.is_some() {
                    0.0
                } else {
                    props.font_size * 0.4
                });
                if state.advance(after) {
                    *overflowed = true;
                }
            }
            emitted
        }
        Block::HorizontalRule => {
            if start_offset > 0 {
                return 0;
            }
            let y = state.pdf_y();
            write_horizontal_rule(state.left_margin, y, state.usable_width, out);
            if state.advance(12.0) && !*overflowed {
                *overflowed = true;
            }
            1
        }
        Block::PageBreak => {
            *overflowed = true;
            1
        }
        _ => 0,
    }
}
