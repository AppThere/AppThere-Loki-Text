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

//! SVG path → PDF path operator conversion.
//!
//! PDF uses a coordinate system with Y increasing upward (origin at bottom-left),
//! while SVG uses Y increasing downward (origin at top-left). All Y coordinates
//! are flipped: `pdf_y = page_height - svg_y`.

/// A PDF path operation (in PDF coordinate space, Y-up).
#[derive(Debug, Clone, PartialEq)]
pub enum PathOp {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64),
    ClosePath,
}

/// Convert a sequence of `PathOp` to a PDF content stream fragment.
pub fn ops_to_pdf_stream(ops: &[PathOp]) -> String {
    let mut buf = String::new();
    for op in ops {
        match op {
            PathOp::MoveTo(x, y) => {
                buf.push_str(&format!("{:.4} {:.4} m\n", x, y));
            }
            PathOp::LineTo(x, y) => {
                buf.push_str(&format!("{:.4} {:.4} l\n", x, y));
            }
            PathOp::CurveTo(x1, y1, x2, y2, x, y) => {
                buf.push_str(&format!(
                    "{:.4} {:.4} {:.4} {:.4} {:.4} {:.4} c\n",
                    x1, y1, x2, y2, x, y
                ));
            }
            PathOp::ClosePath => {
                buf.push_str("h\n");
            }
        }
    }
    buf
}

/// Parse an SVG path `d` attribute string into PDF `PathOp` operations.
///
/// The `page_height` parameter (in PDF user units / points) is used to flip
/// Y coordinates from SVG space (Y-down) to PDF space (Y-up).
///
/// Supports: M, m, L, l, H, h, V, v, C, c, Z/z.
pub fn parse_svg_path(d: &str, page_height: f64) -> Vec<PathOp> {
    let mut ops = Vec::new();
    let mut cur_x = 0.0f64;
    let mut cur_y = 0.0f64;

    let tokens = tokenise_svg_path(d);
    let mut iter = tokens.iter().peekable();

    while let Some(token) = iter.next() {
        match token.as_str() {
            "M" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    let x = consume_f64(&mut iter);
                    let y = consume_f64(&mut iter);
                    cur_x = x;
                    cur_y = y;
                    ops.push(PathOp::MoveTo(x, flip_y(y, page_height)));
                }
            }
            "m" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    let dx = consume_f64(&mut iter);
                    let dy = consume_f64(&mut iter);
                    cur_x += dx;
                    cur_y += dy;
                    ops.push(PathOp::MoveTo(cur_x, flip_y(cur_y, page_height)));
                }
            }
            "L" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    let x = consume_f64(&mut iter);
                    let y = consume_f64(&mut iter);
                    cur_x = x;
                    cur_y = y;
                    ops.push(PathOp::LineTo(x, flip_y(y, page_height)));
                }
            }
            "l" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    let dx = consume_f64(&mut iter);
                    let dy = consume_f64(&mut iter);
                    cur_x += dx;
                    cur_y += dy;
                    ops.push(PathOp::LineTo(cur_x, flip_y(cur_y, page_height)));
                }
            }
            "H" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    cur_x = consume_f64(&mut iter);
                    ops.push(PathOp::LineTo(cur_x, flip_y(cur_y, page_height)));
                }
            }
            "h" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    cur_x += consume_f64(&mut iter);
                    ops.push(PathOp::LineTo(cur_x, flip_y(cur_y, page_height)));
                }
            }
            "V" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    cur_y = consume_f64(&mut iter);
                    ops.push(PathOp::LineTo(cur_x, flip_y(cur_y, page_height)));
                }
            }
            "v" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    cur_y += consume_f64(&mut iter);
                    ops.push(PathOp::LineTo(cur_x, flip_y(cur_y, page_height)));
                }
            }
            "C" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    let x1 = consume_f64(&mut iter);
                    let y1 = consume_f64(&mut iter);
                    let x2 = consume_f64(&mut iter);
                    let y2 = consume_f64(&mut iter);
                    let x = consume_f64(&mut iter);
                    let y = consume_f64(&mut iter);
                    cur_x = x;
                    cur_y = y;
                    ops.push(PathOp::CurveTo(
                        x1,
                        flip_y(y1, page_height),
                        x2,
                        flip_y(y2, page_height),
                        x,
                        flip_y(y, page_height),
                    ));
                }
            }
            "c" => {
                while iter.peek().and_then(|t| t.parse::<f64>().ok()).is_some() {
                    let dx1 = consume_f64(&mut iter);
                    let dy1 = consume_f64(&mut iter);
                    let dx2 = consume_f64(&mut iter);
                    let dy2 = consume_f64(&mut iter);
                    let dx = consume_f64(&mut iter);
                    let dy = consume_f64(&mut iter);
                    ops.push(PathOp::CurveTo(
                        cur_x + dx1,
                        flip_y(cur_y + dy1, page_height),
                        cur_x + dx2,
                        flip_y(cur_y + dy2, page_height),
                        cur_x + dx,
                        flip_y(cur_y + dy, page_height),
                    ));
                    cur_x += dx;
                    cur_y += dy;
                }
            }
            "Z" | "z" => {
                ops.push(PathOp::ClosePath);
            }
            _ => {} // skip unknown / numbers that appear without a command context
        }
    }
    ops
}

