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

//! Level 3 — Category 1: Invalid XML.
//!
//! Tests for illegal characters, undeclared namespaces, namespace prefix
//! conflicts, truncated documents, and extreme nesting depth.

use odt_format::Document;

const NS_OFFICE: &str = super::NS_OFFICE;
const NS_TEXT: &str = super::NS_TEXT;

// ── Whitespace-only inputs ────────────────────────────────────────────────────

/// Input that is only whitespace is not valid XML and must return an error.
#[test]
fn test_whitespace_only_input() {
    assert!(
        Document::from_xml("   \n\t  ").is_err(),
        "Whitespace-only input must return Err"
    );
}

/// A single space is not valid XML.
#[test]
fn test_single_space_input() {
    assert!(
        Document::from_xml(" ").is_err(),
        "Single space must return Err"
    );
}

// ── Illegal control characters ────────────────────────────────────────────────

/// 0x01 is forbidden by the XML specification and must be rejected.
#[test]
fn test_illegal_control_char_0x01_in_text_content() {
    let xml = super::fodt("<text:p>Hello\x01World</text:p>");
    assert!(
        Document::from_xml(&xml).is_err(),
        "Control character 0x01 in text content must be rejected"
    );
}

/// 0x0B (vertical tab) is an illegal XML character and must be rejected.
#[test]
fn test_illegal_control_char_0x0b_in_text_content() {
    let xml = super::fodt("<text:p>Hello\x0BWorld</text:p>");
    assert!(
        Document::from_xml(&xml).is_err(),
        "Control character 0x0B in text content must be rejected"
    );
}

/// 0x0C (form feed) is an illegal XML character and must be rejected.
#[test]
fn test_illegal_control_char_0x0c_in_text_content() {
    let xml = super::fodt("<text:p>Hello\x0CWorld</text:p>");
    assert!(
        Document::from_xml(&xml).is_err(),
        "Control character 0x0C in text content must be rejected"
    );
}

// ── Namespace errors ──────────────────────────────────────────────────────────

/// An element using an undeclared namespace prefix must be rejected.
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
    assert!(
        Document::from_xml(&xml).is_err(),
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
    assert!(
        Document::from_xml(&xml).is_err(),
        "Undeclared namespace prefix on attribute must be rejected"
    );
}

/// Namespace prefix redeclaration to a different URI is legal XML; the
/// parser must not panic (may succeed or return an error).
#[test]
fn test_namespace_prefix_redeclaration_is_handled() {
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
    let _ = Document::from_xml(&xml); // must not panic
}

// ── Truncated inputs ──────────────────────────────────────────────────────────

/// Input truncated in the middle of an attribute value must be rejected.
#[test]
fn test_truncated_mid_attribute_value() {
    let xml = format!(
        r#"<?xml version="1.0"?>
<office:document xmlns:office="{NS_OFFICE}" xmlns:text="{NS_TEXT}" office:version="1.3"#
    );
    assert!(
        Document::from_xml(&xml).is_err(),
        "Truncated attribute value must return Err"
    );
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
    assert!(
        Document::from_xml(&xml).is_err(),
        "Truncated element name must return Err"
    );
}

// ── Extreme nesting ───────────────────────────────────────────────────────────

/// 500-level deeply nested elements must not cause a stack overflow.
///
/// The parser's pre-scan rejects inputs that exceed the nesting depth limit,
/// returning a clean error rather than aborting with a stack overflow.
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
    let xml = super::fodt(&body);

    match Document::from_xml(&xml) {
        Ok(_) => {} // accepted — depth limit is high enough, no overflow
        Err(e) => assert!(
            !e.to_lowercase().contains("panic"),
            "Error from deep nesting must not mention panic: {e}"
        ),
    }
}

// ── Wrong ODT semantics ───────────────────────────────────────────────────────

/// Valid XML whose `office:body` child is not `office:text` must be rejected.
#[test]
fn test_valid_xml_wrong_odt_semantics_missing_office_text() {
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
