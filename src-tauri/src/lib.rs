mod commands;
mod fonts;

use commands::android::{FilePickerHandle, UriPermissionHandle};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    eprintln!("DEBUG: run() starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri::plugin::Builder::<_, ()>::new("uriPermission")
                .setup(|app, api| {
                    #[cfg(target_os = "android")]
                    {
                        let handle = api
                            .register_android_plugin("com.appthere.loki", "UriPermissionPlugin")?;
                        app.manage(UriPermissionHandle(handle));
                    }
                    Ok(())
                })
                .build(),
        )
        .plugin(
            tauri::plugin::Builder::<_, ()>::new("filePicker")
                .setup(|app, api| {
                    #[cfg(target_os = "android")]
                    {
                        let handle =
                            api.register_android_plugin("com.appthere.loki", "FilePickerPlugin")?;
                        app.manage(FilePickerHandle(handle));
                    }
                    Ok(())
                })
                .build(),
        )
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
                        &MenuItem::with_id(
                            app,
                            "menu-export-pdf",
                            "Export to PDF/X...",
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
                        &PredefinedMenuItem::separator(app)?,
                        &MenuItem::with_id(app, "menu-find", "Find…", true, Some("CmdOrCtrl+F"))?,
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
            commands::locale::get_system_locale,
            commands::fs::save_document,
            commands::fs::open_document,
            commands::export::save_epub,
            commands::session::serialize_document,
            commands::session::deserialize_document,
            commands::vector::open_vector_document,
            commands::vector::save_vector_document,
            commands::vector::new_vector_document,
            commands::vector::serialize_vector_document,
            commands::vector::deserialize_vector_document,
            commands::vector::batch_convert_colours,
            commands::vector::convert_document_colour_mode,
            commands::vector::get_output_intent_profiles,
            commands::vector::preview_colour_conversion,
            commands::vector::search_pantone,
            commands::pdf::validate_pdf_x_conformance,
            commands::pdf::export_pdf_x,
            commands::pdf::validate_text_pdf_x_conformance,
            commands::pdf::export_text_pdf_x,
            commands::android::pick_file_to_open,
            commands::android::take_persistable_uri_permission
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
