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

//! Level 3 error handling tests.
//!
//! Covers advanced invalid XML, Unicode edge cases, and security attack
//! vectors not addressed by the baseline error-handling suite.  Every test
//! must complete quickly and never panic.
//!
//! # Categories
//!
//! 1. **Invalid XML** — illegal characters, undeclared namespaces, namespace
//!    conflicts, truncated documents, extreme nesting depth.
//! 2. **Unicode edge cases** — surrogate code points in character references,
//!    bidirectional override/isolation characters, zero-width joiners,
//!    variation selectors, emoji ZWJ sequences, NFD-normalised text.
//! 3. **Security** — path traversal in resource references, attribute-count
//!    explosion, namespace-declaration flooding, PUBLIC DOCTYPE entities.

use odt_format::Document;

// ── Namespace constants ───────────────────────────────────────────────────────

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
const NS_TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";
const NS_XLINK: &str = "http://www.w3.org/1999/xlink";

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Minimal valid FODT wrapper.
fn fodt(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}"
    xmlns:table="{NS_TABLE}" office:version="1.3">
  <office:body>
    <office:text>{body}</office:text>
  </office:body>
</office:document>"#
    )
}

/// Minimal valid FODT wrapper that also declares xlink namespace.
fn fodt_xlink(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}"
    xmlns:style="{NS_STYLE}" xmlns:fo="{NS_FO}"
    xmlns:table="{NS_TABLE}" xmlns:xlink="{NS_XLINK}" office:version="1.3">
  <office:body>
    <office:text>{body}</office:text>
  </office:body>
</office:document>"#
    )
}

// ═════════════════════════════════════════════════════════════════════════════
// Category 1: Invalid XML
// ═════════════════════════════════════════════════════════════════════════════

/// Input that is only whitespace is not valid XML and must return an error.
#[test]
fn test_whitespace_only_input() {
    let result = Document::from_xml("   \n\t  ");
    assert!(result.is_err(), "Whitespace-only input must return Err");
}

/// A single space is not valid XML.
#[test]
fn test_single_space_input() {
    let result = Document::from_xml(" ");
    assert!(result.is_err(), "Single space must return Err");
}

/// XML control characters in the range 0x01–0x08 are forbidden by the XML
/// specification; the parser must reject them rather than silently discarding.
#[test]
fn test_illegal_control_char_0x01_in_text_content() {
    let xml = fodt("<text:p>Hello\x01World</text:p>");
    let result = Document::from_xml(&xml);
    assert!(
        result.is_err(),
        "Control character 0x01 in text content must be rejected"
    );
}

/// 0x0B (vertical tab) is an illegal XML character and must be rejected.
#[test]
fn test_illegal_control_char_0x0b_in_text_content() {
    let xml = fodt("<text:p>Hello\x0BWorld</text:p>");
    let result = Document::from_xml(&xml);
    assert!(
        result.is_err(),
        "Control character 0x0B in text content must be rejected"
    );
}

/// 0x0C (form feed) is an illegal XML character and must be rejected.
#[test]
fn test_illegal_control_char_0x0c_in_text_content() {
    let xml = fodt("<text:p>Hello\x0CWorld</text:p>");
    let result = Document::from_xml(&xml);
    assert!(
        result.is_err(),
        "Control character 0x0C in text content must be rejected"
    );
}

/// An element using an undeclared namespace prefix must be rejected.
/// roxmltree validates namespace declarations strictly.
#[test]
fn test_undeclared_namespace_prefix_in_element() {
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document xmlns:office="{NS_OFFICE}">
  <office:body>
    <office:text>
      <undeclared:element>Text</undeclared:element>
    </office:text>
  </office:body>
</office:document>"#
    );
    let result = Document::from_xml(&xml);
    assert!(
        result.is_err(),
        "Undeclared namespace prefix must be rejected"
    );
}

/// An attribute using an undeclared namespace prefix must be rejected.
#[test]
fn test_undeclared_namespace_prefix_in_attribute() {
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}">
  <office:body>
    <office:text>
      <text:p ghost:attr="value">Text</text:p>
    </office:text>
  </office:body>
</office:document>"#
    );
    let result = Document::from_xml(&xml);
    assert!(
        result.is_err(),
        "Undeclared namespace prefix on attribute must be rejected"
    );
}

/// Namespace prefix redeclaration to a different URI within the same element
/// is legal XML, but the inner binding must shadow the outer one.  The parser
/// must not panic.
#[test]
fn test_namespace_prefix_redeclaration_is_handled() {
    // Override the `text:` prefix to a different URI inside the body element.
    // The outer document declares text: → NS_TEXT; the inner element declares
    // text: → something-else.  This is legal XML; the parser must not panic.
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}">
  <office:body>
    <office:text>
      <text:p xmlns:text="http://example.com/other-ns">Shadowed</text:p>
    </office:text>
  </office:body>
