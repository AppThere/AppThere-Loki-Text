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

//! XMP metadata generation for PDF/X compliance.

use crate::export_settings::{PdfExportSettings, PdfXStandard};

/// Build an XMP metadata packet for the PDF/X document.
///
/// The packet declares the GTS_PDFXVersion and includes basic dc: metadata.
pub fn build_xmp_packet(title: Option<&str>, settings: &PdfExportSettings) -> String {
    let gts_version = settings.standard.gts_version_string();
    let title_str = title.unwrap_or("Untitled");
    let conformance_attr = xmp_conformance_attr(settings.standard);

    format!(
        r#"<?xpacket begin="\xef\xbb\xbf" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="AppThere Loki">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/"
        xmlns:pdfx="http://ns.adobe.com/pdfx/1.3/"
        xmlns:xmpMM="http://ns.adobe.com/xap/1.0/mm/"
        {conformance_attr}>
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">{title_str}</rdf:li>
        </rdf:Alt>
      </dc:title>
      <pdfx:GTS_PDFXVersion>{gts_version}</pdfx:GTS_PDFXVersion>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>
"#,
        conformance_attr = conformance_attr,
        title_str = escape_xml(title_str),
        gts_version = gts_version,
    )
}

fn xmp_conformance_attr(standard: PdfXStandard) -> &'static str {
    match standard {
        PdfXStandard::X1a2001 => {
            r#"xmlns:pdfxid="http://www.npes.org/pdfx/ns/id/" pdfxid:GTS_PDFXConformance="PDF/X-1a:2001""#
        }
        PdfXStandard::X4_2008 => {
            r#"xmlns:pdfxid="http://www.npes.org/pdfx/ns/id/" pdfxid:GTS_PDFXConformance="PDF/X-4""#
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xmp_contains_gts_version_x4() {
        let settings = PdfExportSettings {
            standard: PdfXStandard::X4_2008,
            ..Default::default()
        };
        let xmp = build_xmp_packet(Some("Test Doc"), &settings);
        assert!(xmp.contains("PDF/X-4"), "XMP should declare PDF/X-4");
    }

    #[test]
    fn xmp_contains_gts_version_x1a() {
        let settings = PdfExportSettings {
            standard: PdfXStandard::X1a2001,
            output_condition_identifier: "FOGRA39".to_string(),
            ..Default::default()
        };
        let xmp = build_xmp_packet(Some("Print Doc"), &settings);
        assert!(
            xmp.contains("PDF/X-1a:2001"),
            "XMP should declare PDF/X-1a:2001"
        );
    }

    #[test]
    fn xmp_escapes_title_xml() {
        let settings = PdfExportSettings::default();
        let xmp = build_xmp_packet(Some("A & B <test>"), &settings);
        assert!(xmp.contains("A &amp; B &lt;test&gt;"));
    }
}
