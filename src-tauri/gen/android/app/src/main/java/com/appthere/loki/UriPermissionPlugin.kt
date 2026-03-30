package com.appthere.loki

import android.app.Activity
import android.content.Intent
import android.net.Uri
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

/**
 * Tauri plugin that provides Android content:// URI access via ContentResolver.
 *
 * Tauri's plugin-fs uses Rust's std::fs, which cannot open content:// URIs on
 * Android. This plugin provides three commands that bypass std::fs entirely:
 *
 *  - takePersistablePermission: persists SAF URI permissions across app restarts.
 *  - readUri: reads all bytes from a content:// URI via ContentResolver.
 *  - writeUri: writes a byte array to a content:// URI via ContentResolver,
 *              truncating any existing content.
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

    /**
     * Read all bytes from a content:// URI using Android's ContentResolver.
     *
     * Returns a JSObject `{ bytes: number[] }` where each element is an
     * unsigned byte value (0–255). The frontend converts this to a Uint8Array.
     */
    @Command
    fun readUri(invoke: Invoke) {
        val uri = invoke.getArgs().getString("uri")
        if (uri.isEmpty()) {
            invoke.reject("Missing uri parameter")
            return
        }
        try {
            val parsedUri = Uri.parse(uri)
            val inputStream = activity.contentResolver.openInputStream(parsedUri)
                ?: run { invoke.reject("Cannot open input stream for URI: $uri"); return }
            val rawBytes = inputStream.readBytes()
            inputStream.close()

            val arr = JSArray()
            for (b in rawBytes) {
                arr.put(b.toInt() and 0xFF)
            }
            val result = JSObject()
            result.put("bytes", arr)
            invoke.resolve(result)
        } catch (e: Exception) {
            invoke.reject(e.message ?: "Failed to read URI")
        }
    }

    /**
     * Write bytes to a content:// URI using Android's ContentResolver.
     *
     * Expects `{ uri: string, bytes: number[] }`. Opens the URI in write-truncate
     * mode ("wt") so any existing content is replaced. The frontend converts a
     * Uint8Array to a plain number array before invoking this command.
     */
    @Command
    fun writeUri(invoke: Invoke) {
        val uri = invoke.getArgs().getString("uri")
        if (uri.isEmpty()) {
            invoke.reject("Missing uri parameter")
            return
        }
        try {
            val bytesJson = invoke.getArgs().getJSONArray("bytes")
            val bytes = ByteArray(bytesJson.length())
            for (i in 0 until bytesJson.length()) {
                bytes[i] = (bytesJson.getInt(i) and 0xFF).toByte()
            }

            val parsedUri = Uri.parse(uri)
            val outputStream = activity.contentResolver.openOutputStream(parsedUri, "wt")
                ?: run { invoke.reject("Cannot open output stream for URI: $uri"); return }
            outputStream.write(bytes)
            outputStream.flush()
            outputStream.close()
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "Failed to write URI")
        }
    }
}
