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
//! vectors not addressed by the baseline error-handling suite.
//!
//! # Categories
//!
//! 1. [`invalid_xml`] — illegal control characters, undeclared namespaces,
//!    namespace prefix conflicts, truncated documents, extreme nesting depth.
//! 2. [`unicode_edge_cases`] — surrogate code-point references, bidi
//!    isolation/override characters, ZWJ sequences, NFD text, 5 MB text nodes.
//! 3. [`security`] — path traversal in resource references, attribute-count
//!    explosion, namespace flooding, PUBLIC DOCTYPE entities.

pub mod invalid_xml;
pub mod security;
pub mod unicode_edge_cases;

// ── Shared helpers ────────────────────────────────────────────────────────────

const NS_OFFICE: &str = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
const NS_TEXT: &str = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
const NS_STYLE: &str = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
const NS_FO: &str = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
const NS_TABLE: &str = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";
const NS_XLINK: &str = "http://www.w3.org/1999/xlink";

/// Minimal valid FODT wrapper.
pub fn fodt(body: &str) -> String {
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

/// Minimal valid FODT wrapper that also declares the xlink namespace.
pub fn fodt_xlink(body: &str) -> String {
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
