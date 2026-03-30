/**
 * Returns true when running inside an Android WebView.
 *
 * Tauri's Android WebView always includes "Android" in the user-agent string.
 * This is the lightest-weight platform check available without importing
 * @tauri-apps/plugin-os (not installed in this project).
 *
 * Used to gate Android-only ContentResolver plugin commands so they are never
 * invoked on desktop where the uriPermission plugin does not exist.
 */
export function isAndroid(): boolean {
    return /android/i.test(navigator.userAgent);
}
