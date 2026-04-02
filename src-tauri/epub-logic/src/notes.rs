// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

use serde::{Deserialize, Serialize};

use crate::html::escape_xml;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Footnote or endnote content mirroring the TypeScript `FootnoteContent` interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FootnoteContent {
    pub id: String,
    pub serialised_state: String,
    pub created_at: u64,
}

/// Placement mode for notes: bottom-of-page footnote or end-of-document endnote.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FootnotePlacement {
    Footnote,
    Endnote,
}

// ---------------------------------------------------------------------------
// notes.xhtml generation
// ---------------------------------------------------------------------------

/// Map from footnote UUID → (1-based seq, source section id).
pub type FootnoteSeqMap = std::collections::HashMap<String, (usize, String)>;

/// Build the `footnote_seq_map` by walking all section blocks in document order.
///
/// Must be called after `from_tiptap` has built the sections.
pub fn build_seq_map(sections: &[crate::ContentSection]) -> FootnoteSeqMap {
    let mut map = FootnoteSeqMap::new();
    let mut seq = 1usize;
    for section in sections {
        collect_seq_from_blocks(&section.blocks, &section.id, &mut seq, &mut map);
    }
    map
}

fn collect_seq_from_blocks(
    blocks: &[common_core::Block],
    section_id: &str,
    seq: &mut usize,
    map: &mut FootnoteSeqMap,
) {
    use common_core::{Block, Inline};
    for block in blocks {
        match block {
            Block::Paragraph { content, .. } | Block::Heading { content, .. } => {
                for inline in content {
                    if let Inline::FootnoteRef { id } = inline {
                        map.insert(id.clone(), (*seq, section_id.to_string()));
                        *seq += 1;
                    }
                }
            }
            Block::BulletList { content }
            | Block::OrderedList { content }
            | Block::ListItem { content }
            | Block::Blockquote { content }
            | Block::Table { content }
            | Block::TableRow { content } => {
                collect_seq_from_blocks(content, section_id, seq, map);
            }
            Block::TableHeader { content, .. } | Block::TableCell { content, .. } => {
                collect_seq_from_blocks(content, section_id, seq, map);
            }
            _ => {}
        }
    }
}

/// Generate a complete `notes.xhtml` document from footnote content.
pub fn generate_notes_xhtml(
    footnotes: &[FootnoteContent],
    seq_map: &FootnoteSeqMap,
    placement: &FootnotePlacement,
    doc_lang: &str,
    notes_heading: &str,
) -> String {
    let (section_type, aside_type) = match placement {
        FootnotePlacement::Footnote => ("footnotes", "footnote"),
        FootnotePlacement::Endnote => ("endnotes", "endnote"),
    };

    let mut asides = String::new();
    // Emit in seq order
    let mut entries: Vec<(&FootnoteContent, usize, String)> = footnotes
        .iter()
        .filter_map(|fn_c| {
            seq_map
                .get(&fn_c.id)
                .map(|(seq, section_id)| (fn_c, *seq, section_id.clone()))
        })
        .collect();
    entries.sort_by_key(|(_, seq, _)| *seq);

    for (fn_c, seq, section_id) in &entries {
        let content_html = plain_text_from_serialised(&fn_c.serialised_state);
        asides.push_str(&format!(
            "      <aside epub:type=\"{aside_type}\" role=\"doc-{aside_type}\" id=\"fn-{seq}\">\n\
                     <p>\n\
                       <a href=\"{section_id}.xhtml#fnref-{seq}\" epub:type=\"backlink\" \
             role=\"doc-backlink\">{seq}.</a>\n\
                       {content_html}\n\
                     </p>\n\
                   </aside>\n",
        ));
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE html>\n\
         <html xmlns=\"http://www.w3.org/1999/xhtml\"\n\
               xmlns:epub=\"http://www.idpf.org/2007/ops\"\n\
               lang=\"{lang}\">\n\
           <body epub:type=\"backmatter\">\n\
             <section epub:type=\"{section_type}\" role=\"doc-{section_type}\">\n\
               <h2>{heading}</h2>\n\
         {asides}\
             </section>\n\
           </body>\n\
         </html>\n",
        lang = escape_xml(doc_lang),
        section_type = section_type,
        heading = escape_xml(notes_heading),
        asides = asides,
    )
}

/// Extract plain text from a minimal Lexical JSON state string.
/// Falls back to empty string on parse failure.
fn plain_text_from_serialised(json: &str) -> String {
    // Simple extraction: find all "text":"..." values
    let mut result = String::new();
    let mut search = json;
    while let Some(pos) = search.find("\"text\":\"") {
        let rest = &search[pos + 8..];
        if let Some(end) = rest.find('"') {
            result.push_str(escape_xml(&rest[..end]).as_str());
        }
        search = &search[pos + 8..];
    }
    result
}
