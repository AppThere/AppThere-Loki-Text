package com.appthere.loki

import android.app.Activity
import android.content.Intent
import android.net.Uri
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

/**
 * Tauri plugin that persists Android content:// URI permissions across app restarts.
 *
 * When a file is opened via the Storage Access Framework (SAF) file picker, Android
 * grants a temporary permission for that content URI. This permission is revoked when
 * the app process is killed. Calling takePersistablePermission saves the permission
 * durably so subsequent sessions can still read the file (e.g., from Recents).
 */
@TauriPlugin
class UriPermissionPlugin(private val activity: Activity) : Plugin(activity) {

    @Command
    fun takePersistablePermission(invoke: Invoke) {
        val uri = invoke.getArgs().getString("uri")
        if (uri.isEmpty()) {
            invoke.reject("Missing uri parameter")
            return
        }

        try {
            val parsedUri = Uri.parse(uri)
            val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or
                    Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            activity.contentResolver.takePersistableUriPermission(parsedUri, flags)
            invoke.resolve()
        } catch (e: SecurityException) {
            // The URI was not offered with FLAG_GRANT_PERSISTABLE_URI_PERMISSION,
            // or the temporary permission has already expired. Reject so the caller
            // can log the error, but this is non-fatal for the read-open path.
            invoke.reject(e.message ?: "SecurityException persisting URI permission")
        } catch (e: Exception) {
            invoke.reject(e.message ?: "Failed to persist URI permission")
        }
    }
}
