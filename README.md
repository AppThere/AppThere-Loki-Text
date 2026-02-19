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