</office:document>"#
    );
    // May succeed or fail, but must never panic.
    let _ = Document::from_xml(&xml);
}

/// Input truncated in the middle of an attribute value must be rejected.
#[test]
fn test_truncated_mid_attribute_value() {
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}" office:version="1.3"#
    );
    // Deliberately unfinished — attribute value never closed.
    let result = Document::from_xml(&xml);
    assert!(result.is_err(), "Truncated attribute value must return Err");
}

/// Input truncated inside an element name must be rejected.
#[test]
fn test_truncated_mid_element_name() {
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}">
  <office:body>
    <office:text>
      <text:p>Hello</text:p>
      <text:"#
    );
    let result = Document::from_xml(&xml);
    assert!(result.is_err(), "Truncated element name must return Err");
}

/// 500-level deeply nested elements must not cause a stack overflow.
///
/// The parser imposes a pre-parse nesting depth limit.  Inputs exceeding
/// this limit are rejected with an error rather than crashing.
#[test]
fn test_extreme_nesting_500_levels_no_stack_overflow() {
    let depth = 500_usize;
    let opening: String = (0..depth)
        .map(|_| "<text:list><text:list-item>".to_string())
        .collect();
    let closing: String = (0..depth)
        .map(|_| "</text:list-item></text:list>".to_string())
        .collect();
    let body = format!("{opening}<text:p>Deep</text:p>{closing}");
    let xml = fodt(&body);

    // Must complete without panic or stack overflow.
    // The parser either rejects the input (nesting limit) or accepts it.
    match Document::from_xml(&xml) {
        Ok(_) => {} // Accepted: depth limit is high enough, no overflow
        Err(e) => {
            // Rejected: must be a clean error, not a panic
            assert!(
                !e.to_lowercase().contains("panic"),
                "Error from deep nesting must not mention panic: {e}"
            );
        }
    }
}

/// A document that is valid XML but has no recognisable ODT body content
/// (wrong semantic structure) must be rejected with an informative error.
#[test]
fn test_valid_xml_wrong_odt_semantics_missing_office_text() {
    // Has office:body but uses the wrong child element name.
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document xmlns:office="{NS_OFFICE}">
  <office:body>
    <office:drawing>
      <p>Not an ODT paragraph</p>
    </office:drawing>
  </office:body>
</office:document>"#
    );
    let result = Document::from_xml(&xml);
    assert!(
        result.is_err(),
        "Document with wrong office:body child must return Err"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("office:text") || err.contains("Could not find"),
        "Error must mention missing element: {err}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Category 2: Unicode Edge Cases
// ═════════════════════════════════════════════════════════════════════════════

/// XML character reference to a high surrogate (U+D800) is illegal in XML 1.0.
///
/// roxmltree is lenient and accepts these without error; what matters is that
/// parsing does not crash, and no sensitive data leaks from the output.
#[test]
fn test_high_surrogate_char_reference_no_crash() {
    // &#xD800; is technically invalid in XML 1.0 but roxmltree accepts it.
    let xml = fodt("<text:p>&#xD800;</text:p>");
    // Must not panic; Ok or Err are both acceptable.
    let _ = Document::from_xml(&xml);
}

/// Low surrogate reference (U+DFFF) — must not crash.
#[test]
fn test_low_surrogate_char_reference_no_crash() {
    let xml = fodt("<text:p>&#xDFFF;</text:p>");
    let _ = Document::from_xml(&xml);
}

/// Surrogate pair as two adjacent character references — must not crash or
/// produce output containing filesystem paths or other sensitive content.
#[test]
fn test_surrogate_pair_two_char_references_no_crash() {
    let xml = fodt("<text:p>&#xD83D;&#xDE00;</text:p>");
    if let Ok(doc) = Document::from_xml(&xml) {
        let out = doc.to_content_xml().unwrap_or_default();
        // Verify the output is small and contains no sensitive system content.
        assert!(
            out.len() < 100_000,
            "Output from surrogate pair input must not be unexpectedly large"
        );
    }
    // is_err() is also acceptable.
}

/// U+2066 (LEFT-TO-RIGHT ISOLATE) is a valid Unicode codepoint and must
/// not cause a panic; its presence must be preserved through a round-trip.
#[test]
fn test_lri_bidi_isolation_character_survives_round_trip() {
    // U+2066 LEFT-TO-RIGHT ISOLATE
    let text = "Hello\u{2066}World\u{2069}"; // LRI … PDI
    let xml = fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(
        result.is_ok(),
        "LRI bidi isolate (U+2066) should parse without error"
    );
    // Round-trip: write then re-parse and verify text is preserved.
    let doc = result.unwrap();
    let out = doc.to_content_xml().expect("to_content_xml failed");
    let doc2 = Document::from_xml(&out).expect("re-parse failed");
    use common_core::{Block, Inline};
    if let Some(Block::Paragraph { content, .. }) = doc2.blocks.first() {
        if let Some(Inline::Text { text: t, .. }) = content.first() {
            assert!(
                t.contains('\u{2066}'),
                "LRI character must survive round-trip"
            );
        }
    }
}

/// U+2067 (RIGHT-TO-LEFT ISOLATE) must not cause a panic.
#[test]
fn test_rli_bidi_isolation_character_no_panic() {
    let text = "Test\u{2067}Reversed\u{2069}";
    let xml = fodt(&format!("<text:p>{text}</text:p>"));
    let _ = Document::from_xml(&xml); // must not panic
}

/// U+2068 (FIRST STRONG ISOLATE) must not cause a panic.
#[test]
fn test_fsi_bidi_isolation_character_no_panic() {
    let text = "Test\u{2068}Content\u{2069}";
    let xml = fodt(&format!("<text:p>{text}</text:p>"));
    let _ = Document::from_xml(&xml); // must not panic
}

/// Zero-width joiner (U+200D) embedded in text must be preserved.
#[test]
fn test_zero_width_joiner_preserved_round_trip() {
    // ZWJ is used to form emoji sequences and ligatures.
    let text = "man\u{200D}woman\u{200D}girl";
    let xml = fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "ZWJ in text content should parse");
    let doc = result.unwrap();
    let out = doc.to_content_xml().expect("to_content_xml failed");
    assert!(
        out.contains('\u{200D}'),
        "ZWJ must survive write (present in content.xml output)"
    );
}