fn flip_y(svg_y: f64, page_height: f64) -> f64 {
    page_height - svg_y
}

fn consume_f64<'a, I: Iterator<Item = &'a String>>(iter: &mut std::iter::Peekable<I>) -> f64 {
    iter.next().and_then(|t| t.parse().ok()).unwrap_or(0.0)
}

/// Tokenise an SVG path `d` string into command letters and number strings.
fn tokenise_svg_path(d: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut num_buf = String::new();

    for ch in d.chars() {
        if ch.is_ascii_alphabetic() {
            let trimmed = num_buf.trim().to_string();
            if !trimmed.is_empty() {
                for part in trimmed.split(|c: char| c == ',' || c.is_whitespace()) {
                    let p = part.trim();
                    if !p.is_empty() {
                        tokens.push(p.to_string());
                    }
                }
                num_buf.clear();
            }
            tokens.push(ch.to_string());
        } else {
            num_buf.push(ch);
        }
    }
    // Flush remaining numbers.
    let trimmed = num_buf.trim().to_string();
    if !trimmed.is_empty() {
        for part in trimmed.split(|c: char| c == ',' || c.is_whitespace()) {
            let p = part.trim();
            if !p.is_empty() {
                tokens.push(p.to_string());
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_rect_path() {
        // M 10 20 L 110 20 L 110 120 L 10 120 Z
        let d = "M 10 20 L 110 20 L 110 120 L 10 120 Z";
        let page_h = 200.0;
        let ops = parse_svg_path(d, page_h);
        assert_eq!(ops[0], PathOp::MoveTo(10.0, 180.0)); // 200 - 20
        assert_eq!(ops[1], PathOp::LineTo(110.0, 180.0));
        assert_eq!(ops[2], PathOp::LineTo(110.0, 80.0)); // 200 - 120
        assert_eq!(ops[4], PathOp::ClosePath);
    }

    #[test]
    fn y_flip_is_applied() {
        let d = "M 0 100 L 100 200";
        let ops = parse_svg_path(d, 300.0);
        assert_eq!(ops[0], PathOp::MoveTo(0.0, 200.0)); // 300 - 100
        assert_eq!(ops[1], PathOp::LineTo(100.0, 100.0)); // 300 - 200
    }

    #[test]
    fn ops_to_stream_format() {
        let ops = vec![
            PathOp::MoveTo(10.0, 20.0),
            PathOp::LineTo(100.0, 20.0),
            PathOp::ClosePath,
        ];
        let s = ops_to_pdf_stream(&ops);
        assert!(s.contains("10.0000 20.0000 m"));
        assert!(s.contains("100.0000 20.0000 l"));
        assert!(s.contains("h"));
    }
}
