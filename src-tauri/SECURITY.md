# Security Configuration

This document describes the deliberate security configuration decisions made
for AppThere Loki Text. It covers the Tauri filesystem capability scope and
the Content Security Policy applied to the frontend webview.

---

## Filesystem Scope

**File:** `src-tauri/capabilities/default.json`

```json
{
  "identifier": "fs:scope",
  "allow": ["$HOME/**", "$APPLOCALDATA/**"]
}
```

### Allowed paths

| Entry | Resolves to | Purpose |
|---|---|---|
| `$HOME/**` | User home directory tree | Open/save dialogs for ODT, FODT, and EPUB files |
| `$APPLOCALDATA/**` | Platform app-data directory | Session auto-save files managed by `SessionManager` |

### Rationale

The previous scope included a bare `"**"` entry that matched every absolute
path on the filesystem, including system directories, other users' home
directories, and application bundles. This violated the principle of least
privilege.

All legitimate file operations in the application fall within one of the two
remaining scopes:

- **`useFileOperations.ts`** — open and save dialogs are presented by the
  Tauri dialog plugin. The user selects the path, which is always under
  `$HOME` on all supported platforms (macOS, Linux, Windows).
- **`sessionStorage.ts`** — session auto-save writes go to the platform
  app-data directory, resolved by Tauri as `$APPLOCALDATA`.
- **`src-tauri/src/commands/fs.rs`** — reads and writes user-supplied paths
  that originate from dialog selections, which are constrained to `$HOME`.
- **`src-tauri/src/commands/export.rs`** — EPUB export destination is
  user-selected via a save dialog, which is constrained to `$HOME`.

No operation legitimately requires access outside these two trees. The bare
`"**"` wildcard has been removed permanently.

---

## Content Security Policy

**File:** `src-tauri/tauri.conf.json`

```
default-src 'self' tauri: ipc: asset: https://asset.localhost;
script-src 'self' 'unsafe-inline' asset: https://asset.localhost;
style-src 'self' 'unsafe-inline' asset: https://asset.localhost;
font-src 'self' asset: https://asset.localhost data:;
img-src 'self' asset: https://asset.localhost data: blob:;
connect-src 'self' ipc: https://ipc.localhost
```

### Per-directive explanation

| Directive | Value | Reason |
|---|---|---|
| `default-src` | `'self' tauri: ipc: asset: https://asset.localhost` | Fallback for any resource type not covered by a more specific directive. Covers Tauri's internal schemes on all platforms. |
| `script-src` | `'self' 'unsafe-inline' asset: https://asset.localhost` | Allows scripts from the bundled app origin. `'unsafe-inline'` is required because Vite's build output injects inline script blocks (see Known Limitations). |
| `style-src` | `'self' 'unsafe-inline' asset: https://asset.localhost` | Allows styles from the bundled app origin. `'unsafe-inline'` is required because Tailwind CSS and Radix UI inject inline styles at runtime (see Known Limitations). |
| `font-src` | `'self' asset: https://asset.localhost data:` | Allows bundled fonts. `data:` covers any font loaded as a data URI by the build toolchain. |
| `img-src` | `'self' asset: https://asset.localhost data: blob:` | `data:` is required for the image embedding feature: images are stored as `data:image/…;base64,…` URIs and rendered directly in the editor. `blob:` covers object URLs the frontend may generate for user-selected image files. |
| `connect-src` | `'self' ipc: https://ipc.localhost` | Covers the Tauri IPC transport. `ipc:` is used on macOS and Linux; `https://ipc.localhost` is used on Windows. |

### What is explicitly excluded

- No `http:` or `https:` origins for external network resources — the app is
  fully local and must not load scripts, styles, fonts, or data from the
  internet.
- No `eval()` or `new Function()` — `'unsafe-eval'` is absent from
  `script-src`. Lexical does not require dynamic code evaluation.
- No `data:` in `script-src` — prevents script injection via data URIs.

---

## Known Limitations

### `'unsafe-inline'` in `script-src`

Vite's production bundle may emit inline `<script>` blocks depending on the
chunk-splitting configuration. Until a build audit confirms that the
production output contains no inline scripts, `'unsafe-inline'` must remain.

**Conditions for removal:** Audit the built `dist/` directory to verify no
`<script>` tags contain inline code. If confirmed, replace `'unsafe-inline'`
with a build-time `nonce-…` value (requires Vite plugin support) or remove
it entirely and rely solely on `'self'`.

### `'unsafe-inline'` in `style-src`

Tailwind CSS v3 and Radix UI primitives inject styles at runtime via
`style` attributes and `<style>` elements. Removing `'unsafe-inline'` from
`style-src` would require migrating to a CSS-in-JS solution that supports
CSP nonces, which is a significant architecture change.

**Conditions for removal:** Migrate all runtime style injection to
nonce-based or hash-based inline styles, or switch to a static CSS
extraction approach that eliminates inline styles entirely.
