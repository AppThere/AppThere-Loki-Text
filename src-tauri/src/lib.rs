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
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
                use tauri::Emitter;

                // App Menu (macOS only really, but Tauri handles default app info)
                let app_menu = Submenu::with_items(
                    app,
                    "Loki", // Use the app name instead of "App"
                    true,
                    &[
                        &PredefinedMenuItem::about(app, None, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::services(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::hide(app, None)?,
                        &PredefinedMenuItem::hide_others(app, None)?,
                        &PredefinedMenuItem::show_all(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::quit(app, None)?,
                    ],
                )?;

                // File Menu
                let file_menu = Submenu::with_items(
                    app,
                    "File",
                    true,
                    &[
                        &MenuItem::with_id(app, "menu-new", "New", true, Some("CmdOrCtrl+N"))?,
                        &MenuItem::with_id(app, "menu-open", "Open...", true, Some("CmdOrCtrl+O"))?,
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(app, "menu-save", "Save", true, Some("CmdOrCtrl+S"))?,
                        &MenuItem::with_id(
                            app,
                            "menu-save-as",
                            "Save As...",
                            true,
                            Some("CmdOrCtrl+Shift+S"),
                        )?,
                        &MenuItem::with_id(
                            app,
                            "menu-export-epub",
                            "Export to EPUB...",
                            true,
                            None::<&str>,
                        )?,
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(
                            app,
                            "menu-print",
                            "Print...",
                            true,
                            Some("CmdOrCtrl+P"),
                        )?,
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(app, "menu-close", "Close", true, Some("CmdOrCtrl+W"))?,
                    ],
                )?;

                // Edit Menu
                let edit_menu = Submenu::with_items(
                    app,
                    "Edit",
                    true,
                    &[
                        &MenuItem::with_id(app, "menu-undo", "Undo", true, Some("CmdOrCtrl+Z"))?,
                        &MenuItem::with_id(
                            app,
                            "menu-redo",
                            "Redo",
                            true,
                            Some("CmdOrCtrl+Shift+Z"),
                        )?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::cut(app, None)?,
                        &PredefinedMenuItem::copy(app, None)?,
                        &PredefinedMenuItem::paste(app, None)?,
                        &PredefinedMenuItem::separator(app)?,
                        &PredefinedMenuItem::select_all(app, None)?,
                    ],
                )?;

                let menu = Menu::with_items(app, &[&app_menu, &file_menu, &edit_menu])?;

                app.set_menu(menu)?;

                app.on_menu_event(move |app, event| {
                    let id = event.id.as_ref();
                    // Emit all menu events to the frontend
                    let _ = app.emit(id, ());
                });
            }

            Ok(())
        })
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
