use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-use types from common-core
pub use common_core::{
    Block, Inline, Metadata, StyleDefinition, StyleFamily, TiptapAttrs, TiptapMark, TiptapNode,
};

mod conversion;
mod css;
mod html;
mod nav;
mod opf;
mod table;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A section of content within the EPUB, split at page-break boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSection {
    pub id: String,
    pub title: Option<String>,
    pub blocks: Vec<Block>,
}

/// A font asset to be bundled in the EPUB.
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

/// An image asset decoded from a data URI and embedded in the EPUB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAsset {
    /// The original `src` value used to look up this asset during rendering.
    pub original_src: String,
    /// Filename inside `OEBPS/Images/` (e.g. `"image-000.png"`).
    pub filename: String,
    /// Raw image bytes.
    pub data: Vec<u8>,
    /// IANA media type string (e.g. `"image/png"`).
    pub media_type: String,
}

// ---------------------------------------------------------------------------
// EpubDocument
// ---------------------------------------------------------------------------

/// The assembled EPUB document ready for ZIP serialisation.
#[derive(Debug, Clone)]
pub struct EpubDocument {
    pub sections: Vec<ContentSection>,
    pub styles: HashMap<String, StyleDefinition>,
    pub metadata: Metadata,
    pub fonts: Vec<FontAsset>,
    /// Images decoded from data-URI `src` values found in the document.
    pub images: Vec<ImageAsset>,
}

impl EpubDocument {
    /// Build an `EpubDocument` from a `TiptapNode` document tree.
    ///
    /// `images` may be pre-populated by the caller (e.g. file-path images
    /// loaded by `export.rs`).  Data-URI images found in `Block::Image` nodes
    /// are decoded and appended automatically.
    pub fn from_tiptap(
        root: TiptapNode,
        styles: HashMap<String, StyleDefinition>,
        metadata: Metadata,
        fonts: Vec<FontAsset>,
        mut images: Vec<ImageAsset>,
    ) -> Self {
        let mut sections = Vec::new();
        let mut current_blocks: Vec<Block> = Vec::new();
        let mut section_counter = 1usize;

        // Convert TiptapNode tree to flat Block list
        let blocks = match root {
            TiptapNode::Doc { content } => content
                .into_iter()
                .filter_map(conversion::tiptap_node_to_block)
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };

        // Decode any data-URI images found in the block tree
        let mut data_uri_images = conversion::extract_images_from_blocks(&blocks);
        images.append(&mut data_uri_images);

        // Split at PageBreak or style-level page breaks
        for block in blocks {
            let mut break_before = false;
            let mut break_after = false;

            let style_name = match &block {
                Block::Paragraph { style_name, .. } => style_name.as_deref(),
                Block::Heading { style_name, .. } => style_name.as_deref(),
                _ => None,
            };

            if let Some(name) = style_name {
                if let Some(style) = styles.get(name) {
                    if style
                        .attributes
                        .get("fo:break-before")
                        .map(|s| s.as_str())
                        == Some("page")
                    {
                        break_before = true;
                    }
                    if style
                        .attributes
                        .get("fo:break-after")
                        .map(|s| s.as_str())
                        == Some("page")
                    {
                        break_after = true;
                    }
                }
            }

            if break_before && !current_blocks.is_empty() {
                sections.push(ContentSection {
                    id: format!("section-{}", section_counter),
                    title: extract_section_title(&current_blocks),
                    blocks: current_blocks.clone(),
                });
                current_blocks.clear();
                section_counter += 1;
            }

            if matches!(block, Block::PageBreak) {
                if !current_blocks.is_empty() {
                    sections.push(ContentSection {
                        id: format!("section-{}", section_counter),
                        title: extract_section_title(&current_blocks),
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
                    title: extract_section_title(&current_blocks),
                    blocks: current_blocks.clone(),
                });
                current_blocks.clear();
                section_counter += 1;
            }
        }

        if !current_blocks.is_empty() {
            sections.push(ContentSection {
                id: format!("section-{}", section_counter),
                title: extract_section_title(&current_blocks),
                blocks: current_blocks,
            });
        }

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
            images,
        }
    }

    /// Render a content section to a self-contained XHTML document string.
    pub fn section_to_xhtml(&self, section: &ContentSection) -> String {
        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str("<!DOCTYPE html>\n");
        out.push_str(
            "<html xmlns=\"http://www.w3.org/1999/xhtml\" \
             xmlns:epub=\"http://www.idpf.org/2007/ops\">\n",
        );
        out.push_str("<head>\n");
        // G5: escape section title in <title>
        out.push_str(&format!(
            "  <title>{}</title>\n",
            html::escape_xml(section.title.as_deref().unwrap_or("Section"))
        ));
        out.push_str(
            "  <link rel=\"stylesheet\" type=\"text/css\" href=\"../Styles/styles.css\"/>\n",
        );
        out.push_str("</head>\n");
        out.push_str("<body>\n");
        for block in &section.blocks {
            out.push_str(&html::block_to_html(block, &self.styles, &self.images));
        }
        out.push_str("</body>\n");
        out.push_str("</html>\n");
        out
    }

    /// Generate the OPF 3.0 package document.
    pub fn to_package_opf(&self) -> String {
        opf::generate_package_opf(&self.metadata, &self.sections, &self.fonts, &self.images)
    }

    /// Generate the EPUB 3 Navigation Document (nav.xhtml).
    pub fn to_nav_xhtml(&self) -> String {
        nav::generate_nav_xhtml(&self.sections)
    }

    /// Generate the CSS stylesheet.
    pub fn to_css(&self) -> String {
        css::generate_css(&self.styles, &self.fonts)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the text content of the first heading in `blocks`, falling back to
/// `None` (so the caller can use a generic "Section N" label).
fn extract_section_title(blocks: &[Block]) -> Option<String> {
    for block in blocks {
        if let Block::Heading { content, .. } = block {
            let text: String = content
                .iter()
                .filter_map(|inline| {
                    if let common_core::Inline::Text { text, .. } = inline {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}
