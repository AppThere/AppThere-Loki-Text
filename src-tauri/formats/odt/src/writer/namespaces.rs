//! XML namespace attribute helpers for ODT writers.
//!
//! Provides functions to push the standard ODF namespace declarations
//! onto `quick_xml::BytesStart` elements, eliminating repeated boilerplate
//! across the content, styles, and FODT writers.

use quick_xml::events::BytesStart;

/// Pushes all standard ODF namespace attributes for `office:document-content`.
///
/// Used when generating `content.xml` for ZIP-format ODT files.
pub fn push_content_ns(elem: &mut BytesStart) {
    push_common_ns(elem);
    elem.push_attribute((
        "xmlns:table",
        "urn:oasis:names:tc:opendocument:xmlns:table:1.0",
    ));
    elem.push_attribute((
        "xmlns:draw",
        "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0",
    ));
    elem.push_attribute((
        "xmlns:number",
        "urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0",
    ));
    elem.push_attribute((
        "xmlns:svg",
        "urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0",
    ));
    elem.push_attribute((
        "xmlns:chart",
        "urn:oasis:names:tc:opendocument:xmlns:chart:1.0",
    ));
    elem.push_attribute((
        "xmlns:dr3d",
        "urn:oasis:names:tc:opendocument:xmlns:dr3d:1.0",
    ));
    elem.push_attribute(("xmlns:math", "http://www.w3.org/1998/Math/MathML"));
    elem.push_attribute((
        "xmlns:form",
        "urn:oasis:names:tc:opendocument:xmlns:form:1.0",
    ));
    elem.push_attribute((
        "xmlns:script",
        "urn:oasis:names:tc:opendocument:xmlns:script:1.0",
    ));
    elem.push_attribute(("xmlns:loki", "https://appthere.com/loki/ns"));
    elem.push_attribute(("office:version", "1.3"));
}

/// Pushes all standard ODF namespace attributes for `office:document` (FODT).
///
/// Used when generating a standalone FODT flat XML document.
pub fn push_fodt_ns(elem: &mut BytesStart) {
    push_common_ns(elem);
    elem.push_attribute((
        "xmlns:table",
        "urn:oasis:names:tc:opendocument:xmlns:table:1.0",
    ));
    elem.push_attribute((
        "xmlns:draw",
        "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0",
    ));
    elem.push_attribute((
        "xmlns:number",
        "urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0",
    ));
    elem.push_attribute((
        "xmlns:svg",
        "urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0",
    ));
    elem.push_attribute((
        "xmlns:chart",
        "urn:oasis:names:tc:opendocument:xmlns:chart:1.0",
    ));
    elem.push_attribute((
        "xmlns:dr3d",
        "urn:oasis:names:tc:opendocument:xmlns:dr3d:1.0",
    ));
    elem.push_attribute(("xmlns:math", "http://www.w3.org/1998/Math/MathML"));
    elem.push_attribute((
        "xmlns:form",
        "urn:oasis:names:tc:opendocument:xmlns:form:1.0",
    ));
    elem.push_attribute((
        "xmlns:script",
        "urn:oasis:names:tc:opendocument:xmlns:script:1.0",
    ));
    elem.push_attribute(("xmlns:loki", "https://appthere.com/loki/ns"));
    elem.push_attribute(("office:mimetype", "application/vnd.oasis.opendocument.text"));
    elem.push_attribute(("office:version", "1.3"));
}

/// Pushes namespace attributes for `office:document-styles`.
pub fn push_styles_doc_ns(elem: &mut BytesStart) {
    push_common_ns(elem);
    elem.push_attribute((
        "xmlns:table",
        "urn:oasis:names:tc:opendocument:xmlns:table:1.0",
    ));
    elem.push_attribute((
        "xmlns:draw",
        "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0",
    ));
    elem.push_attribute((
        "xmlns:number",
        "urn:oasis:names:tc:opendocument:xmlns:datastyle:1.0",
    ));
    elem.push_attribute((
        "xmlns:svg",
        "urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0",
    ));
    elem.push_attribute((
        "xmlns:chart",
        "urn:oasis:names:tc:opendocument:xmlns:chart:1.0",
    ));
    elem.push_attribute((
        "xmlns:dr3d",
        "urn:oasis:names:tc:opendocument:xmlns:dr3d:1.0",
    ));
    elem.push_attribute(("xmlns:math", "http://www.w3.org/1998/Math/MathML"));
    elem.push_attribute((
        "xmlns:form",
        "urn:oasis:names:tc:opendocument:xmlns:form:1.0",
    ));
    elem.push_attribute((
        "xmlns:script",
        "urn:oasis:names:tc:opendocument:xmlns:script:1.0",
    ));
    elem.push_attribute(("xmlns:loki", "https://appthere.com/loki/ns"));
    elem.push_attribute(("office:version", "1.3"));
}

/// Pushes the core ODF namespace declarations shared by all document types.
fn push_common_ns(elem: &mut BytesStart) {
    elem.push_attribute((
        "xmlns:office",
        "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
    ));
    elem.push_attribute((
        "xmlns:text",
        "urn:oasis:names:tc:opendocument:xmlns:text:1.0",
    ));
    elem.push_attribute((
        "xmlns:style",
        "urn:oasis:names:tc:opendocument:xmlns:style:1.0",
    ));
    elem.push_attribute((
        "xmlns:fo",
        "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0",
    ));
    elem.push_attribute(("xmlns:xlink", "http://www.w3.org/1999/xlink"));
    elem.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
    elem.push_attribute((
        "xmlns:meta",
        "urn:oasis:names:tc:opendocument:xmlns:meta:1.0",
    ));
}
