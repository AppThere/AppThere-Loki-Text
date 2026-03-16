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

static ATKINSON_REGULAR: &[u8] =
    include_bytes!("../../src/assets/fonts/AtkinsonHyperlegibleNext-Regular.ttf");
static ATKINSON_BOLD: &[u8] =
    include_bytes!("../../src/assets/fonts/AtkinsonHyperlegibleNext-Bold.ttf");
static ATKINSON_ITALIC: &[u8] =
    include_bytes!("../../src/assets/fonts/AtkinsonHyperlegibleNext-Italic.ttf");
static ATKINSON_BOLD_ITALIC: &[u8] =
    include_bytes!("../../src/assets/fonts/AtkinsonHyperlegibleNext-BoldItalic.ttf");

static BITTER_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/Bitter-Variable.ttf");
static BITTER_ITALIC: &[u8] = include_bytes!("../../src/assets/fonts/Bitter-Italic-Variable.ttf");

static BODONI_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/BodoniModa-Variable.ttf");
static BODONI_ITALIC: &[u8] =
    include_bytes!("../../src/assets/fonts/BodoniModa-Italic-Variable.ttf");

static CAVEAT_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/Caveat-Variable.ttf");

static CORMORANT_VARIABLE: &[u8] =
    include_bytes!("../../src/assets/fonts/CormorantGaramond-Variable.ttf");
static CORMORANT_ITALIC: &[u8] =
    include_bytes!("../../src/assets/fonts/CormorantGaramond-Italic-Variable.ttf");

static COURIER_REGULAR: &[u8] = include_bytes!("../../src/assets/fonts/CourierPrime-Regular.ttf");
static COURIER_BOLD: &[u8] = include_bytes!("../../src/assets/fonts/CourierPrime-Bold.ttf");
static COURIER_ITALIC: &[u8] = include_bytes!("../../src/assets/fonts/CourierPrime-Italic.ttf");
static COURIER_BOLD_ITALIC: &[u8] =
    include_bytes!("../../src/assets/fonts/CourierPrime-BoldItalic.ttf");

static GEIST_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/Geist-Variable.ttf");

static LEXEND_VARIABLE: &[u8] = include_bytes!("../../src/assets/fonts/Lexend-Variable.ttf");

static NEWSREADER_VARIABLE: &[u8] =
    include_bytes!("../../src/assets/fonts/Newsreader-Variable.ttf");
static NEWSREADER_ITALIC: &[u8] =
    include_bytes!("../../src/assets/fonts/Newsreader-Italic-Variable.ttf");

static PUBLIC_SANS_VARIABLE: &[u8] =
    include_bytes!("../../src/assets/fonts/PublicSans-Variable.ttf");
