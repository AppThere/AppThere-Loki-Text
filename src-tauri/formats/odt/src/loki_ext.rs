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
}