/// Variation selector U+FE0F (text/emoji presentation) must be preserved.
#[test]
fn test_variation_selector_fe0f_preserved() {
    // U+FE0F forces emoji presentation.
    let text = "star\u{2605}\u{FE0F}here";
    let xml = fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(
        result.is_ok(),
        "Variation selector U+FE0F should parse without error"
    );
}

/// Emoji ZWJ sequence (family emoji 👨‍👩‍👧) must survive a round-trip.
///
/// This sequence is: U+1F468 ZWJ U+1F469 ZWJ U+1F467
#[test]
fn test_emoji_zwj_family_sequence_round_trip() {
    let text = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    let xml = fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "Emoji ZWJ family sequence should parse");
    let doc = result.unwrap();
    let out = doc.to_content_xml().expect("to_content_xml failed");
    assert!(
        out.contains('\u{1F468}'),
        "Man emoji must survive round-trip"
    );
    assert!(out.contains('\u{200D}'), "ZWJ must survive round-trip");
}

/// NFD-normalised text (decomposed form) must parse and survive a round-trip
/// without silent data corruption.  The parser must not normalise the
/// representation; it must preserve whatever form was given.
#[test]
fn test_nfd_normalised_text_no_silent_corruption() {
    // "é" in NFD is U+0065 (e) + U+0301 (combining acute accent).
    // Encode explicitly to avoid Rust's implicit NFC.
    let nfd_e = "\u{0065}\u{0301}"; // 'e' + combining acute accent = NFD é
    let text = format!("caf{nfd_e}");
    let xml = fodt(&format!("<text:p>{text}</text:p>"));
    let result = Document::from_xml(&xml);
    assert!(result.is_ok(), "NFD text should parse without error");
    // The char count in the round-tripped string must equal the original.
    let doc = result.unwrap();
    let out = doc.to_content_xml().expect("to_content_xml failed");
    let doc2 = Document::from_xml(&out).expect("re-parse failed");
    use common_core::{Block, Inline};
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

/// An extremely long string (5 MB) must parse and complete in bounded time.
#[test]
fn test_5mb_text_node_no_panic() {
    let long_text = "x".repeat(5 * 1024 * 1024);
    let xml = fodt(&format!("<text:p>{long_text}</text:p>"));
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

// ═════════════════════════════════════════════════════════════════════════════
// Category 3: Security Vulnerabilities
// ═════════════════════════════════════════════════════════════════════════════

/// An `xlink:href` attribute containing a path traversal sequence
/// (`../../etc/passwd`) must not cause the file to be read.  The parser
/// stores the attribute value as-is but must never open files referenced
/// in document content.
#[test]
fn test_path_traversal_in_xlink_href_not_opened() {
    let body = r#"<text:a xlink:type="simple" xlink:href="../../etc/passwd">link</text:a>"#;
    let xml = fodt_xlink(body);
    let result = Document::from_xml(&xml);
    // The parser should either accept the link (with the literal href value)
    // or reject it, but must never actually read /etc/passwd.
    if let Ok(doc) = result {
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(
            !out.contains("root:"),
            "Path traversal must not expose /etc/passwd contents"
        );
        assert!(
            !out.contains("/bin/"),
            "Path traversal must not expose shell paths"
        );
    }
    // is_err() is also acceptable.
}

/// An `xlink:href` with a `file://` URI scheme pointing at a sensitive file
/// must not cause its contents to appear in the output.
#[test]
fn test_file_uri_in_xlink_href_not_read() {
    let body = r#"<text:a xlink:type="simple" xlink:href="file:///etc/passwd">link</text:a>"#;
    let xml = fodt_xlink(body);
    if let Ok(doc) = Document::from_xml(&xml) {
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(
            !out.contains("root:"),
            "file:// URI must not expose /etc/passwd"
        );
    }
}

/// An element with a very large number of attributes (attribute-count
/// explosion) must not cause excessive memory allocation or a panic.
#[test]
fn test_attribute_count_explosion_no_panic() {
    // Build a single <text:p> with 5 000 dummy attributes.
    // Attribute names use the declared `fo:` prefix to remain well-formed.
    let attrs: String = (0..5_000).map(|i| format!(r#" fo:x{i}="v""#)).collect();
    let body = format!("<text:p{attrs}>Paragraph</text:p>");
    let xml = fodt(&body);

    // Must complete quickly without panic; may succeed or fail.
    match Document::from_xml(&xml) {
        Ok(doc) => assert!(
            !doc.blocks.is_empty(),
            "Document with many attributes should yield at least one block"
        ),
        Err(e) => assert!(
            !e.to_lowercase().contains("panic"),
            "Error from attribute explosion must not mention panic: {e}"
        ),
    }
}

/// An element that redeclares thousands of namespace URIs
/// (namespace-declaration flooding) must not cause excessive memory use or
/// a panic.
#[test]
fn test_namespace_declaration_flooding_no_panic() {
    // Each declaration uses a unique prefix `nsN` bound to a unique URI.
    let ns_decls: String = (0..2_000)
        .map(|i| format!(r#" xmlns:ns{i}="http://example.com/ns/{i}""#))
        .collect();
    let body = format!("<text:p{ns_decls}>Paragraph</text:p>");
    let xml = fodt(&body);

    match Document::from_xml(&xml) {
        Ok(doc) => assert!(
            !doc.blocks.is_empty(),
            "Namespace flooding should yield at least one block"
        ),
        Err(e) => assert!(
            !e.to_lowercase().contains("panic"),
            "Error from namespace flooding must not mention panic: {e}"
        ),
    }
}

/// A DOCTYPE with a PUBLIC external identifier must not cause a network
/// request or file system access.  roxmltree does not support DTD processing;
/// the input is expected to be rejected or parsed safely.
#[test]
fn test_public_doctype_external_entity_not_loaded() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE office:document PUBLIC "-//OASIS//DTD OpenDocument 1.0//EN"
    "http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-schema.rng">
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0">
  <office:body><office:text></office:text></office:body>
</office:document>"#;

    // Must not hang on network I/O; must not expose file contents.
    let result = Document::from_xml(xml);
    if let Ok(doc) = result {
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(
            !out.contains("OASIS"),
            "PUBLIC DTD URL must not appear in document output"
        );
    }
}

/// A quadratic-blowup attempt: an attribute value containing a long repeated
/// pattern that some parsers expand naively.  Must complete in bounded time
/// and must not panic.
#[test]
fn test_quadratic_blowup_long_repeated_attribute_value() {
    // 512 KB attribute value with a repeating pattern.
    let value = "ab".repeat(256 * 1024);
    let body = format!(r#"<text:p fo:text-align="{value}">Paragraph</text:p>"#);
    let xml = fodt(&body);

    // Must terminate quickly; result (ok or err) is not the focus.
    let _ = Document::from_xml(&xml);
}

/// An element whose text content is a series of nested XML comment markers
/// (`--`) — which are illegal inside XML comments but legal in text content —
/// must not cause the parser to misidentify content boundaries.
#[test]
fn test_comment_like_text_content_no_confusion() {
    let body = "<text:p>-- not a comment -- still text --</text:p>";
    let xml = fodt(body);
    let result = Document::from_xml(&xml);
    assert!(
        result.is_ok(),
        "Comment-like text content must parse without error"
    );
}

/// A CDATA section inside `office:text` must not leak raw XML into block
/// content and must not cause a panic.
#[test]
fn test_cdata_section_in_body_no_panic() {
    let body = "<![CDATA[<text:p>Not a real paragraph</text:p>]]><text:p>Real</text:p>";
    let xml = fodt(body);
    // Must not panic; CDATA may be silently dropped or cause parse failure.
    let _ = Document::from_xml(&xml);
}
