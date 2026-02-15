use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
    pub styles: HashMap<String, StyleDefinition>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Block {
    Paragraph {
        #[serde(rename = "styleName")]
        style_name: Option<String>,
        #[serde(default)]
        attrs: Option<BlockAttrs>,
        content: Vec<Inline>,
    },
    Heading {
        level: u32,
        #[serde(rename = "styleName")]
        style_name: Option<String>,
        #[serde(default)]
        attrs: Option<BlockAttrs>,
        content: Vec<Inline>,
    },
    Image {
        src: String,
        alt: Option<String>,
        title: Option<String>,
    },
    BulletList {
        content: Vec<Block>,
    },
    OrderedList {
        content: Vec<Block>,
    },
    ListItem {
        content: Vec<Block>,
    },
    Blockquote {
        content: Vec<Block>,
    },
    Table {
        content: Vec<Block>,
    },
    TableRow {
        content: Vec<Block>,
    },
    TableHeader {
        #[serde(default)]
        attrs: Option<CellAttrs>,
        content: Vec<Block>,
    },
    TableCell {
        #[serde(default)]
        attrs: Option<CellAttrs>,
        content: Vec<Block>,
    },
    HorizontalRule,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct BlockAttrs {
    #[serde(rename = "textAlign")]
    pub text_align: Option<String>,
    pub indent: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CellAttrs {
    pub colspan: Option<u32>,
    pub rowspan: Option<u32>,
    pub colwidth: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Inline {
    Text {
        text: String,
        #[serde(rename = "styleName")]
        style_name: Option<String>,
        #[serde(default)]
        marks: Vec<TiptapMark>,
    },
    LineBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TiptapMark {
    NamedSpanStyle { attrs: TiptapAttrs },
    Bold,
    Italic,
    Underline,
    Strike,
    Superscript,
    Subscript,
    Link { attrs: LinkAttrs },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LinkAttrs {
    pub href: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyleDefinition {
    pub name: String,
    pub family: StyleFamily,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StyleFamily {
    Paragraph,
    Text,
}

// Tiptap representation for bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TiptapNode {
    Doc {
        content: Vec<TiptapNode>,
    },
    Paragraph {
        attrs: Option<TiptapAttrs>,
        content: Option<Vec<TiptapNode>>,
    },
    Heading {
        attrs: Option<TiptapAttrs>,
        content: Option<Vec<TiptapNode>>,
    },
    Text {
        text: String,
        marks: Option<Vec<TiptapMark>>,
    },
    Image {
        attrs: ImageAttrs,
    },
    BulletList {
        content: Vec<TiptapNode>,
    },
    OrderedList {
        content: Vec<TiptapNode>,
    },
    ListItem {
        content: Vec<TiptapNode>,
    },
    Blockquote {
        content: Vec<TiptapNode>,
    },
    Table {
        content: Vec<TiptapNode>,
    },
    TableRow {
        content: Vec<TiptapNode>,
    },
    TableHeader {
        attrs: Option<CellAttrs>,
        content: Vec<TiptapNode>,
    },
    TableCell {
        attrs: Option<CellAttrs>,
        content: Vec<TiptapNode>,
    },
    HorizontalRule,
    HardBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Eq, Hash)]
pub struct TiptapAttrs {
    #[serde(rename = "styleName")]
    pub style_name: Option<String>,
    pub level: Option<u32>,
    #[serde(rename = "textAlign")]
    pub text_align: Option<String>,
    pub indent: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageAttrs {
    pub src: String,
    pub alt: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    #[serde(rename = "creationDate")]
    pub creation_date: Option<String>,
    pub generator: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TiptapResponse {
    pub content: TiptapNode,
    pub styles: HashMap<String, StyleDefinition>,
    pub metadata: Metadata,
}

impl Document {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            styles: HashMap::new(),
            metadata: Metadata::default(),
        }
    }

    pub fn from_fodt(xml: &str) -> Result<Self, String> {
        let doc = roxmltree::Document::parse(xml).map_err(|e| e.to_string())?;
        let root = doc.root_element();

        // Namespaces
        let ns_office = "urn:oasis:names:tc:opendocument:xmlns:office:1.0";
        let ns_text = "urn:oasis:names:tc:opendocument:xmlns:text:1.0";
        let ns_style = "urn:oasis:names:tc:opendocument:xmlns:style:1.0";
        let ns_fo = "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0";
        let ns_dc = "http://purl.org/dc/elements/1.1/";
        let ns_meta = "urn:oasis:names:tc:opendocument:xmlns:meta:1.0";
        let ns_draw = "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0";
        let ns_table = "urn:oasis:names:tc:opendocument:xmlns:table:1.0";
        let ns_xlink = "http://www.w3.org/1999/xlink";

        // Parse Metadata
        let mut metadata = Metadata::default();
        if let Some(meta_node) = root
            .children()
            .find(|n| n.has_tag_name((ns_office, "meta")))
        {
            for child in meta_node.children() {
                if child.has_tag_name((ns_dc, "title")) {
                    metadata.title = child.text().map(|s| s.to_string());
                } else if child.has_tag_name((ns_dc, "description")) {
                    metadata.description = child.text().map(|s| s.to_string());
                } else if child.has_tag_name((ns_dc, "subject")) {
                    metadata.subject = child.text().map(|s| s.to_string());
                } else if child.has_tag_name((ns_dc, "creator")) {
                    metadata.creator = child.text().map(|s| s.to_string());
                } else if child.has_tag_name((ns_meta, "creation-date")) {
                    metadata.creation_date = child.text().map(|s| s.to_string());
                } else if child.has_tag_name((ns_meta, "generator")) {
                    metadata.generator = child.text().map(|s| s.to_string());
                }
            }
        }

        // Parse styles
        let mut style_map = HashMap::new();
        style_map.insert(
            "Strong".to_string(),
            ("text".to_string(), vec![TiptapMark::Bold]),
        );
        style_map.insert(
            "Emphasis".to_string(),
            ("text".to_string(), vec![TiptapMark::Italic]),
        );

        let mut style_definitions = HashMap::new();
        let style_nodes = root
            .children()
            .filter(|n| {
                n.has_tag_name((ns_office, "styles"))
                    || n.has_tag_name((ns_office, "automatic-styles"))
            })
            .flat_map(|n| n.children())
            .filter(|n| n.has_tag_name((ns_style, "style")));

        for style_node in style_nodes {
            if let Some(name) = style_node.attribute((ns_style, "name")) {
                let family_str = style_node.attribute((ns_style, "family")).unwrap_or("");
                let family = match family_str {
                    "paragraph" => StyleFamily::Paragraph,
                    "text" => StyleFamily::Text,
                    _ => StyleFamily::Text,
                };

                let mut attrs = HashMap::new();
                let mut marks = Vec::new();

                for prop_node in style_node.children() {
                    if prop_node.has_tag_name((ns_style, "text-properties")) {
                        if prop_node.attribute((ns_fo, "font-weight")) == Some("bold") {
                            marks.push(TiptapMark::Bold);
                        }
                        if prop_node.attribute((ns_fo, "font-style")) == Some("italic") {
                            marks.push(TiptapMark::Italic);
                        }
                        if let Some(u) = prop_node.attribute((ns_style, "text-underline-style")) {
                            if u != "none" {
                                marks.push(TiptapMark::Underline);
                            }
                        }
                        // ... more marks logic
                    }
                    for attr in prop_node.attributes() {
                        attrs.insert(attr.name().to_string(), attr.value().to_string());
                    }
                }
                style_definitions.insert(
                    name.to_string(),
                    StyleDefinition {
                        name: name.to_string(),
                        family,
                        attributes: attrs,
                    },
                );
                style_map.insert(name.to_string(), (family_str.to_string(), marks));
            }
        }

        let office_text = root
            .children()
            .find(|n| n.has_tag_name((ns_office, "body")))
            .and_then(|n| n.children().find(|c| c.has_tag_name((ns_office, "text"))))
            .ok_or("Could not find office:text")?;

        // Helper functions
        fn parse_inlines(
            node: roxmltree::Node,
            ns_text: &str,
            ns_xlink: &str,
            style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
        ) -> Vec<Inline> {
            let mut inlines = Vec::new();
            for child in node.children() {
                if child.is_text() {
                    inlines.push(Inline::Text {
                        text: child.text().unwrap_or("").to_string(),
                        style_name: None,
                        marks: Vec::new(),
                    });
                } else if child.has_tag_name((ns_text, "span")) {
                    let s_name = child.attribute((ns_text, "style-name"));
                    let marks = s_name
                        .and_then(|s| style_map.get(s))
                        .map(|(_, m)| m.clone())
                        .unwrap_or_default();
                    inlines.push(Inline::Text {
                        text: child.text().unwrap_or("").to_string(),
                        style_name: s_name.map(|s| s.to_string()),
                        marks,
                    });
                } else if child.has_tag_name((ns_text, "line-break")) {
                    inlines.push(Inline::LineBreak);
                } else if child.has_tag_name((ns_text, "a")) {
                    let href = child
                        .attribute((ns_xlink, "href"))
                        .unwrap_or("")
                        .to_string();
                    let inner = parse_inlines(child, ns_text, ns_xlink, style_map);
                    for mut i in inner {
                        if let Inline::Text { ref mut marks, .. } = i {
                            marks.push(TiptapMark::Link {
                                attrs: LinkAttrs {
                                    href: href.clone(),
                                    target: Some("_blank".to_string()),
                                },
                            });
                        }
                        inlines.push(i);
                    }
                }
            }
            inlines
        }

        fn parse_blocks(
            node: roxmltree::Node,
            ns_text: &str,
            ns_table: &str,
            ns_draw: &str,
            ns_xlink: &str,
            style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
        ) -> Vec<Block> {
            let mut blocks = Vec::new();
            for child in node.children() {
                if child.has_tag_name((ns_text, "p")) {
                    if let Some(frame) = child
                        .children()
                        .find(|n| n.has_tag_name((ns_draw, "frame")))
                    {
                        if let Some(img) = frame
                            .children()
                            .find(|n| n.has_tag_name((ns_draw, "image")))
                        {
                            let href = img.attribute((ns_xlink, "href")).unwrap_or("").to_string();
                            blocks.push(Block::Image {
                                src: href,
                                alt: None,
                                title: None,
                            });
                            continue;
                        }
                    }
                    let style_name = child
                        .attribute((ns_text, "style-name"))
                        .map(|s| s.to_string());
                    let content = parse_inlines(child, ns_text, ns_xlink, style_map);
                    blocks.push(Block::Paragraph {
                        style_name,
                        attrs: None,
                        content,
                    });
                } else if child.has_tag_name((ns_text, "h")) {
                    let level = child
                        .attribute((ns_text, "outline-level"))
                        .and_then(|l| l.parse().ok())
                        .unwrap_or(1);
                    let content = parse_inlines(child, ns_text, ns_xlink, style_map);
                    blocks.push(Block::Heading {
                        level,
                        style_name: None,
                        attrs: None,
                        content,
                    });
                } else if child.has_tag_name((ns_text, "list")) {
                    let mut items = Vec::new();
                    for item in child
                        .children()
                        .filter(|n| n.has_tag_name((ns_text, "list-item")))
                    {
                        let content =
                            parse_blocks(item, ns_text, ns_table, ns_draw, ns_xlink, style_map);
                        items.push(Block::ListItem { content });
                    }
                    blocks.push(Block::BulletList { content: items });
                } else if child.has_tag_name((ns_table, "table")) {
                    let mut rows = Vec::new();
                    for row in child
                        .children()
                        .filter(|n| n.has_tag_name((ns_table, "table-row")))
                    {
                        let mut cells = Vec::new();
                        for cell in row.children() {
                            if cell.has_tag_name((ns_table, "table-cell")) {
                                let content = parse_blocks(
                                    cell, ns_text, ns_table, ns_draw, ns_xlink, style_map,
                                );
                                cells.push(Block::TableCell {
                                    attrs: None,
                                    content,
                                });
                            }
                        }
                        rows.push(Block::TableRow { content: cells });
                    }
                    blocks.push(Block::Table { content: rows });
                }
            }
            blocks
        }

        let blocks = parse_blocks(
            office_text,
            ns_text,
            ns_table,
            ns_draw,
            ns_xlink,
            &style_map,
        );
        Ok(Document {
            blocks,
            styles: style_definitions,
            metadata,
        })
    }

    pub fn to_fodt(&self) -> Result<String, String> {
        use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
        use quick_xml::Writer;
        use std::io::Cursor;

        let mut writer = Writer::new(Cursor::new(Vec::new()));
        writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| e.to_string())?;

        let mut document = BytesStart::new("office:document");
        document.push_attribute((
            "xmlns:office",
            "urn:oasis:names:tc:opendocument:xmlns:office:1.0",
        ));
        document.push_attribute((
            "xmlns:text",
            "urn:oasis:names:tc:opendocument:xmlns:text:1.0",
        ));
        document.push_attribute((
            "xmlns:style",
            "urn:oasis:names:tc:opendocument:xmlns:style:1.0",
        ));
        document.push_attribute((
            "xmlns:fo",
            "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0",
        ));
        document.push_attribute(("xmlns:xlink", "http://www.w3.org/1999/xlink"));
        document.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
        document.push_attribute((
            "xmlns:meta",
            "urn:oasis:names:tc:opendocument:xmlns:meta:1.0",
        ));
        document.push_attribute((
            "xmlns:draw",
            "urn:oasis:names:tc:opendocument:xmlns:drawing:1.0",
        ));
        document.push_attribute((
            "xmlns:table",
            "urn:oasis:names:tc:opendocument:xmlns:table:1.0",
        ));
        document.push_attribute(("office:mimetype", "application/vnd.oasis.opendocument.text"));
        document.push_attribute(("office:version", "1.3"));
        writer
            .write_event(Event::Start(document))
            .map_err(|e| e.to_string())?;

        writer
            .write_event(Event::Start(BytesStart::new("office:body")))
            .map_err(|e| e.to_string())?;
        writer
            .write_event(Event::Start(BytesStart::new("office:text")))
            .map_err(|e| e.to_string())?;

        fn write_blocks(
            blocks: &Vec<Block>,
            writer: &mut Writer<Cursor<Vec<u8>>>,
        ) -> Result<(), String> {
            for block in blocks {
                match block {
                    Block::Paragraph {
                        style_name,
                        content,
                        ..
                    } => {
                        let mut p = BytesStart::new("text:p");
                        if let Some(s) = style_name {
                            p.push_attribute(("text:style-name", s.as_str()));
                        }
                        writer
                            .write_event(Event::Start(p))
                            .map_err(|e| e.to_string())?;
                        write_inlines(content, writer)?;
                        writer
                            .write_event(Event::End(BytesEnd::new("text:p")))
                            .map_err(|e| e.to_string())?;
                    }
                    Block::Heading {
                        level,
                        style_name,
                        content,
                        ..
                    } => {
                        let mut h = BytesStart::new("text:h");
                        if let Some(s) = style_name {
                            h.push_attribute(("text:style-name", s.as_str()));
                        }
                        h.push_attribute(("text:outline-level", level.to_string().as_str()));
                        writer
                            .write_event(Event::Start(h))
                            .map_err(|e| e.to_string())?;
                        write_inlines(content, writer)?;
                        writer
                            .write_event(Event::End(BytesEnd::new("text:h")))
                            .map_err(|e| e.to_string())?;
                    }
                    Block::BulletList { content } => {
                        writer
                            .write_event(Event::Start(BytesStart::new("text:list")))
                            .map_err(|e| e.to_string())?;
                        for item in content {
                            write_blocks(&vec![item.clone()], writer)?;
                        }
                        writer
                            .write_event(Event::End(BytesEnd::new("text:list")))
                            .map_err(|e| e.to_string())?;
                    }
                    Block::OrderedList { content } => {
                        writer
                            .write_event(Event::Start(BytesStart::new("text:list")))
                            .map_err(|e| e.to_string())?;
                        for item in content {
                            write_blocks(&vec![item.clone()], writer)?;
                        }
                        writer
                            .write_event(Event::End(BytesEnd::new("text:list")))
                            .map_err(|e| e.to_string())?;
                    }
                    Block::ListItem { content } => {
                        writer
                            .write_event(Event::Start(BytesStart::new("text:list-item")))
                            .map_err(|e| e.to_string())?;
                        write_blocks(content, writer)?;
                        writer
                            .write_event(Event::End(BytesEnd::new("text:list-item")))
                            .map_err(|e| e.to_string())?;
                    }
                    Block::Table { content } => {
                        writer
                            .write_event(Event::Start(BytesStart::new("table:table")))
                            .map_err(|e| e.to_string())?;
                        write_blocks(content, writer)?;
                        writer
                            .write_event(Event::End(BytesEnd::new("table:table")))
                            .map_err(|e| e.to_string())?;
                    }
                    Block::TableRow { content } => {
                        writer
                            .write_event(Event::Start(BytesStart::new("table:table-row")))
                            .map_err(|e| e.to_string())?;
                        write_blocks(content, writer)?;
                        writer
                            .write_event(Event::End(BytesEnd::new("table:table-row")))
                            .map_err(|e| e.to_string())?;
                    }
                    Block::TableCell { content, .. } => {
                        writer
                            .write_event(Event::Start(BytesStart::new("table:table-cell")))
                            .map_err(|e| e.to_string())?;
                        write_blocks(content, writer)?;
                        writer
                            .write_event(Event::End(BytesEnd::new("table:table-cell")))
                            .map_err(|e| e.to_string())?;
                    }
                    Block::Image { src, .. } => {
                        let mut frame = BytesStart::new("draw:frame");
                        frame.push_attribute(("draw:name", "Image"));
                        writer
                            .write_event(Event::Start(frame))
                            .map_err(|e| e.to_string())?;
                        let mut img = BytesStart::new("draw:image");
                        img.push_attribute(("xlink:href", src.as_str()));
                        img.push_attribute(("xlink:type", "simple"));
                        img.push_attribute(("xlink:show", "embed"));
                        img.push_attribute(("xlink:actuate", "onLoad"));
                        writer
                            .write_event(Event::Empty(img))
                            .map_err(|e| e.to_string())?;
                        writer
                            .write_event(Event::End(BytesEnd::new("draw:frame")))
                            .map_err(|e| e.to_string())?;
                    }
                    _ => {}
                }
            }
            Ok(())
        }

        fn write_inlines(
            inlines: &Vec<Inline>,
            writer: &mut Writer<Cursor<Vec<u8>>>,
        ) -> Result<(), String> {
            for inline in inlines {
                match inline {
                    Inline::Text {
                        text, style_name, ..
                    } => {
                        if let Some(s) = style_name {
                            let mut span = BytesStart::new("text:span");
                            span.push_attribute(("text:style-name", s.as_str()));
                            writer
                                .write_event(Event::Start(span))
                                .map_err(|e| e.to_string())?;
                            writer
                                .write_event(Event::Text(BytesText::new(text)))
                                .map_err(|e| e.to_string())?;
                            writer
                                .write_event(Event::End(BytesEnd::new("text:span")))
                                .map_err(|e| e.to_string())?;
                        } else {
                            writer
                                .write_event(Event::Text(BytesText::new(text)))
                                .map_err(|e| e.to_string())?;
                        }
                    }
                    Inline::LineBreak => {
                        writer
                            .write_event(Event::Empty(BytesStart::new("text:line-break")))
                            .map_err(|e| e.to_string())?;
                    }
                }
            }
            Ok(())
        }

        write_blocks(&self.blocks, &mut writer)?;

        writer
            .write_event(Event::End(BytesEnd::new("office:text")))
            .map_err(|e| e.to_string())?;
        writer
            .write_event(Event::End(BytesEnd::new("office:body")))
            .map_err(|e| e.to_string())?;
        writer
            .write_event(Event::End(BytesEnd::new("office:document")))
            .map_err(|e| e.to_string())?;

        let result = writer.into_inner().into_inner();
        String::from_utf8(result).map_err(|e| e.to_string())
    }

    pub fn from_tiptap(
        root: TiptapNode,
        styles: HashMap<String, StyleDefinition>,
        metadata: Metadata,
    ) -> Self {
        let mut blocks = Vec::new();
        match root {
            TiptapNode::Doc { content } => {
                for node in content {
                    if let Some(block) = Self::tiptap_node_to_block(node) {
                        blocks.push(block);
                    }
                }
            }
            _ => {}
        }
        Document {
            blocks,
            styles,
            metadata,
        }
    }

    fn tiptap_node_to_block(node: TiptapNode) -> Option<Block> {
        match node {
            TiptapNode::Paragraph { attrs, content } => {
                let style_name = attrs.as_ref().and_then(|a| a.style_name.clone());
                let block_attrs = attrs.map(|a| BlockAttrs {
                    text_align: a.text_align,
                    indent: a.indent,
                });
                let inlines = Self::tiptap_content_to_inlines(content.unwrap_or_default());
                Some(Block::Paragraph {
                    style_name,
                    attrs: block_attrs.into(),
                    content: inlines,
                })
            }
            TiptapNode::Heading { attrs, content } => {
                let style_name = attrs.as_ref().and_then(|a| a.style_name.clone());
                let level = attrs.as_ref().and_then(|a| a.level).unwrap_or(1);
                let block_attrs = attrs.map(|a| BlockAttrs {
                    text_align: a.text_align,
                    indent: a.indent,
                });
                let inlines = Self::tiptap_content_to_inlines(content.unwrap_or_default());
                Some(Block::Heading {
                    level,
                    style_name,
                    attrs: block_attrs.into(),
                    content: inlines,
                })
            }
            TiptapNode::Image { attrs } => Some(Block::Image {
                src: attrs.src,
                alt: attrs.alt,
                title: attrs.title,
            }),
            TiptapNode::BulletList { content } => {
                let items = content
                    .into_iter()
                    .filter_map(Self::tiptap_node_to_block)
                    .collect();
                Some(Block::BulletList { content: items })
            }
            TiptapNode::OrderedList { content } => {
                let items = content
                    .into_iter()
                    .filter_map(Self::tiptap_node_to_block)
                    .collect();
                Some(Block::OrderedList { content: items })
            }
            TiptapNode::ListItem { content } => {
                let items = content
                    .into_iter()
                    .filter_map(Self::tiptap_node_to_block)
                    .collect();
                Some(Block::ListItem { content: items })
            }
            TiptapNode::Blockquote { content } => {
                let items = content
                    .into_iter()
                    .filter_map(Self::tiptap_node_to_block)
                    .collect();
                Some(Block::Blockquote { content: items })
            }
            TiptapNode::Table { content } => {
                let items = content
                    .into_iter()
                    .filter_map(Self::tiptap_node_to_block)
                    .collect();
                Some(Block::Table { content: items })
            }
            TiptapNode::TableRow { content } => {
                let items = content
                    .into_iter()
                    .filter_map(Self::tiptap_node_to_block)
                    .collect();
                Some(Block::TableRow { content: items })
            }
            TiptapNode::TableHeader { attrs, content } => {
                let items = content
                    .into_iter()
                    .filter_map(Self::tiptap_node_to_block)
                    .collect();
                Some(Block::TableHeader {
                    attrs,
                    content: items,
                })
            }
            TiptapNode::TableCell { attrs, content } => {
                let items = content
                    .into_iter()
                    .filter_map(Self::tiptap_node_to_block)
                    .collect();
                Some(Block::TableCell {
                    attrs,
                    content: items,
                })
            }
            TiptapNode::HorizontalRule => Some(Block::HorizontalRule),
            _ => None,
        }
    }

    fn tiptap_content_to_inlines(nodes: Vec<TiptapNode>) -> Vec<Inline> {
        let mut inlines = Vec::new();
        for node in nodes {
            match node {
                TiptapNode::Text { text, marks } => {
                    inlines.push(Inline::Text {
                        text,
                        style_name: None,
                        marks: marks.unwrap_or_default(),
                    });
                }
                TiptapNode::HardBreak => {
                    inlines.push(Inline::LineBreak);
                }
                _ => {}
            }
        }
        inlines
    }

    pub fn to_tiptap(&self) -> TiptapNode {
        let content = self
            .blocks
            .iter()
            .map(|b| Self::block_to_tiptap(b))
            .collect();
        TiptapNode::Doc { content }
    }

    fn block_to_tiptap(block: &Block) -> TiptapNode {
        match block {
            Block::Paragraph {
                style_name,
                attrs,
                content,
            } => {
                let t_attrs = Some(TiptapAttrs {
                    style_name: style_name.clone(),
                    text_align: attrs.as_ref().and_then(|a| a.text_align.clone()),
                    indent: attrs.as_ref().and_then(|a| a.indent),
                    level: None,
                });
                TiptapNode::Paragraph {
                    attrs: t_attrs,
                    content: Some(Self::inlines_to_tiptap(content)),
                }
            }
            Block::Heading {
                level,
                style_name,
                attrs,
                content,
            } => {
                let t_attrs = Some(TiptapAttrs {
                    style_name: style_name.clone(),
                    text_align: attrs.as_ref().and_then(|a| a.text_align.clone()),
                    indent: attrs.as_ref().and_then(|a| a.indent),
                    level: Some(*level),
                });
                TiptapNode::Heading {
                    attrs: t_attrs,
                    content: Some(Self::inlines_to_tiptap(content)),
                }
            }
            Block::Image { src, alt, title } => TiptapNode::Image {
                attrs: ImageAttrs {
                    src: src.clone(),
                    alt: alt.clone(),
                    title: title.clone(),
                },
            },
            Block::BulletList { content } => TiptapNode::BulletList {
                content: content.iter().map(Self::block_to_tiptap).collect(),
            },
            Block::OrderedList { content } => TiptapNode::OrderedList {
                content: content.iter().map(Self::block_to_tiptap).collect(),
            },
            Block::ListItem { content } => TiptapNode::ListItem {
                content: content.iter().map(Self::block_to_tiptap).collect(),
            },
            Block::Blockquote { content } => TiptapNode::Blockquote {
                content: content.iter().map(Self::block_to_tiptap).collect(),
            },
            Block::Table { content } => TiptapNode::Table {
                content: content.iter().map(Self::block_to_tiptap).collect(),
            },
            Block::TableRow { content } => TiptapNode::TableRow {
                content: content.iter().map(Self::block_to_tiptap).collect(),
            },
            Block::TableHeader { attrs, content } => TiptapNode::TableHeader {
                attrs: attrs.clone(),
                content: content.iter().map(Self::block_to_tiptap).collect(),
            },
            Block::TableCell { attrs, content } => TiptapNode::TableCell {
                attrs: attrs.clone(),
                content: content.iter().map(Self::block_to_tiptap).collect(),
            },
            Block::HorizontalRule => TiptapNode::HorizontalRule,
        }
    }

    fn inlines_to_tiptap(inlines: &Vec<Inline>) -> Vec<TiptapNode> {
        let mut nodes = Vec::new();
        for inline in inlines {
            match inline {
                Inline::Text {
                    text,
                    style_name: _,
                    marks,
                } => {
                    nodes.push(TiptapNode::Text {
                        text: text.clone(),
                        marks: Some(marks.clone()),
                    });
                }
                Inline::LineBreak => {
                    nodes.push(TiptapNode::HardBreak);
                }
            }
        }
        nodes
    }
}
