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

use serde::{Deserialize, Serialize};

/// An RGBA colour with 8-bit channels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    /// Alpha: 0 = fully transparent, 255 = fully opaque.
    pub a: u8,
}

impl Colour {
    /// Parse a hex colour string. Supports `#rgb`, `#rrggbb`, `#rrggbbaa`.
    pub fn from_hex(hex: &str) -> Option<Colour> {
        let hex = hex.trim().trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                Some(Colour { r, g, b, a: 255 })
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Colour { r, g, b, a: 255 })
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Colour { r, g, b, a })
            }
            _ => None,
        }
    }

    /// Return `#rrggbbaa` or `#rrggbb` if fully opaque.
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    /// Return an SVG-compatible colour string.
    /// Uses `rgba(r,g,b,a)` when not fully opaque, otherwise `#rrggbb`.
    pub fn to_svg_colour(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            let alpha = self.a as f64 / 255.0;
            format!("rgba({},{},{},{:.4})", self.r, self.g, self.b, alpha)
        }
    }

    pub fn transparent() -> Colour {
        Colour {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    pub fn black() -> Colour {
        Colour {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub fn white() -> Colour {
        Colour {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

/// Parse a CSS colour name or hex string into a Colour.
pub fn parse_css_colour(s: &str) -> Option<Colour> {
    let s = s.trim();
    if s.starts_with('#') {
        return Colour::from_hex(s);
    }
    // Basic CSS named colours
    match s.to_lowercase().as_str() {
        "black" => Some(Colour::black()),
        "white" => Some(Colour::white()),
        "red" => Some(Colour {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }),
        "green" => Some(Colour {
            r: 0,
            g: 128,
            b: 0,
            a: 255,
        }),
        "blue" => Some(Colour {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        }),
        "yellow" => Some(Colour {
            r: 255,
            g: 255,
            b: 0,
            a: 255,
        }),
        "cyan" => Some(Colour {
            r: 0,
            g: 255,
            b: 255,
            a: 255,
        }),
        "magenta" => Some(Colour {
            r: 255,
            g: 0,
            b: 255,
            a: 255,
        }),
        "orange" => Some(Colour {
            r: 255,
            g: 165,
            b: 0,
            a: 255,
        }),
        "purple" => Some(Colour {
            r: 128,
            g: 0,
            b: 128,
            a: 255,
        }),
        "pink" => Some(Colour {
            r: 255,
            g: 192,
            b: 203,
            a: 255,
        }),
        "brown" => Some(Colour {
            r: 165,
            g: 42,
            b: 42,
            a: 255,
        }),
        "gray" | "grey" => Some(Colour {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        }),
        "lime" => Some(Colour {
            r: 0,
            g: 255,
            b: 0,
            a: 255,
        }),
        "navy" => Some(Colour {
            r: 0,
            g: 0,
            b: 128,
            a: 255,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex_6digit() {
        let c = Colour::from_hex("#ff8800").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 136);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_from_hex_3digit() {
        let c = Colour::from_hex("#f80").unwrap();
        assert_eq!(c.r, 0xff);
        assert_eq!(c.g, 0x88);
        assert_eq!(c.b, 0x00);
    }

    #[test]
    fn test_from_hex_8digit() {
        let c = Colour::from_hex("#ff880080").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 136);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn test_roundtrip_hex_opaque() {
        let c = Colour {
            r: 100,
            g: 200,
            b: 50,
            a: 255,
        };
        let hex = c.to_hex();
        let back = Colour::from_hex(&hex).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn test_roundtrip_hex_transparent() {
        let c = Colour {
            r: 100,
            g: 200,
            b: 50,
            a: 128,
        };
        let hex = c.to_hex();
        let back = Colour::from_hex(&hex).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn test_to_svg_colour_opaque() {
        let c = Colour::black();
        assert_eq!(c.to_svg_colour(), "#000000");
    }
}
