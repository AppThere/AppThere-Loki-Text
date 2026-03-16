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

//! Compile-time font registry for loki-pdf text export.
//!
//! All font files from `src/assets/fonts/` are embedded into the binary via
//! `include_bytes!`. The [`build_font_resolver`] function constructs a
//! [`MapFontResolver`] that covers the bundled font families.
//!
//! # Note
//! The font paths below are relative to this file's location in `src-tauri/src/`.
//! The assets live in `../src/assets/fonts/` relative to the `src-tauri/` directory.

use loki_pdf::MapFontResolver;

// ---------------------------------------------------------------------------
// Embedded font data — all 22 bundled fonts.
// ---------------------------------------------------------------------------

static ATKINSON_REGULAR: &[u8] = include_bytes!("../../src/assets/fonts/AtkinsonHyperlegibleNext-Regular.ttf");
static ATKINSON_BOLD:    &[u8] = include_bytes!("../../src/assets/fonts/AtkinsonHyperlegibleNext-Bold.ttf");
static ATKINSON_ITALIC:  &[u8] = include_bytes!("../../src/assets/fonts/AtkinsonHyperlegibleNext-Italic.ttf");
static ATKINSON_BOLD_ITALIC: &[u8] = include_bytes!("../../src/assets/fonts/AtkinsonHyperlegibleNext-BoldItalic.ttf");

static BITTER_VARIABLE:  &[u8] = include_bytes!("../../src/assets/fonts/Bitter-Variable.ttf");
static BITTER_ITALIC:    &[u8] = include_bytes!("../../src/assets/fonts/Bitter-Italic-Variable.ttf");

static BODONI_VARIABLE:  &[u8] = include_bytes!("../../src/assets/fonts/BodoniModa-Variable.ttf");
static BODONI_ITALIC:    &[u8] = include_bytes!("../../src/assets/fonts/BodoniModa-Italic-Variable.ttf");

static CAVEAT_VARIABLE:  &[u8] = include_bytes!("../../src/assets/fonts/Caveat-Variable.ttf");

static CORMORANT_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/CormorantGaramond-Variable.ttf");
static CORMORANT_ITALIC:   &[u8] = include_bytes!("../../src/assets/fonts/CormorantGaramond-Italic-Variable.ttf");

static COURIER_REGULAR: &[u8] = include_bytes!("../../src/assets/fonts/CourierPrime-Regular.ttf");
static COURIER_BOLD:    &[u8] = include_bytes!("../../src/assets/fonts/CourierPrime-Bold.ttf");
static COURIER_ITALIC:  &[u8] = include_bytes!("../../src/assets/fonts/CourierPrime-Italic.ttf");
static COURIER_BOLD_ITALIC: &[u8] = include_bytes!("../../src/assets/fonts/CourierPrime-BoldItalic.ttf");

static GEIST_VARIABLE:  &[u8] = include_bytes!("../../src/assets/fonts/Geist-Variable.ttf");

static LEXEND_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/Lexend-Variable.ttf");

static NEWSREADER_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/Newsreader-Variable.ttf");
static NEWSREADER_ITALIC:   &[u8] = include_bytes!("../../src/assets/fonts/Newsreader-Italic-Variable.ttf");

static PUBLIC_SANS_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/PublicSans-Variable.ttf");
static PUBLIC_SANS_ITALIC:   &[u8] = include_bytes!("../../src/assets/fonts/PublicSans-Italic-Variable.ttf");

static ROBOTO_FLEX: &[u8] = include_bytes!("../../src/assets/fonts/RobotoFlex-Variable.ttf");

// ---------------------------------------------------------------------------
// Resolver factory
// ---------------------------------------------------------------------------

