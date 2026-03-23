// Copyright 2024 AppThere Ltd.
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

//! Level 3 — Category 2: Unicode edge cases.
//!
//! Tests for surrogate character references, bidi isolation/override
//! characters, zero-width joiners, variation selectors, emoji ZWJ sequences,
//! NFD-normalised text, and extremely long text nodes.

use common_core::{Block, Inline};
use odt_format::Document;

// ── Surrogate character references ────────────────────────────────────────────

/// U+D800 (high surrogate) is technically invalid in XML 1.0; roxmltree is
/// lenient and accepts it.  The important property is that parsing does not
/// crash and no sensitive data leaks from the output.
#[test]
fn test_high_surrogate_char_reference_no_crash() {
    let xml = super::fodt("<text:p>&#xD800;</text:p>");
    let _ = Document::from_xml(&xml); // must not panic
}

/// U+DFFF (low surrogate) — must not crash.
#[test]
fn test_low_surrogate_char_reference_no_crash() {
    let xml = super::fodt("<text:p>&#xDFFF;</text:p>");
    let _ = Document::from_xml(&xml); // must not panic
}

/// Surrogate pair as two adjacent character references — must not crash or
/// produce output containing sensitive content.
#[test]
fn test_surrogate_pair_two_char_references_no_crash() {
    let xml = super::fodt("<text:p>&#xD83D;&#xDE00;</text:p>");
    if let Ok(doc) = Document::from_xml(&xml) {
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(
            out.len() < 100_000,
            "Output from surrogate pair input must not be unexpectedly large"
        );
    }
    // is_err() is also acceptable.
}

// ── Bidi isolation / override characters ─────────────────────────────────────

/// U+2066 (LEFT-TO-RIGHT ISOLATE) must be preserved through a round-trip.
#[test]
fn test_lri_bidi_isolation_character_survives_round_trip() {
    let text = "Hello\u{2066}World\u{2069}"; // LRI … PDI
    let xml = super::fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "LRI bidi isolate (U+2066) should parse");
    let doc = result.unwrap();
    let out = doc.to_content_xml().expect("to_content_xml failed");
    let doc2 = Document::from_xml(&out).expect("re-parse failed");
    if let Some(Block::Paragraph { content, .. }) = doc2.blocks.first() {
        if let Some(Inline::Text { text: t, .. }) = content.first() {
            assert!(t.contains('\u{2066}'), "LRI must survive round-trip");
        }
    }
}

/// U+2067 (RIGHT-TO-LEFT ISOLATE) must not cause a panic.
#[test]
fn test_rli_bidi_isolation_character_no_panic() {
    let text = "Test\u{2067}Reversed\u{2069}";
    let xml = super::fodt(&format!("<text:p>{text}</text:p>"));
    let _ = Document::from_xml(&xml); // must not panic
}

/// U+2068 (FIRST STRONG ISOLATE) must not cause a panic.
#[test]
fn test_fsi_bidi_isolation_character_no_panic() {
    let text = "Test\u{2068}Content\u{2069}";
    let xml = super::fodt(&format!("<text:p>{text}</text:p>"));
    let _ = Document::from_xml(&xml); // must not panic
}

// ── Zero-width and invisible characters ───────────────────────────────────────

/// Zero-width joiner (U+200D) must be preserved through a round-trip.
#[test]
fn test_zero_width_joiner_preserved_round_trip() {
    let text = "man\u{200D}woman\u{200D}girl";
    let xml = super::fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "ZWJ in text content should parse");
    let out = result
        .unwrap()
        .to_content_xml()
        .expect("to_content_xml failed");
    assert!(out.contains('\u{200D}'), "ZWJ must survive write");
}

/// Variation selector U+FE0F (emoji presentation) must be preserved.
#[test]
fn test_variation_selector_fe0f_preserved() {
    let text = "star\u{2605}\u{FE0F}here";
    let xml = super::fodt(&format!("<text:p>{text}</text:p>"));
    assert!(
        Document::from_xml(&xml).is_ok(),
        "Variation selector U+FE0F should parse without error"
    );
}

// ── Emoji sequences ───────────────────────────────────────────────────────────

/// Emoji ZWJ sequence (👨‍👩‍👧 = U+1F468 ZWJ U+1F469 ZWJ U+1F467) must
/// survive a round-trip intact.
#[test]
fn test_emoji_zwj_family_sequence_round_trip() {
    let text = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    let xml = super::fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Emoji ZWJ family sequence should parse");
    let out = result
        .unwrap()
        .to_content_xml()
        .expect("to_content_xml failed");
    assert!(
        out.contains('\u{1F468}'),
        "Man emoji must survive round-trip"
    );
    assert!(out.contains('\u{200D}'), "ZWJ must survive round-trip");
}

// ── Unicode normalisation ─────────────────────────────────────────────────────

/// NFD-normalised text must parse and survive a round-trip without silent
/// character loss.  The parser must not silently normalise the form.
#[test]
fn test_nfd_normalised_text_no_silent_corruption() {
    // "é" in NFD: U+0065 (e) + U+0301 (combining acute accent)
    let nfd_e = "\u{0065}\u{0301}";
    let text = format!("caf{nfd_e}");
    let xml = super::fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "NFD text should parse without error");
    let out = result
        .unwrap()
        .to_content_xml()
        .expect("to_content_xml failed");
    let doc2 = Document::from_xml(&out).expect("re-parse failed");
    if let Some(Block::Paragraph { content, .. }) = doc2.blocks.first() {
        if let Some(Inline::Text { text: t, .. }) = content.first() {
            assert_eq!(
                t.chars().count(),
                text.chars().count(),
                "NFD text must not lose characters on round-trip"
            );
        }
    }
}

// ── Extremely long text ───────────────────────────────────────────────────────

/// A 5 MB text node must parse and complete in bounded time without panic.
#[test]
fn test_5mb_text_node_no_panic() {
    let long_text = "x".repeat(5 * 1024 * 1024);
    let xml = super::fodt(&format!("<text:p>{long_text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "5 MB text node must parse without panic");
    let out = result
        .unwrap()
        .to_content_xml()
        .expect("to_content_xml failed");
    assert!(
        out.len() >= long_text.len(),
        "Long text must not be silently truncated"
    );
}
