use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
mod tests;

// Re-use types from odt-logic
pub use odt_logic::{
    Block, Inline, Metadata, StyleDefinition, TiptapAttrs, TiptapMark, TiptapNode,
};

/// Represents a section of content in the EPUB
/// Sections are split at HorizontalRule blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSection {
    pub id: String,
    pub title: Option<String>,
    pub blocks: Vec<Block>,
}

/// Represents a font asset to be bundled in the EPUB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontAsset {
    pub family_name: String,
    pub filename: String,
    pub data: Vec<u8>,
    pub format: FontFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FontFormat {
    TrueType,
    OpenType,
    WOFF,
    WOFF2,
}

impl FontFormat {
    pub fn media_type(&self) -> &str {
        match self {
            FontFormat::TrueType => "font/ttf",
            FontFormat::OpenType => "font/otf",
            FontFormat::WOFF => "font/woff",
            FontFormat::WOFF2 => "font/woff2",
        }
    }

    pub fn from_filename(filename: &str) -> Self {
        let ext = filename.split('.').next_back().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "ttf" => FontFormat::TrueType,
            "otf" => FontFormat::OpenType,
            "woff" => FontFormat::WOFF,
            "woff2" => FontFormat::WOFF2,
            _ => FontFormat::TrueType,
        }
    }
}

/// Main EPUB document structure
#[derive(Debug, Clone)]
pub struct EpubDocument {
    pub sections: Vec<ContentSection>,
    pub styles: HashMap<String, StyleDefinition>,
    pub metadata: Metadata,
    pub fonts: Vec<FontAsset>,
}

impl EpubDocument {
    /// Create a new EPUB document from Tiptap JSON
    /// Splits content at PageBreak blocks into sections
    pub fn from_tiptap(
        root: TiptapNode,
        styles: HashMap<String, StyleDefinition>,
        metadata: Metadata,
        fonts: Vec<FontAsset>,
    ) -> Self {
        let mut sections = Vec::new();
        let mut current_blocks = Vec::new();
        let mut section_counter = 1;

        // Extract blocks from the doc root
        let blocks = match root {
            TiptapNode::Doc { content } => content
                .into_iter()
                .filter_map(odt_logic::Document::tiptap_node_to_block)
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };

        // Split at PageBreak or Style Breaks
        for block in blocks {
            let mut break_before = false;
            let mut break_after = false;

            // Check style for page breaks
            let style_name = match &block {
                Block::Paragraph { style_name, .. } => style_name.as_deref(),
                Block::Heading { style_name, .. } => style_name.as_deref(),
                _ => None,
            };

            if let Some(name) = style_name {
                if let Some(style) = styles.get(name) {
                    if style.attributes.get("fo:break-before").map(|s| s.as_str()) == Some("page") {
                        break_before = true;
                    }
                    if style.attributes.get("fo:break-after").map(|s| s.as_str()) == Some("page") {
                        break_after = true;
                    }
                }
            }

            if break_before && !current_blocks.is_empty() {
                sections.push(ContentSection {
                    id: format!("section-{}", section_counter),
                    title: Some(format!("Section {}", section_counter)),
                    blocks: current_blocks.clone(),
                });
                current_blocks.clear();
                section_counter += 1;
            }

            if matches!(block, Block::PageBreak) {
                // Always split at explicit PageBreak
                if !current_blocks.is_empty() {
                    sections.push(ContentSection {
                        id: format!("section-{}", section_counter),
                        title: Some(format!("Section {}", section_counter)),
                        blocks: current_blocks.clone(),
                    });
                    current_blocks.clear();
                    section_counter += 1;
                }
                continue;
            }

            current_blocks.push(block.clone());

            if break_after && !current_blocks.is_empty() {
                sections.push(ContentSection {
                    id: format!("section-{}", section_counter),
                    title: Some(format!("Section {}", section_counter)),
                    blocks: current_blocks.clone(),
                });
                current_blocks.clear();
                section_counter += 1;
            }
        }

        // Add remaining blocks as final section
        if !current_blocks.is_empty() {
            sections.push(ContentSection {
                id: format!("section-{}", section_counter),
                title: Some(format!("Section {}", section_counter)),
                blocks: current_blocks,
            });
        }

        // If no sections created, create a default one
        if sections.is_empty() {
            sections.push(ContentSection {
                id: "section-1".to_string(),
                title: Some("Section 1".to_string()),
                blocks: Vec::new(),
            });
        }

        EpubDocument {
            sections,
            styles,
            metadata,
            fonts,
        }
    }

