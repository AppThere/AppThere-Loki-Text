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
        content: Vec<Inline>,
    },
    Heading {
        level: u32,
        #[serde(rename = "styleName")]
        style_name: Option<String>,
        content: Vec<Inline>,
    },
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
    HardBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TiptapAttrs {
    #[serde(rename = "styleName")]
    pub style_name: Option<String>,
    pub level: Option<u32>,
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

// Keep TiptapMark definition above for reuse

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
        // Default semantic styles
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
                    _ => StyleFamily::Text, // Default or other types as Text for now
                };

                let mut attrs = HashMap::new();
                let mut marks = Vec::new();

                // Collect properties
                for prop_node in style_node.children() {
                    if prop_node.has_tag_name((ns_style, "text-properties"))
                        || prop_node.has_tag_name((ns_style, "paragraph-properties"))
                    {
                        for attr in prop_node.attributes() {
                            let prefix = match attr.namespace() {
                                Some(ns) if ns == ns_fo => "fo",
                                Some(ns) if ns == ns_style => "style",
                                Some(ns) if ns == ns_text => "text",
                                _ => "",
                            };
                            let key = if prefix.is_empty() {
                                attr.name().to_string()
                            } else {
                                format!("{}:{}", prefix, attr.name())
                            };
                            attrs.insert(key, attr.value().to_string());
                        }

                        // Specific mark mapping for text-properties
                        if prop_node.has_tag_name((ns_style, "text-properties")) {
                            if prop_node.attribute((ns_fo, "font-weight")) == Some("bold") {
                                marks.push(TiptapMark::Bold);
                            }
                            if prop_node.attribute((ns_fo, "font-style")) == Some("italic") {
                                marks.push(TiptapMark::Italic);
                            }
                            if let Some(u) = prop_node.attribute((ns_style, "text-underline-style"))
                            {
                                if u != "none" {
                                    marks.push(TiptapMark::Underline);
                                }
                            }
                            if let Some(s) =
                                prop_node.attribute((ns_style, "text-line-through-style"))
                            {
                                if s != "none" {
                                    marks.push(TiptapMark::Strike);
                                }
                            }
                            if let Some(v) = prop_node.attribute((ns_style, "text-position")) {
                                if v.contains("super") {
                                    marks.push(TiptapMark::Superscript);
                                } else if v.contains("sub") {
                                    marks.push(TiptapMark::Subscript);
                                }
                            }
                        }
                    }
                }

                if family == StyleFamily::Text {
                    if (name == "Strong" || name.starts_with("T"))
                        && !marks.contains(&TiptapMark::Bold)
                        && attrs.get("fo:font-weight") == Some(&"bold".to_string())
                    {
                        // Covered by font-weight check
                    }
                    if name == "Strong" && !marks.contains(&TiptapMark::Bold) {
                        marks.push(TiptapMark::Bold);
                    }
                    if name == "Emphasis" && !marks.contains(&TiptapMark::Italic) {
                        marks.push(TiptapMark::Italic);
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

        // Find office:body/office:text
        let office_text = root
            .children()
            .find(|n| n.has_tag_name((ns_office, "body")))
            .and_then(|n| n.children().find(|c| c.has_tag_name((ns_office, "text"))))
            .ok_or("Could not find office:text")?;

        let mut blocks = Vec::new();
        for node in office_text.children() {
            if !node.has_tag_name((ns_text, "p")) && !node.has_tag_name((ns_text, "h")) {
                continue;
            }
            let style_name = node
                .attribute((ns_text, "style-name"))
                .map(|s| s.to_string());
            let content;

            fn parse_inlines(
                node: roxmltree::Node,
                ns_text: &str,
                style_map: &HashMap<String, (String, Vec<TiptapMark>)>,
            ) -> Vec<Inline> {
                let mut inlines = Vec::new();
                let ns_xlink = "http://www.w3.org/1999/xlink";

                for child in node.children() {
                    if child.is_text() {
                        inlines.push(Inline::Text {
                            text: child.text().unwrap_or("").to_string(),
                            style_name: None,
                            marks: Vec::new(),
                        });
                    } else if child.has_tag_name((ns_text, "span")) {
                        let span_style = child.attribute((ns_text, "style-name"));
                        let marks = span_style
                            .and_then(|s| style_map.get(s))
                            .map(|(_, m)| m.clone())
                            .unwrap_or_default();
                        inlines.push(Inline::Text {
                            text: child.text().unwrap_or("").to_string(),
                            style_name: span_style.map(|s| s.to_string()),
                            marks,
                        });
                    } else if child.has_tag_name((ns_text, "a")) {
                        let href = child
                            .attribute((ns_xlink, "href"))
                            .unwrap_or("")
                            .to_string();
                        let marks = vec![TiptapMark::Link {
                            attrs: LinkAttrs {
                                href,
                                target: Some("_blank".to_string()),
                            },
                        }];
                        // Links can have nested spans or text
                        let inner = parse_inlines(child, ns_text, style_map);
                        for mut inline in inner {
                            if let Inline::Text {
                                marks: ref mut m, ..
                            } = inline
                            {
                                m.extend(marks.clone());
                            }
                            inlines.push(inline);
                        }
                    } else if child.has_tag_name((ns_text, "line-break")) {
                        inlines.push(Inline::LineBreak);
                    }
                }
                inlines
            }

            content = parse_inlines(node, ns_text, &style_map);

            if node.has_tag_name((ns_text, "p")) {
                blocks.push(Block::Paragraph {
                    style_name,
                    content,
                });
            } else if node.has_tag_name((ns_text, "h")) {
                let level = node
                    .attribute((ns_text, "outline-level"))
                    .and_then(|l| l.parse::<u32>().ok())
                    .unwrap_or(1);
                blocks.push(Block::Heading {
                    level,
                    style_name,
                    content,
                });
            }
        }

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

        // Header
        writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| e.to_string())?;

        // office:document
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
        document.push_attribute(("office:mimetype", "application/vnd.oasis.opendocument.text"));
        document.push_attribute(("office:version", "1.3"));
        writer
            .write_event(Event::Start(document))
            .map_err(|e| e.to_string())?;

        // office:meta
        writer
            .write_event(Event::Start(BytesStart::new("office:meta")))
            .map_err(|e| e.to_string())?;

        if let Some(title) = &self.metadata.title {
            writer
                .write_event(Event::Start(BytesStart::new("dc:title")))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(title)))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(BytesEnd::new("dc:title")))
                .map_err(|e| e.to_string())?;
        }
        if let Some(desc) = &self.metadata.description {
            writer
                .write_event(Event::Start(BytesStart::new("dc:description")))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(desc)))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(BytesEnd::new("dc:description")))
                .map_err(|e| e.to_string())?;
        }
        if let Some(subject) = &self.metadata.subject {
            writer
                .write_event(Event::Start(BytesStart::new("dc:subject")))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(subject)))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(BytesEnd::new("dc:subject")))
                .map_err(|e| e.to_string())?;
        }
        if let Some(creator) = &self.metadata.creator {
            writer
                .write_event(Event::Start(BytesStart::new("dc:creator")))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(creator)))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(BytesEnd::new("dc:creator")))
                .map_err(|e| e.to_string())?;
        }
        if let Some(date) = &self.metadata.creation_date {
            writer
                .write_event(Event::Start(BytesStart::new("meta:creation-date")))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(date)))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(BytesEnd::new("meta:creation-date")))
                .map_err(|e| e.to_string())?;
        }
        if let Some(generator) = &self.metadata.generator {
            writer
                .write_event(Event::Start(BytesStart::new("meta:generator")))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::Text(BytesText::new(generator)))
                .map_err(|e| e.to_string())?;
            writer
                .write_event(Event::End(BytesEnd::new("meta:generator")))
                .map_err(|e| e.to_string())?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("office:meta")))
            .map_err(|e| e.to_string())?;

        // office:font-face-decls (optional but good practice)

        // office:styles (Common styles)
        writer
            .write_event(Event::Start(BytesStart::new("office:styles")))
            .map_err(|e| e.to_string())?;

        // Write named styles from self.styles
        let mut sorted_styles: Vec<_> = self.styles.values().collect();
        sorted_styles.sort_by_key(|s| &s.name);

        for style in sorted_styles {
            let mut s = BytesStart::new("style:style");
            s.push_attribute(("style:name", style.name.as_str()));
            let family_str = match style.family {
                StyleFamily::Paragraph => "paragraph",
                StyleFamily::Text => "text",
            };
            s.push_attribute(("style:family", family_str));
            writer
                .write_event(Event::Start(s))
                .map_err(|e| e.to_string())?;

            let mut text_props = BytesStart::new("style:text-properties");
            let mut para_props = BytesStart::new("style:paragraph-properties");
            let mut has_text = false;
            let mut has_para = false;

            for (key, value) in &style.attributes {
                if key.starts_with("fo:") || key.starts_with("style:") {
                    if key.contains("text") || key.contains("font") || key.contains("color") {
                        text_props.push_attribute((key.as_str(), value.as_str()));
                        has_text = true;
                    } else {
                        para_props.push_attribute((key.as_str(), value.as_str()));
                        has_para = true;
                    }
                }
            }

            if has_text {
                writer
                    .write_event(Event::Empty(text_props))
                    .map_err(|e| e.to_string())?;
            }
            if has_para {
                writer
                    .write_event(Event::Empty(para_props))
                    .map_err(|e| e.to_string())?;
            }

            writer
                .write_event(Event::End(BytesEnd::new("style:style")))
                .map_err(|e| e.to_string())?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("office:styles")))
            .map_err(|e| e.to_string())?;

        // office:automatic-styles
        writer
            .write_event(Event::Start(BytesStart::new("office:automatic-styles")))
            .map_err(|e| e.to_string())?;

        let mut next_style_id = 1;
        let mut mark_to_style = HashMap::new();

        for block in &self.blocks {
            let content = match block {
                Block::Paragraph { content, .. } => content,
                Block::Heading { content, .. } => content,
            };
            for inline in content {
                if let Inline::Text { marks, .. } = inline {
                    if !marks.is_empty() {
                        let filtered_marks: Vec<TiptapMark> = marks
                            .iter()
                            .filter(|m| !matches!(m, TiptapMark::Link { .. }))
                            .cloned()
                            .collect();
                        if !filtered_marks.is_empty()
                            && !mark_to_style.contains_key(&filtered_marks)
                        {
                            let style_name = format!("T{}", next_style_id);
                            next_style_id += 1;

                            let mut style = BytesStart::new("style:style");
                            style.push_attribute(("style:name", style_name.as_str()));
                            style.push_attribute(("style:family", "text"));
                            writer
                                .write_event(Event::Start(style))
                                .map_err(|e| e.to_string())?;

                            let mut props = BytesStart::new("style:text-properties");
                            let mut has_props = false;
                            for mark in &filtered_marks {
                                match mark {
                                    TiptapMark::Bold => {
                                        // Usually handled by "Strong" style, but for auto-styles we keep it
                                        props.push_attribute(("fo:font-weight", "bold"));
                                        has_props = true;
                                    }
                                    TiptapMark::Italic => {
                                        props.push_attribute(("fo:font-style", "italic"));
                                        has_props = true;
                                    }
                                    TiptapMark::Underline => {
                                        props.push_attribute((
                                            "style:text-underline-style",
                                            "solid",
                                        ));
                                        props
                                            .push_attribute(("style:text-underline-width", "auto"));
                                        props.push_attribute((
                                            "style:text-underline-color",
                                            "font-color",
                                        ));
                                        has_props = true;
                                    }
                                    TiptapMark::Strike => {
                                        props.push_attribute((
                                            "style:text-line-through-style",
                                            "solid",
                                        ));
                                        has_props = true;
                                    }
                                    TiptapMark::Superscript => {
                                        props.push_attribute(("style:text-position", "super 58%"));
                                        has_props = true;
                                    }
                                    TiptapMark::Subscript => {
                                        props.push_attribute(("style:text-position", "sub 58%"));
                                        has_props = true;
                                    }
                                    _ => {}
                                }
                            }
                            if has_props {
                                writer
                                    .write_event(Event::Empty(props))
                                    .map_err(|e| e.to_string())?;
                            }

                            writer
                                .write_event(Event::End(BytesEnd::new("style:style")))
                                .map_err(|e| e.to_string())?;
                            mark_to_style.insert(filtered_marks, style_name);
                        }
                    }
                }
            }
        }

        writer
            .write_event(Event::End(BytesEnd::new("office:automatic-styles")))
            .map_err(|e| e.to_string())?;

        // office:body
        writer
            .write_event(Event::Start(BytesStart::new("office:body")))
            .map_err(|e| e.to_string())?;

        // office:text
        writer
            .write_event(Event::Start(BytesStart::new("office:text")))
            .map_err(|e| e.to_string())?;

        for block in &self.blocks {
            match block {
                Block::Paragraph {
                    style_name,
                    content,
                } => {
                    let mut p = BytesStart::new("text:p");
                    if let Some(s) = style_name {
                        p.push_attribute(("text:style-name", s.as_str()));
                    }
                    writer
                        .write_event(Event::Start(p))
                        .map_err(|e| e.to_string())?;

                    for inline in content {
                        match inline {
                            Inline::Text {
                                text,
                                style_name,
                                marks,
                            } => {
                                let link = marks.iter().find_map(|m| {
                                    if let TiptapMark::Link { attrs } = m {
                                        Some(attrs)
                                    } else {
                                        None
                                    }
                                });
                                if let Some(l) = link {
                                    let mut a = BytesStart::new("text:a");
                                    a.push_attribute(("xlink:type", "simple"));
                                    a.push_attribute(("xlink:href", l.href.as_str()));
                                    writer
                                        .write_event(Event::Start(a))
                                        .map_err(|e| e.to_string())?;
                                }

                                let filtered_marks: Vec<TiptapMark> = marks
                                    .iter()
                                    .filter(|m| !matches!(m, TiptapMark::Link { .. }))
                                    .cloned()
                                    .collect();
                                let mut active_style = style_name.clone();

                                // Semantic Style Mapping: Prioritize "Strong" and "Emphasis" for single marks
                                if filtered_marks.len() == 1 {
                                    if filtered_marks.contains(&TiptapMark::Bold) {
                                        active_style = Some("Strong".to_string());
                                    } else if filtered_marks.contains(&TiptapMark::Italic) {
                                        active_style = Some("Emphasis".to_string());
                                    }
                                }

                                if active_style.is_none() {
                                    if let Some(auto_style) = mark_to_style.get(&filtered_marks) {
                                        active_style = Some(auto_style.clone());
                                    }
                                }

                                if let Some(s) = active_style {
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

                                if link.is_some() {
                                    writer
                                        .write_event(Event::End(BytesEnd::new("text:a")))
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

                    writer
                        .write_event(Event::End(BytesEnd::new("text:p")))
                        .map_err(|e| e.to_string())?;
                }
                Block::Heading {
                    level,
                    style_name,
                    content,
                } => {
                    let mut h = BytesStart::new("text:h");
                    if let Some(s) = style_name {
                        h.push_attribute(("text:style-name", s.as_str()));
                    }
                    h.push_attribute(("text:outline-level", level.to_string().as_str()));
                    writer
                        .write_event(Event::Start(h))
                        .map_err(|e| e.to_string())?;

                    for inline in content {
                        match inline {
                            Inline::Text {
                                text,
                                style_name,
                                marks,
                            } => {
                                let link = marks.iter().find_map(|m| {
                                    if let TiptapMark::Link { attrs } = m {
                                        Some(attrs)
                                    } else {
                                        None
                                    }
                                });
                                if let Some(l) = link {
                                    let mut a = BytesStart::new("text:a");
                                    a.push_attribute(("xlink:type", "simple"));
                                    a.push_attribute(("xlink:href", l.href.as_str()));
                                    writer
                                        .write_event(Event::Start(a))
                                        .map_err(|e| e.to_string())?;
                                }

                                let filtered_marks: Vec<TiptapMark> = marks
                                    .iter()
                                    .filter(|m| !matches!(m, TiptapMark::Link { .. }))
                                    .cloned()
                                    .collect();
                                let mut active_style = style_name.clone();

                                // Semantic Style Mapping: Prioritize "Strong" and "Emphasis" for single marks
                                if filtered_marks.len() == 1 {
                                    if filtered_marks.contains(&TiptapMark::Bold) {
                                        active_style = Some("Strong".to_string());
                                    } else if filtered_marks.contains(&TiptapMark::Italic) {
                                        active_style = Some("Emphasis".to_string());
                                    }
                                }

                                if active_style.is_none() {
                                    if let Some(auto_style) = mark_to_style.get(&filtered_marks) {
                                        active_style = Some(auto_style.clone());
                                    }
                                }

                                if let Some(s) = active_style {
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

                                if link.is_some() {
                                    writer
                                        .write_event(Event::End(BytesEnd::new("text:a")))
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

                    writer
                        .write_event(Event::End(BytesEnd::new("text:h")))
                        .map_err(|e| e.to_string())?;
                }
            }
        }

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
    pub fn to_tiptap(&self) -> TiptapNode {
        let mut content = Vec::new();
        for block in &self.blocks {
            match block {
                Block::Paragraph {
                    style_name,
                    content: inline_content,
                } => {
                    content.push(TiptapNode::Paragraph {
                        attrs: Some(TiptapAttrs {
                            style_name: style_name.clone(),
                            level: None,
                        }),
                        content: Some(Self::convert_to_tiptap_inlines(inline_content)),
                    });
                }
                Block::Heading {
                    level,
                    style_name,
                    content: inline_content,
                } => {
                    content.push(TiptapNode::Heading {
                        attrs: Some(TiptapAttrs {
                            style_name: style_name.clone(),
                            level: Some(*level),
                        }),
                        content: Some(Self::convert_to_tiptap_inlines(inline_content)),
                    });
                }
            }
        }
        TiptapNode::Doc { content }
    }

    fn convert_to_tiptap_inlines(inlines: &[Inline]) -> Vec<TiptapNode> {
        let mut nodes = Vec::new();
        for inline in inlines {
            match inline {
                Inline::Text {
                    text,
                    style_name,
                    marks,
                } => {
                    let mut marks = marks.clone();
                    if let Some(s) = style_name {
                        marks.push(TiptapMark::NamedSpanStyle {
                            attrs: TiptapAttrs {
                                style_name: Some(s.clone()),
                                level: None,
                            },
                        });
                    }
                    nodes.push(TiptapNode::Text {
                        text: text.clone(),
                        marks: if marks.is_empty() { None } else { Some(marks) },
                    });
                }
                Inline::LineBreak => {
                    nodes.push(TiptapNode::HardBreak);
                }
            }
        }
        nodes
    }

    pub fn from_tiptap(
        tiptap: TiptapNode,
        styles: HashMap<String, StyleDefinition>,
        metadata: Metadata,
    ) -> Self {
        let mut blocks = Vec::new();
        if let TiptapNode::Doc { content } = tiptap {
            for node in content {
                match node {
                    TiptapNode::Paragraph { attrs, content } => {
                        blocks.push(Block::Paragraph {
                            style_name: attrs.and_then(|a| a.style_name),
                            content: Self::convert_inlines(content.unwrap_or_default()),
                        });
                    }
                    TiptapNode::Heading { attrs, content } => {
                        let a = attrs.unwrap_or(TiptapAttrs {
                            style_name: None,
                            level: Some(1),
                        });
                        blocks.push(Block::Heading {
                            level: a.level.unwrap_or(1),
                            style_name: a.style_name,
                            content: Self::convert_inlines(content.unwrap_or_default()),
                        });
                    }
                    _ => {}
                }
            }
        }
        Document {
            blocks,
            styles,
            metadata,
        }
    }

    fn convert_inlines(nodes: Vec<TiptapNode>) -> Vec<Inline> {
        let mut inlines = Vec::new();
        for node in nodes {
            match node {
                TiptapNode::Text { text, marks } => {
                    let marks_val = marks.clone().unwrap_or_default();
                    let style_name = marks_val.iter().find_map(|mark| match mark {
                        TiptapMark::NamedSpanStyle { attrs } => attrs.style_name.clone(),
                        _ => None,
                    });
                    // Remove NamedSpanStyle from the marks list for Inline::Text
                    let filtered_marks: Vec<TiptapMark> = marks_val
                        .into_iter()
                        .filter(|m| !matches!(m, TiptapMark::NamedSpanStyle { .. }))
                        .collect();

                    inlines.push(Inline::Text {
                        text,
                        style_name,
                        marks: filtered_marks,
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
}

impl Inline {
    pub fn text(&self) -> &str {
        match self {
            Inline::Text { text, .. } => text,
            Inline::LineBreak => "",
        }
    }

    pub fn style_name(&self) -> Option<&str> {
        match self {
            Inline::Text { style_name, .. } => style_name.as_deref(),
            Inline::LineBreak => None,
        }
    }

    pub fn marks(&self) -> &[TiptapMark] {
        match self {
            Inline::Text { marks, .. } => marks,
            Inline::LineBreak => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fodt_parsing() {
        let fodt_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">
  <office:body>
    <office:text>
      <text:p text:style-name="Standard">Hello <text:span text:style-name="Emphasis">world</text:span></text:p>
    </office:text>
  </office:body>
</office:document>"#;

        let doc = Document::from_fodt(fodt_xml).unwrap();
        assert_eq!(doc.blocks.len(), 1);
        if let Block::Paragraph {
            style_name,
            content,
        } = &doc.blocks[0]
        {
            assert_eq!(style_name.as_deref(), Some("Standard"));
            assert_eq!(content.len(), 2);
            assert_eq!(content[0].text(), "Hello ");
            assert_eq!(content[1].text(), "world");
            assert_eq!(content[1].style_name(), Some("Emphasis"));
        }
    }

    #[test]
    fn test_tiptap_conversion() {
        let tiptap_json = r#"{
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "attrs": { "styleName": "Standard" },
                    "content": [
                        { "type": "text", "text": "Hello " },
                        {
                            "type": "text",
                            "text": "world",
                            "marks": [
                                {
                                    "type": "namedSpanStyle",
                                    "attrs": { "styleName": "Emphasis" }
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;

        let tiptap_node: TiptapNode = serde_json::from_str(tiptap_json).unwrap();
        let doc = Document::from_tiptap(tiptap_node, HashMap::new());

        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            Block::Paragraph {
                style_name,
                content,
            } => {
                assert_eq!(style_name.as_deref(), Some("Standard"));
                assert_eq!(content.len(), 2);
                let inline = &content[1];
                assert_eq!(inline.style_name(), Some("Emphasis"));
            }
            _ => panic!("Expected paragraph"),
        }
    }

    #[test]
    fn test_fodt_roundtrip() {
        let fodt_xml = r#"<?xml version="1.0" encoding="UTF-8"?><office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0">
            <office:styles>
                <style:style style:name="Emphasis" style:family="text">
                    <style:text-properties fo:font-style="italic"/>
                </style:style>
            </office:styles>
            <office:body>
                <office:text>
                    <text:p text:style-name="Standard">Hello <text:span text:style-name="Emphasis">world</text:span></text:p>
                </office:text>
            </office:body>
        </office:document>"#;

        let doc = Document::from_fodt(fodt_xml).unwrap();
        let generated_xml = doc.to_fodt().unwrap();

        // Parse the generated XML back and compare the Document structure
        // Note: We don't compare the raw XML strings because of potential formatting/declaration differences,
        // but the resulting Document model should be identical.
        let doc2 = Document::from_fodt(&generated_xml).unwrap();
        assert_eq!(doc.blocks.len(), doc2.blocks.len());
        assert_eq!(doc.blocks[0], doc2.blocks[0]);
    }

    #[test]
    fn test_line_break_roundtrip() {
        let fodt_xml = r#"<?xml version="1.0" encoding="UTF-8"?><office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"><office:body><office:text><text:p>Line 1<text:line-break/>Line 2</text:p></office:text></office:body></office:document>"#;

        let doc = Document::from_fodt(fodt_xml).unwrap();
        assert_eq!(doc.blocks.len(), 1);
        match &doc.blocks[0] {
            Block::Paragraph { content, .. } => {
                assert_eq!(content.len(), 3);
                assert!(matches!(content[1], Inline::LineBreak));
            }
            _ => panic!("Expected paragraph"),
        }

        let generated_xml = doc.to_fodt().unwrap();
        assert!(generated_xml.contains("<text:line-break/>"));

        let doc2 = Document::from_fodt(&generated_xml).unwrap();
        assert_eq!(doc, doc2);
    }

    #[test]
    fn test_formatting_roundtrip() {
        let mut doc = Document::new();
        doc.blocks.push(Block::Paragraph {
            style_name: Some("Standard".to_string()),
            content: vec![
                Inline::Text {
                    text: "Bold ".to_string(),
                    style_name: None,
                    marks: vec![TiptapMark::Bold],
                },
                Inline::Text {
                    text: "Italic ".to_string(),
                    style_name: None,
                    marks: vec![TiptapMark::Italic],
                },
                Inline::Text {
                    text: "Underline ".to_string(),
                    style_name: None,
                    marks: vec![TiptapMark::Underline],
                },
                Inline::Text {
                    text: "Strike".to_string(),
                    style_name: None,
                    marks: vec![TiptapMark::Strike],
                },
            ],
        });

        let fodt = doc.to_fodt().unwrap();

        // Basic check for styles
        assert!(fodt.contains("fo:font-weight=\"bold\""));
        assert!(fodt.contains("fo:font-style=\"italic\""));
        assert!(fodt.contains("style:text-underline-style=\"solid\""));
        assert!(fodt.contains("style:text-line-through-style=\"solid\""));

        let doc2 = Document::from_fodt(&fodt).unwrap();

        if let Block::Paragraph { content, .. } = &doc2.blocks[0] {
            assert_eq!(content[0].text(), "Bold ");
            assert!(content[0].marks().contains(&TiptapMark::Bold));

            assert_eq!(content[1].text(), "Italic ");
            assert!(content[1].marks().contains(&TiptapMark::Italic));

            assert_eq!(content[2].text(), "Underline ");
            assert!(content[2].marks().contains(&TiptapMark::Underline));

            assert_eq!(content[3].text(), "Strike");
            assert!(content[3].marks().contains(&TiptapMark::Strike));
        }
    }
}
