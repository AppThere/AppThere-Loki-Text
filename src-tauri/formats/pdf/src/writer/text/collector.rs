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

use crate::fonts::subset::UsedGlyphs;
use common_core::block::Block;
use common_core::inline::Inline;
use common_core::marks::TiptapMark;
use common_core::style::StyleDefinition;
use std::collections::HashMap;

/// (family_id, weight_100_900, italic)
pub type FontKey = (String, u16, bool);

/// Default font family used when a style does not specify one.
const DEFAULT_FONT_FAMILY: &str = "public sans";

/// Extract the font key for an inline based on block style + marks.
pub fn inline_font_key(
    marks: &[TiptapMark],
    style_name: Option<&str>,
    styles: &HashMap<String, StyleDefinition>,
    block_style_name: Option<&str>,
) -> FontKey {
    let is_bold = marks.iter().any(|m| matches!(m, TiptapMark::Bold));
    let is_italic = marks.iter().any(|m| matches!(m, TiptapMark::Italic));
    let weight = if is_bold { 700 } else { 400 };

    // Determine font family: inline style → block style → default.
    let family = style_name
        .and_then(|n| styles.get(n))
        .and_then(|s| {
            s.attributes
                .get("fo:font-family")
                .or_else(|| s.attributes.get("style:font-name"))
        })
        .or_else(|| {
            block_style_name.and_then(|n| styles.get(n)).and_then(|s| {
                s.attributes
                    .get("fo:font-family")
                    .or_else(|| s.attributes.get("style:font-name"))
            })
        })
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_FONT_FAMILY);

    (family.to_lowercase(), weight, is_italic)
}

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
        Block::Paragraph {
            style_name,
            content,
            ..
        }
        | Block::Heading {
            style_name,
            content,
            ..
        } => {
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
    if let Inline::Text {
        text,
        style_name,
        marks,
    } = inline
    {
        let key = inline_font_key(marks, style_name.as_deref(), styles, block_style);
        out.entry(key).or_default().extend(text.chars());
    }
}
