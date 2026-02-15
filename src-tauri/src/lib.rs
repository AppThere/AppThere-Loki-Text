use odt_logic::{Document, TiptapNode, StyleDefinition, TiptapResponse, Metadata};
use std::collections::HashMap;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn sync_document(tiptap_json: TiptapNode, styles: HashMap<String, StyleDefinition>, metadata: Metadata) -> Document {
    Document::from_tiptap(tiptap_json, styles, metadata)
}

#[tauri::command]
async fn save_document(path: String, tiptap_json: TiptapNode, styles: HashMap<String, StyleDefinition>, metadata: Metadata) -> Result<(), String> {
    let doc = Document::from_tiptap(tiptap_json, styles, metadata);
    let fodt = doc.to_fodt().map_err(|e| e.to_string())?;
    std::fs::write(&path, fodt).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn open_document(path: String) -> Result<TiptapResponse, String> {
    let xml = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc = Document::from_fodt(&xml)?;
    Ok(TiptapResponse {
        content: doc.to_tiptap(),
        styles: doc.styles,
        metadata: doc.metadata,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            sync_document,
            save_document,
            open_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
