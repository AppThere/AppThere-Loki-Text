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

//! Loki colour extension helpers for ODT documents.
//!
//! The Loki namespace (`https://appthere.com/loki/ns`) is already declared in
//! ODT output via [`crate::writer::namespaces`]. This module provides
//! constants and helpers for serialising and deserialising [`Colour`] values
//! as JSON strings in `loki:colour` XML attributes on `style:text-properties`
//! elements.
//!
//! The round-trip contract is:
//! - **Writer**: when a style has a `loki:colour` key in its attributes map,
//!   the value is emitted as-is on `<style:text-properties loki:colour="…">`.
//! - **Parser**: `loki:colour` on `style:text-properties` is captured in the
//!   attributes map so it survives a write → read round-trip.
//!
//! Only colour values that cannot be represented as a standard `fo:color` hex
//! string (CMYK, Lab, Spot, Linked) need to be stored here. sRGB colours are
//! serialised only in `fo:color` for maximum interoperability.

use common_core::colour_management::Colour;

/// Loki namespace URI used in ODT documents.
pub const LOKI_NS: &str = "https://appthere.com/loki/ns";

/// The attribute key used to store a non-RGB colour in the style attributes map.
pub const LOKI_COLOUR_KEY: &str = "loki:colour";

/// Serialise a [`Colour`] to a compact JSON string for use as a `loki:colour`
/// XML attribute value.
///
/// Returns `None` if serialisation fails (should not happen in practice).
pub fn colour_to_attr(colour: &Colour) -> Option<String> {
    serde_json::to_string(colour).ok()
}

/// Deserialise a [`Colour`] from a `loki:colour` XML attribute JSON string.
///
/// Returns `None` if the string is not valid JSON for a [`Colour`].
pub fn colour_from_attr(s: &str) -> Option<Colour> {
    serde_json::from_str(s).ok()
}

/// Returns `true` if `colour` should be stored in a `loki:colour` attribute.
///
/// Only non-RGB colours need the Loki extension attribute. RGB colours are
/// fully representable by the standard `fo:color` hex attribute.
pub fn needs_loki_attr(colour: &Colour) -> bool {
    !matches!(colour, Colour::Rgb { .. })
}

/// Parse a CSS colour string into a [`Colour`].
///
/// Accepts:
/// - `#rgb`, `#rrggbb`, `#rrggbbaa` hex strings
/// - `rgb(r, g, b)` with integer 0–255 channel values
/// - A subset of CSS named colours: `black`, `white`, `red`, `green`, `blue`,
///   `yellow`, `cyan`, `magenta`, `transparent`
///
/// Returns `None` if the string is not recognised.
pub fn parse_colour_str(s: &str) -> Option<Colour> {
    let s = s.trim();
    if s.starts_with('#') {
        return Colour::from_hex(s);
    }
    if let Some(c) = parse_rgb_function(s) {
        return Some(c);
    }
    parse_css_named_colour(s)
}

/// Convert a [`Colour`] to an ODF-compatible `fo:color` hex string.
///
/// For RGB with full opacity this is an exact `#rrggbb` value. For other
/// colour spaces a device-dependent sRGB approximation is returned. Use a
/// `loki:colour` JSON attribute alongside for lossless round-trips of
/// non-RGB colours.
pub fn colour_to_odf_string(colour: &Colour) -> String {
    colour.to_css_string()
}

// ---------------------------------------------------------------------------
// Private parsing helpers
// ---------------------------------------------------------------------------

/// Parses a CSS `rgb(r, g, b)` function with integer 0–255 channel values.
fn parse_rgb_function(s: &str) -> Option<Colour> {
    let inner = s.strip_prefix("rgb(")?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 3 {
        return None;
    }
    let r: u8 = parts[0].trim().parse().ok()?;
    let g: u8 = parts[1].trim().parse().ok()?;
    let b: u8 = parts[2].trim().parse().ok()?;
    Some(Colour::from_u8_rgb(r, g, b))
}

/// Resolves a small subset of CSS named colours to [`Colour`] values.
fn parse_css_named_colour(s: &str) -> Option<Colour> {
    match s.to_lowercase().as_str() {
        "black" => Some(Colour::black()),
        "white" => Some(Colour::white()),
        "transparent" => Some(Colour::transparent()),
        "red" => Some(Colour::from_u8_rgb(255, 0, 0)),
        "green" => Some(Colour::from_u8_rgb(0, 128, 0)),
        "blue" => Some(Colour::from_u8_rgb(0, 0, 255)),
        "yellow" => Some(Colour::from_u8_rgb(255, 255, 0)),
        "cyan" => Some(Colour::from_u8_rgb(0, 255, 255)),
        "magenta" => Some(Colour::from_u8_rgb(255, 0, 255)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmyk_round_trips_via_attr() {
        let c = Colour::Cmyk {
            c: 0.1,
            m: 0.2,
            y: 0.3,
            k: 0.4,
            alpha: 1.0,
        };
        let json = colour_to_attr(&c).unwrap();
        let back = colour_from_attr(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn rgb_does_not_need_loki_attr() {
        let c = Colour::from_u8_rgb(255, 0, 0);
        assert!(!needs_loki_attr(&c));
    }

    #[test]
    fn cmyk_needs_loki_attr() {
        let c = Colour::Cmyk {
            c: 0.0,
            m: 1.0,
            y: 1.0,
            k: 0.0,
            alpha: 1.0,
        };
        assert!(needs_loki_attr(&c));
    }

    #[test]
    fn loki_ns_uri_matches_namespace_declaration() {
        assert_eq!(LOKI_NS, "https://appthere.com/loki/ns");
    }

    #[test]
    fn parse_colour_str_hex_6() {
        let c = parse_colour_str("#ff0000").unwrap();
        assert_eq!(c, Colour::from_u8_rgb(255, 0, 0));
    }

    #[test]
    fn parse_colour_str_hex_3() {
        let c = parse_colour_str("#f00").unwrap();
        assert_eq!(c, Colour::from_u8_rgb(255, 0, 0));
    }

    #[test]
    fn parse_colour_str_rgb_function() {
        let c = parse_colour_str("rgb(0, 128, 255)").unwrap();
        assert_eq!(c, Colour::from_u8_rgb(0, 128, 255));
    }

    #[test]
    fn parse_colour_str_named_black() {
        let c = parse_colour_str("black").unwrap();
        assert_eq!(c, Colour::black());
    }

    #[test]
    fn parse_colour_str_invalid_returns_none() {
        assert!(parse_colour_str("not-a-colour").is_none());
    }

    #[test]
    fn colour_to_odf_string_rgb_opaque() {
        let c = Colour::from_u8_rgb(255, 0, 0);
        assert_eq!(colour_to_odf_string(&c), "#ff0000");
    }

    #[test]
    fn colour_to_odf_string_black() {
        assert_eq!(colour_to_odf_string(&Colour::black()), "#000000");
    }
}
