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

//! Unit tests for colour.rs.

use super::Colour;

#[test]
fn from_hex_3_digit() {
    let c = Colour::from_hex("#abc").unwrap();
    assert_eq!(c, Colour::from_hex("aabbcc").unwrap());
}

#[test]
fn from_hex_6_digit_with_hash() {
    let c = Colour::from_hex("#ff0000").unwrap();
    if let Colour::Rgb { r, g, b, a } = c {
        assert!((r - 1.0).abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!(b.abs() < 0.01);
        assert!((a - 1.0).abs() < 0.01);
    } else {
        panic!("Expected Rgb");
    }
}

#[test]
fn from_hex_6_digit_without_hash() {
    let c = Colour::from_hex("00ff00").unwrap();
    if let Colour::Rgb { r, g, b, .. } = c {
        assert!(r.abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!(b.abs() < 0.01);
    } else {
        panic!("Expected Rgb");
    }
}

#[test]
fn from_hex_8_digit() {
    let c = Colour::from_hex("#0000ff80").unwrap();
    if let Colour::Rgb { r, g, b, a } = c {
        assert!(r.abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
        assert!((a - 0x80 as f32 / 255.0).abs() < 0.01);
    } else {
        panic!("Expected Rgb");
    }
}

#[test]
fn from_hex_invalid_inputs() {
    assert!(Colour::from_hex("").is_none());
    assert!(Colour::from_hex("#zzzzzz").is_none());
    assert!(Colour::from_hex("#1234").is_none());
    assert!(Colour::from_hex("12345").is_none());
    assert!(Colour::from_hex("#gg0000").is_none());
}

#[test]
fn from_u8_rgba_round_trip_to_hex() {
    let c = Colour::from_u8_rgba(0xde, 0xad, 0xbe, 0xff);
    let hex = c.to_hex().unwrap();
    assert_eq!(hex.to_lowercase(), "#deadbe");
}

#[test]
fn from_u8_rgba_with_alpha_round_trip() {
    let c = Colour::from_u8_rgba(0x11, 0x22, 0x33, 0x44);
    let hex = c.to_hex().unwrap();
    assert_eq!(hex.to_lowercase(), "#11223344");
}

#[test]
fn alpha_for_all_variants() {
    assert!((Colour::Rgb { r: 0.0, g: 0.0, b: 0.0, a: 0.5 }.alpha() - 0.5).abs() < 1e-6);
    assert!((Colour::Cmyk { c: 0.0, m: 0.0, y: 0.0, k: 0.0, alpha: 0.3 }.alpha() - 0.3).abs() < 1e-6);
    assert!((Colour::Lab { l: 50.0, a: 0.0, b: 0.0, alpha: 0.7 }.alpha() - 0.7).abs() < 1e-6);
    let spot = Colour::Spot {
        name: "PANTONE 186 C".to_string(),
        tint: 0.8,
        lab_ref: [41.0, 63.0, 31.0],
        cmyk_fallback: Box::new(Colour::black()),
    };
    assert!((spot.alpha() - 0.8).abs() < 1e-6);
    assert!((Colour::Linked { id: "x".to_string() }.alpha() - 1.0).abs() < 1e-6);
}

#[test]
fn with_alpha_rgb() {
    let c = Colour::Rgb { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }.with_alpha(0.5);
    assert!((c.alpha() - 0.5).abs() < 1e-6);
}

#[test]
fn with_alpha_cmyk() {
    let c = Colour::Cmyk { c: 0.1, m: 0.2, y: 0.3, k: 0.4, alpha: 1.0 }.with_alpha(0.0);
    assert!(c.alpha().abs() < 1e-6);
}

#[test]
fn is_transparent() {
    assert!(Colour::transparent().is_transparent());
    assert!(!Colour::black().is_transparent());
    assert!(!Colour::white().is_transparent());
}

#[test]
fn to_css_string_rgba_format() {
    let c = Colour::Rgb { r: 1.0, g: 0.0, b: 0.0, a: 0.5 };
    let s = c.to_css_string();
    assert!(s.starts_with("rgba("), "Expected rgba format, got: {}", s);
}

#[test]
fn to_css_string_hex_format_when_opaque() {
    let c = Colour::Rgb { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    let s = c.to_css_string();
    assert_eq!(s, "#ff0000");
}

#[test]
fn black_white_transparent_constructors() {
    assert_eq!(Colour::black(), Colour::Rgb { r: 0.0, g: 0.0, b: 0.0, a: 1.0 });
    assert_eq!(Colour::white(), Colour::Rgb { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });
    assert_eq!(Colour::transparent(), Colour::Rgb { r: 0.0, g: 0.0, b: 0.0, a: 0.0 });
}