static PUBLIC_SANS_ITALIC: &[u8] =
    include_bytes!("../../src/assets/fonts/PublicSans-Italic-Variable.ttf");

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
    r.add_font(
        "atkinson hyperlegible next",
        400,
        false,
        ATKINSON_REGULAR.to_vec(),
    );
    r.add_font(
        "atkinson hyperlegible next",
        700,
        false,
        ATKINSON_BOLD.to_vec(),
    );
    r.add_font(
        "atkinson hyperlegible next",
        400,
        true,
        ATKINSON_ITALIC.to_vec(),
    );
    r.add_font(
        "atkinson hyperlegible next",
        700,
        true,
        ATKINSON_BOLD_ITALIC.to_vec(),
    );
    // Short alias
    r.add_font("atkinson", 400, false, ATKINSON_REGULAR.to_vec());
    r.add_font("atkinson", 700, false, ATKINSON_BOLD.to_vec());
    r.add_font("atkinson", 400, true, ATKINSON_ITALIC.to_vec());
    r.add_font("atkinson", 700, true, ATKINSON_BOLD_ITALIC.to_vec());

    // Bitter (variable — same bytes for all weights)
    r.add_font("bitter", 400, false, BITTER_VARIABLE.to_vec());
    r.add_font("bitter", 700, false, BITTER_VARIABLE.to_vec());
    r.add_font("bitter", 400, true, BITTER_ITALIC.to_vec());
    r.add_font("bitter", 700, true, BITTER_ITALIC.to_vec());

    // Bodoni Moda (variable)
    r.add_font("bodoni moda", 400, false, BODONI_VARIABLE.to_vec());
    r.add_font("bodoni moda", 700, false, BODONI_VARIABLE.to_vec());
    r.add_font("bodoni moda", 400, true, BODONI_ITALIC.to_vec());
    r.add_font("bodoni moda", 700, true, BODONI_ITALIC.to_vec());

    // Caveat (variable, handwriting — no italic variant needed)
    r.add_font("caveat", 400, false, CAVEAT_VARIABLE.to_vec());
    r.add_font("caveat", 700, false, CAVEAT_VARIABLE.to_vec());
    r.add_font("caveat", 400, true, CAVEAT_VARIABLE.to_vec());
    r.add_font("caveat", 700, true, CAVEAT_VARIABLE.to_vec());

    // Cormorant Garamond (variable)
    r.add_font(
        "cormorant garamond",
        400,
        false,
        CORMORANT_VARIABLE.to_vec(),
    );
    r.add_font(
        "cormorant garamond",
        700,
        false,
        CORMORANT_VARIABLE.to_vec(),
    );
    r.add_font("cormorant garamond", 400, true, CORMORANT_ITALIC.to_vec());
    r.add_font("cormorant garamond", 700, true, CORMORANT_ITALIC.to_vec());
    r.add_font("cormorant", 400, false, CORMORANT_VARIABLE.to_vec());
    r.add_font("cormorant", 700, false, CORMORANT_VARIABLE.to_vec());
    r.add_font("cormorant", 400, true, CORMORANT_ITALIC.to_vec());
    r.add_font("cormorant", 700, true, CORMORANT_ITALIC.to_vec());

    // Courier Prime (static — proper bold/italic support)
    r.add_font("courier prime", 400, false, COURIER_REGULAR.to_vec());
    r.add_font("courier prime", 700, false, COURIER_BOLD.to_vec());
    r.add_font("courier prime", 400, true, COURIER_ITALIC.to_vec());
    r.add_font("courier prime", 700, true, COURIER_BOLD_ITALIC.to_vec());
    r.add_font("courier", 400, false, COURIER_REGULAR.to_vec());
    r.add_font("courier", 700, false, COURIER_BOLD.to_vec());
    r.add_font("courier", 400, true, COURIER_ITALIC.to_vec());
    r.add_font("courier", 700, true, COURIER_BOLD_ITALIC.to_vec());

    // Geist (variable)
    r.add_font("geist", 400, false, GEIST_VARIABLE.to_vec());
    r.add_font("geist", 700, false, GEIST_VARIABLE.to_vec());
    r.add_font("geist", 400, true, GEIST_VARIABLE.to_vec());
    r.add_font("geist", 700, true, GEIST_VARIABLE.to_vec());

    // Lexend (variable)
    r.add_font("lexend", 400, false, LEXEND_VARIABLE.to_vec());
    r.add_font("lexend", 700, false, LEXEND_VARIABLE.to_vec());
    r.add_font("lexend", 400, true, LEXEND_VARIABLE.to_vec());
    r.add_font("lexend", 700, true, LEXEND_VARIABLE.to_vec());

    // Newsreader (variable)
    r.add_font("newsreader", 400, false, NEWSREADER_VARIABLE.to_vec());
    r.add_font("newsreader", 700, false, NEWSREADER_VARIABLE.to_vec());
    r.add_font("newsreader", 400, true, NEWSREADER_ITALIC.to_vec());
    r.add_font("newsreader", 700, true, NEWSREADER_ITALIC.to_vec());

    // Public Sans (variable) — also the fallback
    r.add_font("public sans", 400, false, PUBLIC_SANS_VARIABLE.to_vec());
    r.add_font("public sans", 700, false, PUBLIC_SANS_VARIABLE.to_vec());
    r.add_font("public sans", 400, true, PUBLIC_SANS_ITALIC.to_vec());
    r.add_font("public sans", 700, true, PUBLIC_SANS_ITALIC.to_vec());

    // Roboto Flex (variable)
    r.add_font("roboto flex", 400, false, ROBOTO_FLEX.to_vec());
    r.add_font("roboto flex", 700, false, ROBOTO_FLEX.to_vec());
    r.add_font("roboto flex", 400, true, ROBOTO_FLEX.to_vec());
    r.add_font("roboto flex", 700, true, ROBOTO_FLEX.to_vec());
    r.add_font("roboto", 400, false, ROBOTO_FLEX.to_vec());
    r.add_font("roboto", 700, false, ROBOTO_FLEX.to_vec());
    r.add_font("roboto", 400, true, ROBOTO_FLEX.to_vec());
    r.add_font("roboto", 700, true, ROBOTO_FLEX.to_vec());

    r
}
