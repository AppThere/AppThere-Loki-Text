package com.appthere.loki

import android.content.Intent
import android.os.Bundle
import androidx.core.view.WindowCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    setTheme(R.style.Theme_appthere_loki)
    super.onCreate(savedInstanceState)
    WindowCompat.setDecorFitsSystemWindows(window, true)
    pluginManager.load(null, "uriPermission", UriPermissionPlugin(this), "{}")
    pluginManager.load(null, "filePicker", FilePickerPlugin(this), "{}")
  }

  /**
   * Automatically persist SAF content:// URI permissions whenever a file-picker
   * activity returns a result.
   *
   * The Tauri IPC router only knows about plugins registered on the Rust side via
   * .plugin(), so the JavaScript plugin:uriPermission|takePersistablePermission
   * call is silently rejected. Doing the work here in onActivityResult ensures
   * permissions are always persisted after the file picker closes, regardless of
   * which plugin or activity launched it.
   */
  override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
    super.onActivityResult(requestCode, resultCode, data)
    if (resultCode != RESULT_OK || data == null) return

    val uri = data.data ?: return
    if (uri.scheme != "content") return

    try {
      val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or
          Intent.FLAG_GRANT_WRITE_URI_PERMISSION
      contentResolver.takePersistableUriPermission(uri, flags)
    } catch (_: SecurityException) {
      // Non-fatal: the URI may not have been offered with
      // FLAG_GRANT_PERSISTABLE_URI_PERMISSION by the provider.
    }
  }
}