    /// Generate XHTML content for a section
    pub fn section_to_xhtml(&self, section: &ContentSection) -> String {
        let mut html = String::new();

        html.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\n");
        html.push_str("<head>\n");
        html.push_str(&format!(
            "  <title>{}</title>\n",
            section.title.as_deref().unwrap_or("Section")
        ));
        html.push_str(
            "  <link rel=\"stylesheet\" type=\"text/css\" href=\"../Styles/styles.css\"/>\n",
        );
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Render blocks
        for block in &section.blocks {
            html.push_str(&self.block_to_html(block));
        }

        html.push_str("</body>\n");
        html.push_str("</html>\n");

        html
    }

    fn block_to_html(&self, block: &Block) -> String {
        match block {
            Block::Paragraph {
                style_name,
                content,
                ..
            } => {
                let mut tag = "p".to_string();
                if let Some(ref name) = style_name {
                    if let Some(style) = self.styles.get(name) {
                        if let Some(level) = style.outline_level {
                            tag = format!("h{}", level);
                        }
                    }
                }

                let class = style_name
                    .as_ref()
                    .map(|s| format!(" class=\"style-{}\"", s.replace(' ', "-")))
                    .unwrap_or_default();
                format!(
                    "  <{}{}>{}</{}>\n",
                    tag,
                    class,
                    self.inlines_to_html(content),
                    tag
                )
            }
            Block::Heading {
                level,
                style_name,
                content,
                ..
            } => {
                let class = style_name
                    .as_ref()
                    .map(|s| format!(" class=\"style-{}\"", s.replace(' ', "-")))
                    .unwrap_or_default();
                format!(
                    "  <h{}{}>{}</h{}>\n",
                    level,
                    class,
                    self.inlines_to_html(content),
                    level
                )
            }
            Block::BulletList { content } => {
                let mut html = String::from("  <ul>\n");
                for item in content {
                    html.push_str(&self.block_to_html(item));
                }
                html.push_str("  </ul>\n");
                html
            }
            Block::OrderedList { content } => {
                let mut html = String::from("  <ol>\n");
                for item in content {
                    html.push_str(&self.block_to_html(item));
                }
                html.push_str("  </ol>\n");
                html
            }
            Block::ListItem { content } => {
                let mut html = String::from("    <li>");
                for block in content {
                    html.push_str(self.block_to_html(block).trim());
                }
                html.push_str("</li>\n");
                html
            }
            Block::Blockquote { content } => {
                let mut html = String::from("  <blockquote>\n");
                for block in content {
                    html.push_str(&self.block_to_html(block));
                }
                html.push_str("  </blockquote>\n");
                html
            }
            Block::HorizontalRule => String::from("  <hr/>\n"),
            _ => String::new(),
        }
    }

    fn inlines_to_html(&self, inlines: &[Inline]) -> String {
        let mut html = String::new();
        for inline in inlines {
            match inline {
                Inline::Text { text, marks, .. } => {
                    let mut wrapped_text = text.clone();

                    // Apply marks
                    for mark in marks {
                        wrapped_text = match mark {
                            TiptapMark::Bold => format!("<strong>{}</strong>", wrapped_text),
                            TiptapMark::Italic => format!("<em>{}</em>", wrapped_text),
                            TiptapMark::Underline => format!("<u>{}</u>", wrapped_text),
                            TiptapMark::Strike => format!("<s>{}</s>", wrapped_text),
                            TiptapMark::Superscript => format!("<sup>{}</sup>", wrapped_text),
                            TiptapMark::Subscript => format!("<sub>{}</sub>", wrapped_text),
                            TiptapMark::Link { attrs } => {
                                format!("<a href=\"{}\">{}</a>", attrs.href, wrapped_text)
                            }
                            _ => wrapped_text,
                        };
                    }

                    html.push_str(&wrapped_text);
                }
                Inline::LineBreak => {
                    html.push_str("<br/>");
                }
            }
        }
        html
    }

