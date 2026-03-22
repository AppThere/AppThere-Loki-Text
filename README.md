# AppThere Loki Text

AppThere Loki Text is a distraction-free writing application designed for focused document creation. It specializes in the OpenDocument Text (.odt) format, providing a cross-platform experience across Desktop (Windows, macOS, Linux) and Android.

## Purpose

The main goal of Loki Text is to provide a clean, "zen" environment for writers who need to produce structured documents without the bloat of traditional office suites. It features:

- **ODT First**: Native support for OpenDocument Text, ensuring compatibility with LibreOffice and other standards-compliant editors.
- **Style-Centric Editing**: A streamlined approach to paragraph styling, including "Next Style" logic for efficient drafting.
- **Cross-Platform**: Built with Tauri to run natively on Desktop and Mobile.
- **Atkinson Hyperlegible Next**: Optimized for readability with high-legibility typography.

## Project Status

**Prototype / Early Development**
This project is currently in active development. Core ODT parsing, styling logic, and basic editor functionality are implemented, but features are subject to significant changes.

## Requirements

To build and run this project, you will need:

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) (v18+) and npm
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites) dependencies for your OS
- **For Android Build**:
  - [Android Studio](https://developer.android.com/studio)
  - Android SDK, NDK, and Java (JDK 17+)
  - Rust Android targets: `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`

## Build Instructions

### 1. Initial Setup

Clone the repository and install dependencies:

```bash
npm install
```

### 2. Desktop Development

Run the application in development mode:

```bash
npm run tauri dev
```

### 3. Android Development

To run on a connected Android device or emulator:

```bash
npm run tauri android dev
```

### 4. Build for Production

To create a production bundle:

**Desktop:**

```bash
npm run tauri build
```

**Android:**

```bash
npm run tauri android build
```

## License

AppThere Loki Text is released under the **Apache License 2.0**. See the LICENSE file (if available) for more details.

Copyright © 2026 AppThere. All rights reserved.

## Internationalization

Loki ships with a full i18n system built on [i18next](https://www.i18next.com/) and [react-i18next](https://react.i18next.com/). Locale JSON files are loaded at runtime via `i18next-http-backend`; the OS locale is detected through a Tauri command backed by the `sys-locale` Rust crate.

### Adding a new UI string

1. Add the key to `public/locales/en/common.json` (general UI) or `public/locales/en/editor.json` (canvas / document strings).
2. Use `t('your.key')` in the component (import `useTranslation` from `react-i18next`).
3. Run the locale validator to surface gaps in other locales:
   ```bash
   node scripts/validate-locales.js
   ```
4. Optionally fill in translations for other locales — missing keys fall through to the English fallback at runtime.

See `src/templates/Component.tsx` for a copy-paste starter component with full i18n wiring.

### Changing language at runtime

```ts
import { setLanguage } from '@/i18n';
setLanguage('fr'); // persists to localStorage and updates document.dir for RTL
```

A user preference stored in `localStorage` under the key `loki.language` always takes precedence over the OS locale.

### Running the locale validator

```bash
node scripts/validate-locales.js
```

Exits with code `0` if every non-English locale contains the same key structure as English; exits with code `1` and prints a per-locale report of missing keys otherwise.

### Supported locales (BCP 47)

| Tag | Language |
|-----|----------|
| `ar` | Arabic |
| `bg` | Bulgarian |
| `cs` | Czech |
| `da` | Danish |
| `de` | German |
| `el` | Greek |
| `en` | English *(reference)* |
| `es` | Spanish |
| `et` | Estonian |
| `fi` | Finnish |
| `fr` | French |
| `ga` | Irish |
| `hi` | Hindi |
| `hr` | Croatian |
| `hu` | Hungarian |
| `it` | Italian |
| `ja` | Japanese |
| `ko` | Korean |
| `lt` | Lithuanian |
| `lv` | Latvian |
| `mt` | Maltese |
| `nl` | Dutch |
| `pl` | Polish |
| `pt` | Portuguese |
| `ro` | Romanian |
| `sk` | Slovak |
| `sl` | Slovenian |
| `sv` | Swedish |
| `zh-Hans` | Chinese (Simplified) |

### RTL support

Arabic (`ar`) is the only RTL locale in the supported set. `setLanguage('ar')` automatically sets `document.documentElement.dir = 'rtl'`; all other locales set it to `'ltr'`. Components should rely on logical CSS properties (`margin-inline-start`, `padding-inline-end`, etc.) rather than physical `left`/`right` to respond correctly to the direction change.

### ESLint enforcement

The ESLint rule `i18next/no-literal-string` is configured at **error** level in `eslint.config.js` with `mode: 'jsx-only'`. Any literal string placed directly inside a JSX expression will fail linting, guiding developers to use `t()` instead. Pure integers, `SCREAMING_SNAKE_CASE` constants, whitespace-only strings, single characters, and version strings are excluded from the rule.
