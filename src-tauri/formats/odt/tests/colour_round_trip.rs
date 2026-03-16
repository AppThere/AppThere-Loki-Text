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

//! Round-trip tests for typed colour fields on [`StyleDefinition`].
//!
//! Each test verifies that `font_colour` and `background_colour` survive a
//! write → parse cycle through ODT styles XML.

use common_core::colour_management::Colour;
use odt_format::parser::parse_document;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
const NS_LOKI: &str = "https://appthere.com/loki/ns";

fn fodt(styles_xml: &str) -> String {
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}"
    xmlns:loki="{NS_LOKI}"
    office:version="1.3">
  <office:styles>{styles_xml}</office:styles>
  <office:body><office:text><text:p>x</text:p></office:text></office:body>
</office:document>"##
    )
}

/// Parse a FODT document and return the named style, or panic.
fn get_style(xml: &str, name: &str) -> common_core::StyleDefinition {
    let doc = parse_document(xml).expect("parse failed");
    doc.styles.get(name).cloned().expect("style not found")
}

// ── Test 1: RGB font colour round-trips via fo:color ─────────────────────────

#[test]
fn rgb_font_colour_round_trips() {
    let xml = fodt(
        r##"<style:style style:name="Red" style:family="text">
               <style:text-properties fo:color="#ff0000"/>
             </style:style>"##,
    );
    let style = get_style(&xml, "Red");
    let fc = style.font_colour.expect("font_colour should be Some");
    assert_eq!(fc, Colour::from_u8_rgb(255, 0, 0));
}

// ── Test 2: CMYK colour emits loki:colour and parses back ─────────────────────

#[test]
fn cmyk_font_colour_round_trips_via_loki_attr() {
    let cmyk = Colour::Cmyk { c: 0.0, m: 1.0, y: 1.0, k: 0.0, alpha: 1.0 };
    let loki_json = xml_escape(&serde_json::to_string(&cmyk).unwrap());
    let styles = format!(
        r##"<style:style style:name="CmykRed" style:family="text">
               <style:text-properties fo:color="#ff0000" loki:colour="{loki_json}"/>
             </style:style>"##
    );
    let xml = fodt(&styles);
    let style = get_style(&xml, "CmykRed");
    let fc = style.font_colour.expect("font_colour should be Some");
    assert_eq!(fc, cmyk);
}

// ── Test 3: Lab colour emits loki:colour and parses back ──────────────────────

#[test]
fn lab_font_colour_round_trips_via_loki_attr() {
    let lab = Colour::Lab { l: 53.0, a: 80.0, b: 67.0, alpha: 1.0 };
    let loki_json = xml_escape(&serde_json::to_string(&lab).unwrap());
    let styles = format!(
        r##"<style:style style:name="LabRed" style:family="text">
               <style:text-properties fo:color="#7f5340" loki:colour="{loki_json}"/>
             </style:style>"##
    );
    let xml = fodt(&styles);
    let style = get_style(&xml, "LabRed");
    let fc = style.font_colour.expect("font_colour should be Some");
    assert_eq!(fc, lab);
}

// ── Test 4: No colour attribute means font_colour is None ─────────────────────

#[test]
fn no_colour_attribute_gives_none_font_colour() {
    let xml = fodt(
        r##"<style:style style:name="Plain" style:family="paragraph">
               <style:text-properties fo:font-weight="bold"/>
             </style:style>"##,
    );
    let style = get_style(&xml, "Plain");
    assert!(style.font_colour.is_none());
}

// ── Test 5: background_colour round-trips via fo:background-color ─────────────

#[test]
fn background_colour_round_trips() {
    let xml = fodt(
        r##"<style:style style:name="Highlighted" style:family="text">
               <style:text-properties fo:background-color="#ffff00"/>
             </style:style>"##,
    );
    let style = get_style(&xml, "Highlighted");
    let bg = style.background_colour.expect("background_colour should be Some");
    assert_eq!(bg, Colour::from_u8_rgb(255, 255, 0));
}

// ── Test 6: Both font and background colour round-trip together ───────────────

#[test]
fn both_colours_round_trip_together() {
    let xml = fodt(
        r##"<style:style style:name="Both" style:family="text">
               <style:text-properties fo:color="#0000ff" fo:background-color="#ffff00"/>
             </style:style>"##,
    );
    let style = get_style(&xml, "Both");
    assert_eq!(
        style.font_colour.expect("font_colour"),
        Colour::from_u8_rgb(0, 0, 255)
    );
    assert_eq!(
        style.background_colour.expect("background_colour"),
        Colour::from_u8_rgb(255, 255, 0)
    );
}

// ── Test 7: loki:colour takes precedence over fo:color ───────────────────────

#[test]
fn loki_colour_takes_precedence_over_fo_color() {
    // fo:color is a hex approximation; loki:colour has the exact CMYK value.
    let cmyk = Colour::Cmyk { c: 1.0, m: 0.0, y: 0.0, k: 0.0, alpha: 1.0 };
    let loki_json = xml_escape(&serde_json::to_string(&cmyk).unwrap());
    let styles = format!(
        r##"<style:style style:name="Cyan" style:family="text">
               <style:text-properties fo:color="#00ffff" loki:colour="{loki_json}"/>
             </style:style>"##
    );
    let xml = fodt(&styles);
    let style = get_style(&xml, "Cyan");
    let fc = style.font_colour.expect("font_colour");
    // Should be the CMYK value, not the RGB fallback.
    assert!(matches!(fc, Colour::Cmyk { .. }), "expected CMYK, got {:?}", fc);
}

// ── Test 8: sRGB stored only in fo:color parses to typed Rgb colour ───────────

#[test]
fn srgb_in_fo_color_only_parses_to_rgb_colour() {
    let xml = fodt(
        r##"<style:style style:name="Blue" style:family="paragraph">
               <style:text-properties fo:color="#0000ff"/>
             </style:style>"##,
    );
    let style = get_style(&xml, "Blue");
    let fc = style.font_colour.expect("font_colour");
    assert!(matches!(fc, Colour::Rgb { .. }), "expected Rgb, got {:?}", fc);
    assert_eq!(fc, Colour::from_u8_rgb(0, 0, 255));
}
