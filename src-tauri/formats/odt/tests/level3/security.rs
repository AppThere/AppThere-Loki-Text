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

//! Level 3 — Category 3: Security vulnerabilities.
//!
//! Tests for path traversal in resource references, attribute-count
//! explosion, namespace-declaration flooding, PUBLIC DOCTYPE entities,
//! quadratic-blowup inputs, and parser-confusion inputs.

use odt_format::Document;

// ── Path traversal ────────────────────────────────────────────────────────────

/// An `xlink:href` containing `../../etc/passwd` must not cause the file to
/// be read.  The parser stores the literal attribute value but never opens it.
#[test]
fn test_path_traversal_in_xlink_href_not_opened() {
    let body = r#"<text:a xlink:type="simple" xlink:href="../../etc/passwd">link</text:a>"#;
    let xml = super::fodt_xlink(body);
    if let Ok(doc) = Document::from_xml(&xml) {
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(
            !out.contains("root:"),
            "Path traversal must not expose /etc/passwd"
        );
        assert!(
            !out.contains("/bin/"),
            "Path traversal must not expose shell paths"
        );
    }
    // is_err() is also acceptable.
}

/// An `xlink:href` with a `file://` URI must not cause its contents to appear
/// in the output.
#[test]
fn test_file_uri_in_xlink_href_not_read() {
    let body = r#"<text:a xlink:type="simple" xlink:href="file:///etc/passwd">link</text:a>"#;
    let xml = super::fodt_xlink(body);
    if let Ok(doc) = Document::from_xml(&xml) {
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(
            !out.contains("root:"),
            "file:// URI must not expose /etc/passwd"
        );
    }
}

// ── Denial-of-service inputs ──────────────────────────────────────────────────

/// A single element with 5 000 attributes must not cause a panic.
#[test]
fn test_attribute_count_explosion_no_panic() {
    let attrs: String = (0..5_000).map(|i| format!(r#" fo:x{i}="v""#)).collect();
    let body = format!("<text:p{attrs}>Paragraph</text:p>");
    let xml = super::fodt(&body);
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

/// An element with 2 000 namespace declarations must not cause a panic.
#[test]
fn test_namespace_declaration_flooding_no_panic() {
    let ns_decls: String = (0..2_000)
        .map(|i| format!(r#" xmlns:ns{i}="http://example.com/ns/{i}""#))
        .collect();
    let body = format!("<text:p{ns_decls}>Paragraph</text:p>");
    let xml = super::fodt(&body);
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

/// A DOCTYPE with a PUBLIC external identifier must not trigger a network
/// request or file system access.
#[test]
fn test_public_doctype_external_entity_not_loaded() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE office:document PUBLIC "-//OASIS//DTD OpenDocument 1.0//EN"
    "http://docs.oasis-open.org/office/v1.2/os/OpenDocument-v1.2-schema.rng">
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0">
  <office:body><office:text></office:text></office:body>
</office:document>"#;
    if let Ok(doc) = Document::from_xml(xml) {
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(
            !out.contains("OASIS"),
            "PUBLIC DTD URL must not appear in document output"
        );
    }
}

/// A 512 KB repeated-pattern attribute value must complete in bounded time.
#[test]
fn test_quadratic_blowup_long_repeated_attribute_value() {
    let value = "ab".repeat(256 * 1024);
    let body = format!(r#"<text:p fo:text-align="{value}">Paragraph</text:p>"#);
    let xml = super::fodt(&body);
    let _ = Document::from_xml(&xml); // must terminate quickly
}

// ── Parser confusion inputs ───────────────────────────────────────────────────

/// `--` sequences in text content are legal in XML but look like comment
/// delimiters; the parser must not confuse them with actual comments.
#[test]
fn test_comment_like_text_content_no_confusion() {
    let body = "<text:p>-- not a comment -- still text --</text:p>";
    let xml = super::fodt(body);
    assert!(
        Document::from_xml(&xml).is_ok(),
        "Comment-like text content must parse without error"
    );
}

/// A CDATA section inside `office:text` must not leak raw XML into block
/// content and must not cause a panic.
#[test]
fn test_cdata_section_in_body_no_panic() {
    let body = "<![CDATA[<text:p>Not a real paragraph</text:p>]]><text:p>Real</text:p>";
    let xml = super::fodt(body);
    let _ = Document::from_xml(&xml); // must not panic
}
