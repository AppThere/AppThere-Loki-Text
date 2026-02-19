mod commands;

use log::info;
use odt_logic::{Document, Metadata, StyleDefinition, TiptapNode};
use std::collections::HashMap;

// Type aliases for easier binding
pub struct AppState {}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn sync_document(
    tiptap_json: String,
    styles: HashMap<String, StyleDefinition>,
    metadata: Metadata,
) -> Result<Document, String> {
    info!("Synchronizing document...");
    let json_node: TiptapNode = serde_json::from_str(&tiptap_json).map_err(|e| e.to_string())?;
    Ok(Document::from_tiptap(json_node, styles, metadata))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("DEBUG: run() starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            sync_document,
            commands::fs::save_document,
            commands::fs::open_document,
            commands::export::save_epub
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