    /// Generate the package document (content.opf)
    pub fn to_package_opf(&self) -> String {
        let mut opf = String::new();

        opf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        opf.push_str("<package xmlns=\"http://www.idpf.org/2007/opf\" version=\"3.0\" unique-identifier=\"uuid\">\n");

        // Metadata
        opf.push_str("  <metadata xmlns:dc=\"http://purl.org/dc/elements/1.1/\">\n");

        let identifier = self
            .metadata
            .identifier
            .clone()
            .unwrap_or_else(|| format!("urn:uuid:{}", uuid::Uuid::new_v4()));
        opf.push_str(&format!(
            "    <dc:identifier id=\"uuid\">{}</dc:identifier>\n",
            identifier
        ));

        let title = self.metadata.title.as_deref().unwrap_or("Untitled");
        opf.push_str(&format!(
            "    <dc:title>{}</dc:title>\n",
            Self::escape_xml(title)
        ));

        if let Some(creator) = &self.metadata.creator {
            opf.push_str(&format!(
                "    <dc:creator>{}</dc:creator>\n",
                Self::escape_xml(creator)
            ));
        }

        let language = self.metadata.language.as_deref().unwrap_or("en");
        opf.push_str(&format!(
            "    <dc:language>{}</dc:language>\n",
            Self::escape_xml(language)
        ));

        let modified = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        opf.push_str(&format!(
            "    <meta property=\"dcterms:modified\">{}</meta>\n",
            modified
        ));

        opf.push_str("  </metadata>\n");

        // Manifest
        opf.push_str("  <manifest>\n");
        opf.push_str("    <item id=\"nav\" href=\"nav.xhtml\" media-type=\"application/xhtml+xml\" properties=\"nav\"/>\n");
        opf.push_str("    <item id=\"css\" href=\"Styles/styles.css\" media-type=\"text/css\"/>\n");

        // Content sections
        for section in &self.sections {
            opf.push_str(&format!(
                "    <item id=\"{}\" href=\"Text/{}.xhtml\" media-type=\"application/xhtml+xml\"/>\n",
                section.id, section.id
            ));
        }

        // Fonts
        for (idx, font) in self.fonts.iter().enumerate() {
            opf.push_str(&format!(
                "    <item id=\"font-{}\" href=\"Fonts/{}\" media-type=\"{}\"/>\n",
                idx,
                font.filename,
                font.format.media_type()
            ));
        }

        opf.push_str("  </manifest>\n");

        // Spine
        opf.push_str("  <spine>\n");
        for section in &self.sections {
            opf.push_str(&format!("    <itemref idref=\"{}\"/>\n", section.id));
        }
        opf.push_str("  </spine>\n");

        opf.push_str("</package>\n");

        opf
    }

    /// Generate the navigation document (nav.xhtml)
    pub fn to_nav_xhtml(&self) -> String {
        let mut nav = String::new();

        nav.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        nav.push_str("<!DOCTYPE html>\n");
        nav.push_str("<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\n");
        nav.push_str("<head>\n");
        nav.push_str("  <title>Navigation</title>\n");
        nav.push_str("</head>\n");
        nav.push_str("<body>\n");
        nav.push_str("  <nav epub:type=\"toc\">\n");
        nav.push_str("    <h1>Table of Contents</h1>\n");
        nav.push_str("    <ol>\n");

        for section in &self.sections {
            let title = section.title.as_deref().unwrap_or("Section");
            nav.push_str(&format!(
                "      <li><a href=\"Text/{}.xhtml\">{}</a></li>\n",
                section.id,
                Self::escape_xml(title)
            ));
        }

        nav.push_str("    </ol>\n");
        nav.push_str("  </nav>\n");
        nav.push_str("</body>\n");
        nav.push_str("</html>\n");

        nav
    }

    /// Generate CSS from styles
    pub fn to_css(&self) -> String {
        let mut css = String::new();

        // Font faces
        for font in &self.fonts {
            css.push_str(&format!(
                "@font-face {{\n  font-family: '{}';\n  src: url('../Fonts/{}');\n}}\n\n",
                font.family_name, font.filename
            ));
        }

        // Style classes
        for (name, style) in &self.styles {
            let class_name = name.replace(' ', "-");
            css.push_str(&format!(".style-{} {{\n", class_name));

            for (key, value) in &style.attributes {
                let css_prop = Self::odf_to_css_property(key);
                if !css_prop.is_empty() {
                    css.push_str(&format!("  {}: {};\n", css_prop, value));
                }
            }

            if let Some(transform) = &style.text_transform {
                css.push_str(&format!("  text-transform: {};\n", transform));
            }

            css.push_str("}\n\n");
        }

        css
    }

    fn odf_to_css_property(odf_prop: &str) -> String {
        match odf_prop {
            "fo:font-family" => "font-family".to_string(),
            "fo:font-size" => "font-size".to_string(),
            "fo:font-weight" => "font-weight".to_string(),
            "fo:font-style" => "font-style".to_string(),
            "fo:text-align" => "text-align".to_string(),
            "fo:margin-top" => "margin-top".to_string(),
            "fo:margin-bottom" => "margin-bottom".to_string(),
            "fo:margin-left" => "margin-left".to_string(),
            "fo:margin-right" => "margin-right".to_string(),
            "fo:text-indent" => "text-indent".to_string(),
            "fo:line-height" => "line-height".to_string(),
            _ => String::new(),
        }
    }

    fn escape_xml(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}
