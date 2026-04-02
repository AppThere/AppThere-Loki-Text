// Copyright 2024 AppThere
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Android bridge commands for file picking and URI permission persistence.
//!
//! JavaScript cannot call `plugin:name|command` IPC directly because Tauri's
//! ACL system denies unknown plugins by default. These regular Tauri commands
//! (registered via `invoke_handler!`) use `PluginHandle::run_mobile_plugin_async`
//! which goes through JNI directly, bypassing the ACL entirely.

use tauri::Runtime;

/// State holder for the FilePickerPlugin handle (Android only).
pub struct FilePickerHandle<R: Runtime>(pub tauri::plugin::PluginHandle<R>);

/// State holder for the UriPermissionPlugin handle (Android only).
pub struct UriPermissionHandle<R: Runtime>(pub tauri::plugin::PluginHandle<R>);

#[derive(serde::Deserialize)]
struct FilePickerResult {
    uri: String,
}

#[derive(serde::Serialize)]
struct TakePersistableArgs {
    uri: String,
}

/// Open a file picker using ACTION_OPEN_DOCUMENT (Android).
///
/// Returns the selected `content://` URI as a string. Permissions are
/// persisted inside the Kotlin activity result callback so the file can
/// be reopened from Recents after the app process is killed.
#[cfg(target_os = "android")]
#[tauri::command]
pub async fn pick_file_to_open(
    state: tauri::State<'_, FilePickerHandle<tauri::Wry>>,
) -> Result<String, String> {
    state
        .0
        .run_mobile_plugin_async::<FilePickerResult>("openFile", ())
        .await
        .map(|r| r.uri)
        .map_err(|e| e.to_string())
}

/// No-op on non-Android platforms; the desktop file dialog is used instead.
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn pick_file_to_open() -> Result<String, String> {
    Err("pick_file_to_open is only available on Android".to_string())
}

/// Persist a `content://` URI permission across app restarts (Android).
///
/// Must be called while the temporary SAF grant is still active (i.e. during
/// the same session in which the URI was obtained from a file picker).
#[cfg(target_os = "android")]
#[tauri::command]
pub async fn take_persistable_uri_permission(
    uri: String,
    state: tauri::State<'_, UriPermissionHandle<tauri::Wry>>,
) -> Result<(), String> {
    state
        .0
        .run_mobile_plugin_async::<()>("takePersistablePermission", TakePersistableArgs { uri })
        .await
        .map_err(|e| e.to_string())
}

/// No-op on non-Android platforms.
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn take_persistable_uri_permission(_uri: String) -> Result<(), String> {
    Ok(())
}
