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

//! Line-breaking and page layout for text document PDF generation.

use crate::fonts::subset::FontSubset;

/// State for the single-pass text layout engine.
pub struct LayoutState {
    /// Current Y position in PDF points (counting DOWN from top: 0 = top margin).
    pub current_y_from_top: f64,
    pub left_margin: f64,
    pub top_margin: f64,
    pub usable_width: f64,
    pub page_height: f64,
    pub bottom_margin: f64,
}

impl LayoutState {
    pub fn new(page_width: f64, page_height: f64, margin: f64) -> Self {
        LayoutState {
            current_y_from_top: margin,
            left_margin: margin,
            top_margin: margin,
            usable_width: page_width - 2.0 * margin,
            page_height,
            bottom_margin: margin,
        }
    }

    /// Returns the PDF Y coordinate (from bottom) for the current line.
    pub fn pdf_y(&self) -> f64 {
        self.page_height - self.current_y_from_top
    }

    /// Advance by `line_height` points. Returns true if the page overflowed.
    pub fn advance(&mut self, delta: f64) -> bool {
        self.current_y_from_top += delta;
        self.current_y_from_top > self.page_height - self.bottom_margin
    }

    /// Reset to the top margin (simulates a page break — single-page Phase 7).
    pub fn reset_page(&mut self) {
        self.current_y_from_top = self.top_margin;
    }
}

/// A word and its measured width in points.
pub struct Word {
    pub text: String,
    pub width: f64,
}

/// Break `text` into words, measuring each one.
pub fn break_words(text: &str, subset: &FontSubset, font_size: f64) -> Vec<Word> {
    use crate::writer::text::measure::measure_text;
    // Split on whitespace, preserving non-empty segments.
    text.split_whitespace()
        .map(|w| {
            let width = measure_text(w, &subset.bytes, font_size);
            Word { text: w.to_string(), width }
        })
        .collect()
}

/// Break a list of words into lines that fit within `max_width`.
///
/// Returns a `Vec<Vec<Word>>` where each inner `Vec` is one line.
pub fn wrap_words(words: Vec<Word>, space_width: f64, max_width: f64) -> Vec<Vec<Word>> {
    let mut lines: Vec<Vec<Word>> = Vec::new();
    let mut current_line: Vec<Word> = Vec::new();
    let mut current_width = 0.0f64;

    for word in words {
        let needed = if current_line.is_empty() {
            word.width
        } else {
            current_width + space_width + word.width
        };

        if needed <= max_width || current_line.is_empty() {
            // Fits on current line (or is the first word — never break before first word).
            if !current_line.is_empty() {
                current_width += space_width;
            }
            current_width += word.width;
            current_line.push(word);
        } else {
            // Start a new line.
            if !current_line.is_empty() {
                lines.push(std::mem::take(&mut current_line));
            }
            current_width = word.width;
            current_line.push(word);
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
