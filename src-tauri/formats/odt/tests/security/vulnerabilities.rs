//! Tests against XML attack vectors: billion laughs (XML bomb) and XXE.
//!
//! roxmltree intentionally does not expand entity references or load external
//! resources, so these attacks are blocked at the parser level. Each test
//! verifies either: (a) the input is rejected, or (b) if accepted, no
//! sensitive content (file paths, expanded entities) appears in the output.

use odt_format::Document;

// ── Entity expansion (billion laughs / XML bomb) ──────────────────────────────

/// DOCTYPE with recursive entity definitions must not exhaust memory.
///
/// roxmltree does not support DOCTYPE or entity expansion, so this input
/// is expected to fail. Either outcome (err or ok-with-no-expansion) is safe;
/// the test verifies the process completes quickly and never panics.
#[test]
fn test_xml_bomb_rejected_or_unexpanded() {
    let bomb = r#"<?xml version="1.0"?>
<!DOCTYPE lolz [
  <!ENTITY lol "lol">
  <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
  <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
]>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0">
  <office:body><office:text>&lol3;</office:text></office:body>
</office:document>"#;

    // Must complete quickly — not hang waiting for exponential expansion.
    let result = Document::from_xml(bomb);

    if let Ok(doc) = result {
        // If the parser accepted the document, entities must NOT have been
        // expanded (i.e. the output must not contain billions of "lol"s).
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(
            out.len() < 10_000,
            "Entity expansion must not occur: output too large ({} bytes)",
            out.len()
        );
    }
    // is_err() is also an acceptable outcome.
}

// ── External entity injection (XXE) ──────────────────────────────────────────

/// An external entity referencing `/etc/passwd` must not cause the file's
/// contents to appear in the parsed document.
#[test]
fn test_xxe_file_system_entity_not_loaded() {
    let xxe = r#"<?xml version="1.0"?>
<!DOCTYPE foo [
  <!ENTITY xxe SYSTEM "file:///etc/passwd">
]>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0">
  <office:body><office:text><text:p xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">&xxe;</text:p></office:text></office:body>
</office:document>"#;

    let result = Document::from_xml(xxe);

    if let Ok(doc) = result {
        let out = doc.to_content_xml().unwrap_or_default();
        // The classic marker in /etc/passwd that would confirm file was read.
        assert!(
            !out.contains("root:"),
            "External entity must not be loaded: /etc/passwd content detected"
        );
        assert!(
            !out.contains("/bin/sh"),
            "External entity must not be loaded: shell path detected"
        );
    }
    // is_err() means the parser rejected it — also safe.
}

/// An XXE attack via parameter entities must not be expanded.
#[test]
fn test_xxe_parameter_entity_not_expanded() {
    let xxe = r#"<?xml version="1.0"?>
<!DOCTYPE foo [
  <!ENTITY % passwd SYSTEM "file:///etc/passwd">
  %passwd;
]>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0">
  <office:body><office:text></office:text></office:body>
</office:document>"#;

    let result = Document::from_xml(xxe);

    if let Ok(doc) = result {
        let out = doc.to_content_xml().unwrap_or_default();
        assert!(!out.contains("root:"), "Parameter entity must not be loaded");
    }
}

/// An SSRF-style entity pointing to an HTTP URL must not trigger a network
/// request.  roxmltree never makes network calls; verify the result is safe.
#[test]
fn test_xxe_http_url_not_fetched() {
    let xxe = r#"<?xml version="1.0"?>
<!DOCTYPE foo [
  <!ENTITY remote SYSTEM "http://169.254.169.254/latest/meta-data/">
]>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0">
  <office:body><office:text><text:p xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">&remote;</text:p></office:text></office:body>
</office:document>"#;

    let result = Document::from_xml(xxe);

    if let Ok(doc) = result {
        let out = doc.to_content_xml().unwrap_or_default();
        // AWS instance metadata would contain "ami-" or "instance-id".
        assert!(
            !out.contains("ami-") && !out.contains("instance-id"),
            "HTTP entity must not be fetched"
        );
    }
}
