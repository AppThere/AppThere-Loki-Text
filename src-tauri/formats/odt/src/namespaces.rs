//! ODF XML namespace constants.
//!
//! Centralizes all namespace URI strings used throughout ODT parsing and
//! writing to avoid magic strings scattered across the codebase.
//!
//! # Examples
//!
//! ```
//! use odt_format::namespaces::Ns;
//!
//! let ns = Ns::default();
//! assert_eq!(ns.office, "urn:oasis:names:tc:opendocument:xmlns:office:1.0");
//! ```

/// All ODF XML namespace URIs used in ODT documents.
///
/// Create with [`Ns::default()`] to obtain the standard set.
pub struct Ns {
    /// `urn:oasis:names:tc:opendocument:xmlns:office:1.0`
    pub office: &'static str,
    /// `urn:oasis:names:tc:opendocument:xmlns:text:1.0`
    pub text: &'static str,
    /// `urn:oasis:names:tc:opendocument:xmlns:style:1.0`
    pub style: &'static str,
    /// `urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0`
    pub fo: &'static str,
    /// `http://purl.org/dc/elements/1.1/`
    pub dc: &'static str,
    /// `urn:oasis:names:tc:opendocument:xmlns:meta:1.0`
    pub meta: &'static str,
    /// `urn:oasis:names:tc:opendocument:xmlns:drawing:1.0`
    pub draw: &'static str,
    /// `urn:oasis:names:tc:opendocument:xmlns:table:1.0`
    pub table: &'static str,
    /// `http://www.w3.org/1999/xlink`
    pub xlink: &'static str,
    /// `https://appthere.com/loki/ns`
    pub loki: &'static str,
}

impl Default for Ns {
    /// Returns the standard set of ODF namespace URIs.
    fn default() -> Self {
        Self {
            office: "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
            text: "urn:oasis:names:tc:opendocument:xmlns:text:1.0",
            style: "urn:oasis:names:tc:opendocument:xmlns:style:1.0",
            fo: "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0",
            dc: "http://purl.org/dc/elements/1.1/",
            meta: "urn:oasis:names:tc:opendocument:xmlns:meta:1.0",
            draw: "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0",
            table: "urn:oasis:names:tc:opendocument:xmlns:table:1.0",
            xlink: "http://www.w3.org/1999/xlink",
            loki: "https://appthere.com/loki/ns",
        }
    }
}

/// Returns the namespace prefix string for a given namespace URI.
///
/// Used when building attribute keys during style parsing.
///
/// # Examples
///
/// ```
/// use odt_format::namespaces::ns_prefix;
///
/// assert_eq!(ns_prefix("urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"), "fo:");
/// assert_eq!(ns_prefix("unknown"), "");
/// ```
#[must_use]
pub fn ns_prefix(ns: &str) -> &'static str {
    match ns {
        "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" => "fo:",
        "urn:oasis:names:tc:opendocument:xmlns:style:1.0" => "style:",
        "urn:oasis:names:tc:opendocument:xmlns:text:1.0" => "text:",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ns_default_office_uri() {
        let ns = Ns::default();
        assert_eq!(
            ns.office,
            "urn:oasis:names:tc:opendocument:xmlns:office:1.0"
        );
    }

    #[test]
    fn ns_default_dc_uri() {
        let ns = Ns::default();
        assert_eq!(ns.dc, "http://purl.org/dc/elements/1.1/");
    }

    #[test]
    fn ns_default_xlink_uri() {
        let ns = Ns::default();
        assert_eq!(ns.xlink, "http://www.w3.org/1999/xlink");
    }

    #[test]
    fn ns_prefix_fo() {
        assert_eq!(
            ns_prefix("urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"),
            "fo:"
        );
    }

    #[test]
    fn ns_prefix_style() {
        assert_eq!(
            ns_prefix("urn:oasis:names:tc:opendocument:xmlns:style:1.0"),
            "style:"
        );
    }

    #[test]
    fn ns_prefix_text() {
        assert_eq!(
            ns_prefix("urn:oasis:names:tc:opendocument:xmlns:text:1.0"),
            "text:"
        );
    }

    #[test]
    fn ns_prefix_unknown_returns_empty() {
        assert_eq!(ns_prefix("http://unknown.ns/"), "");
    }
}
