package com.appthere.loki

import android.app.Activity
import android.content.Intent
import android.net.Uri
import androidx.activity.result.ActivityResult
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

/**
 * File picker that uses ACTION_OPEN_DOCUMENT so content:// URI permissions can
 * be made persistent across app restarts.
 *
 * The standard Tauri dialog plugin uses ACTION_GET_CONTENT, which returns a
 * temporary URI that cannot be persisted — so files opened that way always fail
 * to reopen from the Recents list after the app process is killed.
 *
 * ACTION_OPEN_DOCUMENT includes FLAG_GRANT_PERSISTABLE_URI_PERMISSION.
 * We call ContentResolver.takePersistableUriPermission() inside the activity
 * result callback (the only window where the grant is valid) before returning
 * the URI to the frontend.
 */
@TauriPlugin
class FilePickerPlugin(private val activity: Activity) : Plugin(activity) {

    @Command
    fun openFile(invoke: Invoke) {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(
                Intent.EXTRA_MIME_TYPES,
                arrayOf(
                    "application/vnd.oasis.opendocument.text",
                    "application/vnd.oasis.opendocument.text-flat-xml",
                )
            )
        }
        startActivityForResult(invoke, intent, "openFileResult")
    }

    @ActivityCallback
    fun openFileResult(invoke: Invoke, result: ActivityResult) {
        when (result.resultCode) {
            Activity.RESULT_OK -> {
                val uri: Uri = result.data?.data ?: run {
                    invoke.reject("No file selected")
                    return
                }

                // Persist the permission immediately while the grant is still
                // valid. This allows readFile / writeFile to work in future
                // sessions after the app process is killed.
                try {
                    val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or
                            Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                    activity.contentResolver.takePersistableUriPermission(uri, flags)
                } catch (_: SecurityException) {
                    // Non-fatal: file is still usable within this session.
                }

                val res = JSObject()
                res.put("uri", uri.toString())
                invoke.resolve(res)
            }

            Activity.RESULT_CANCELED -> invoke.reject("cancelled")
            else -> invoke.reject("Failed to open file picker")
        }
    }
}
