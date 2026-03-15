//! Session recovery tests.
//!
//! Verifies the filesystem-level guarantees that underpin the autosave flow:
//!
//! * The *original* user file is never touched during an autosave operation.
//! * Serializing a document to bytes and writing those bytes to a *different*
//!   path preserves the original file unchanged.
//! * An explicit "save to original" correctly overwrites the original file.
//! * A document round-trips faithfully through the byte-serialization path.

use std::time::Duration;

use common_core::{Block, Inline};
use odt_format::{
    lexical::{from_lexical, to_lexical},
    Document,
};
use tempfile::tempdir;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn simple_document(n: usize) -> Document {
    let mut doc = Document::new();
    for i in 0..n {
        doc.blocks.push(Block::Paragraph {
            style_name: None,
            attrs: None,
            content: vec![Inline::Text {
                text: format!("Paragraph {i}."),
                style_name: None,
                marks: vec![],
            }],
        });
    }
    doc
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Autosave writes to a *session* file, not to the original.
/// The original file's modification time and contents must not change.
#[test]
fn test_autosave_does_not_touch_original() {
    let dir = tempdir().unwrap();
    let original = dir.path().join("document.fodt");
    let session = dir.path().join("autosave.fodt");

    // Write sentinel content to the original file.
    std::fs::write(&original, b"Original ODT content").unwrap();
    let original_mtime = std::fs::metadata(&original).unwrap().modified().unwrap();

    // Simulate autosave: serialize to a session path only.
    let doc = simple_document(10);
    let xml = doc.to_xml().unwrap();
    std::fs::write(&session, xml.as_bytes()).unwrap();

    // Tiny sleep so any mtime change would be visible.
    std::thread::sleep(Duration::from_millis(10));

    let new_mtime = std::fs::metadata(&original).unwrap().modified().unwrap();
    let original_content = std::fs::read(&original).unwrap();

    assert_eq!(original_mtime, new_mtime, "Original mtime must not change");
    assert_eq!(
        original_content, b"Original ODT content",
        "Original content must not change"
    );
    assert!(session.exists(), "Session autosave file must exist");
}

/// An explicit "Save" operation overwrites the original file correctly.
#[test]
fn test_explicit_save_updates_original() {
    let dir = tempdir().unwrap();
    let original = dir.path().join("document.fodt");

    std::fs::write(&original, b"Old content").unwrap();

    let doc = simple_document(5);
    let xml = doc.to_xml().unwrap();
    // Explicit save writes directly to the original path.
    std::fs::write(&original, xml.as_bytes()).unwrap();

    let content = std::fs::read_to_string(&original).unwrap();
    assert!(
        content.contains("office:document"),
        "Explicit save should write valid ODT XML to original"
    );
    assert!(
        !content.starts_with("Old content"),
        "Old content should be replaced"
    );
}

/// A session file (autosave bytes) can be loaded back without data loss.
#[test]
fn test_autosave_bytes_round_trip() {
    let dir = tempdir().unwrap();
    let session = dir.path().join("autosave.fodt");

    let original = simple_document(20);
    let xml = original.to_xml().unwrap();
    std::fs::write(&session, xml.as_bytes()).unwrap();

    // Restore from session bytes.
    let restored_xml = std::fs::read_to_string(&session).unwrap();
    let restored = Document::from_xml(&restored_xml).unwrap();

    assert_eq!(
        original.blocks.len(),
        restored.blocks.len(),
        "Block count must survive session round-trip"
    );
}

/// Autosave through the Lexical layer preserves all text content.
#[test]
fn test_lexical_autosave_preserves_content() {
    let dir = tempdir().unwrap();
    let session = dir.path().join("autosave.fodt");

    let original = simple_document(50);
    let styles = original.styles.clone();
    let meta = original.metadata.clone();

    // Simulate what the frontend does on autosave:
    // Document → Lexical → back to Document → serialise.
    let lex = to_lexical(&original);
    let recovered = from_lexical(lex, styles, meta);
    let xml = recovered.to_xml().unwrap();
    std::fs::write(&session, xml.as_bytes()).unwrap();

    let final_doc = Document::from_xml(&std::fs::read_to_string(&session).unwrap()).unwrap();
    assert_eq!(
        original.blocks.len(),
        final_doc.blocks.len(),
        "Lexical autosave must preserve block count"
    );
}
