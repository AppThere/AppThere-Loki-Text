use crate::{html::escape_xml, ContentSection};

/// Generate the EPUB 3 Navigation Document (nav.xhtml).
pub(crate) fn generate_nav_xhtml(sections: &[ContentSection]) -> String {
    let mut nav = String::new();

    nav.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    nav.push_str("<!DOCTYPE html>\n");
    nav.push_str(
        "<html xmlns=\"http://www.w3.org/1999/xhtml\" \
         xmlns:epub=\"http://www.idpf.org/2007/ops\">\n",
    );
    nav.push_str("<head>\n");
    nav.push_str("  <title>Navigation</title>\n");
    nav.push_str("</head>\n");
    nav.push_str("<body>\n");
    nav.push_str("  <nav epub:type=\"toc\">\n");
    nav.push_str("    <h1>Table of Contents</h1>\n");
    nav.push_str("    <ol>\n");

    for section in sections {
        let title = section.title.as_deref().unwrap_or("Section");
        nav.push_str(&format!(
            "      <li><a href=\"Text/{}.xhtml\">{}</a></li>\n",
            section.id,
            escape_xml(title)
        ));
    }

    nav.push_str("    </ol>\n");
    nav.push_str("  </nav>\n");
    nav.push_str("</body>\n");
    nav.push_str("</html>\n");

    nav
}