/// Build a [`MapFontResolver`] populated with all bundled font families.
///
/// The fallback font is **Public Sans Regular**.
///
/// For variable fonts, the same bytes are registered for both bold and italic
/// variants — visual weight/slant differences require OpenType variation-axis
/// support (Phase 8). Static variants (Courier Prime) are registered properly.
pub fn build_font_resolver() -> MapFontResolver {
    let mut r = MapFontResolver::new("public sans");

    // Atkinson Hyperlegible Next (static)
    r.add_font("atkinson hyperlegible next", false, false, ATKINSON_REGULAR.to_vec());
    r.add_font("atkinson hyperlegible next", true,  false, ATKINSON_BOLD.to_vec());
    r.add_font("atkinson hyperlegible next", false, true,  ATKINSON_ITALIC.to_vec());
    r.add_font("atkinson hyperlegible next", true,  true,  ATKINSON_BOLD_ITALIC.to_vec());
    // Short alias
    r.add_font("atkinson", false, false, ATKINSON_REGULAR.to_vec());
    r.add_font("atkinson", true,  false, ATKINSON_BOLD.to_vec());
    r.add_font("atkinson", false, true,  ATKINSON_ITALIC.to_vec());
    r.add_font("atkinson", true,  true,  ATKINSON_BOLD_ITALIC.to_vec());

    // Bitter (variable — same bytes for all weights)
    r.add_font("bitter", false, false, BITTER_VARIABLE.to_vec());
    r.add_font("bitter", true,  false, BITTER_VARIABLE.to_vec());
    r.add_font("bitter", false, true,  BITTER_ITALIC.to_vec());
    r.add_font("bitter", true,  true,  BITTER_ITALIC.to_vec());

    // Bodoni Moda (variable)
    r.add_font("bodoni moda", false, false, BODONI_VARIABLE.to_vec());
    r.add_font("bodoni moda", true,  false, BODONI_VARIABLE.to_vec());
    r.add_font("bodoni moda", false, true,  BODONI_ITALIC.to_vec());
    r.add_font("bodoni moda", true,  true,  BODONI_ITALIC.to_vec());

    // Caveat (variable, handwriting — no italic variant needed)
    r.add_font("caveat", false, false, CAVEAT_VARIABLE.to_vec());
    r.add_font("caveat", true,  false, CAVEAT_VARIABLE.to_vec());
    r.add_font("caveat", false, true,  CAVEAT_VARIABLE.to_vec());
    r.add_font("caveat", true,  true,  CAVEAT_VARIABLE.to_vec());

    // Cormorant Garamond (variable)
    r.add_font("cormorant garamond", false, false, CORMORANT_VARIABLE.to_vec());
    r.add_font("cormorant garamond", true,  false, CORMORANT_VARIABLE.to_vec());
    r.add_font("cormorant garamond", false, true,  CORMORANT_ITALIC.to_vec());
    r.add_font("cormorant garamond", true,  true,  CORMORANT_ITALIC.to_vec());
    r.add_font("cormorant", false, false, CORMORANT_VARIABLE.to_vec());
    r.add_font("cormorant", true,  false, CORMORANT_VARIABLE.to_vec());
    r.add_font("cormorant", false, true,  CORMORANT_ITALIC.to_vec());
    r.add_font("cormorant", true,  true,  CORMORANT_ITALIC.to_vec());

    // Courier Prime (static — proper bold/italic support)
    r.add_font("courier prime", false, false, COURIER_REGULAR.to_vec());
    r.add_font("courier prime", true,  false, COURIER_BOLD.to_vec());
    r.add_font("courier prime", false, true,  COURIER_ITALIC.to_vec());
    r.add_font("courier prime", true,  true,  COURIER_BOLD_ITALIC.to_vec());
    r.add_font("courier", false, false, COURIER_REGULAR.to_vec());
    r.add_font("courier", true,  false, COURIER_BOLD.to_vec());
    r.add_font("courier", false, true,  COURIER_ITALIC.to_vec());
    r.add_font("courier", true,  true,  COURIER_BOLD_ITALIC.to_vec());

    // Geist (variable)
    r.add_font("geist", false, false, GEIST_VARIABLE.to_vec());
    r.add_font("geist", true,  false, GEIST_VARIABLE.to_vec());
    r.add_font("geist", false, true,  GEIST_VARIABLE.to_vec());
    r.add_font("geist", true,  true,  GEIST_VARIABLE.to_vec());

    // Lexend (variable)
    r.add_font("lexend", false, false, LEXEND_VARIABLE.to_vec());
    r.add_font("lexend", true,  false, LEXEND_VARIABLE.to_vec());
    r.add_font("lexend", false, true,  LEXEND_VARIABLE.to_vec());
    r.add_font("lexend", true,  true,  LEXEND_VARIABLE.to_vec());

    // Newsreader (variable)
    r.add_font("newsreader", false, false, NEWSREADER_VARIABLE.to_vec());
    r.add_font("newsreader", true,  false, NEWSREADER_VARIABLE.to_vec());
    r.add_font("newsreader", false, true,  NEWSREADER_ITALIC.to_vec());
    r.add_font("newsreader", true,  true,  NEWSREADER_ITALIC.to_vec());

    // Public Sans (variable) — also the fallback
    r.add_font("public sans", false, false, PUBLIC_SANS_VARIABLE.to_vec());
    r.add_font("public sans", true,  false, PUBLIC_SANS_VARIABLE.to_vec());
    r.add_font("public sans", false, true,  PUBLIC_SANS_ITALIC.to_vec());
    r.add_font("public sans", true,  true,  PUBLIC_SANS_ITALIC.to_vec());

    // Roboto Flex (variable)
    r.add_font("roboto flex", false, false, ROBOTO_FLEX.to_vec());
    r.add_font("roboto flex", true,  false, ROBOTO_FLEX.to_vec());
    r.add_font("roboto flex", false, true,  ROBOTO_FLEX.to_vec());
    r.add_font("roboto flex", true,  true,  ROBOTO_FLEX.to_vec());
    r.add_font("roboto", false, false, ROBOTO_FLEX.to_vec());
    r.add_font("roboto", true,  false, ROBOTO_FLEX.to_vec());
    r.add_font("roboto", false, true,  ROBOTO_FLEX.to_vec());
    r.add_font("roboto", true,  true,  ROBOTO_FLEX.to_vec());

    r
}
