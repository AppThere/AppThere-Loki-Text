# Loki — Developer Notes for Claude

## Android Safe Area (Edge-to-Edge UI)

Loki targets Android devices that use edge-to-edge rendering (status bar /
camera cut-out at the top, home indicator at the bottom). The Tauri WebView
extends under these system UI elements by default.

Two utility CSS classes handle this — **they must be applied to every view**:

| Class     | CSS                                           | When to use                                              |
|-----------|-----------------------------------------------|----------------------------------------------------------|
| `safe-pt` | `padding-top: env(safe-area-inset-top)`       | Any element that forms the **top edge** of a full-screen view   |
| `safe-pb` | `padding-bottom: env(safe-area-inset-bottom)` | Any element that forms the **bottom edge** of a full-screen view |

Both classes are defined in `src/index.css` under `@layer utilities`.

### Rule

> **Every new full-screen view must apply `safe-pt` to its top bar and
> `safe-pb` to its bottom bar/toolbar.**

### Existing usage

| Component                      | Class applied | Location                                      |
|-------------------------------|---------------|-----------------------------------------------|
| `TopBar` (text editor)         | `safe-pt`     | top bar container                             |
| `LandingPage`                  | `safe-pt`     | header section                                |
| `LandingPage`                  | `safe-pb`     | spacer div at page bottom                     |
| `VectorEditor`                 | `safe-pt`     | top bar container                             |
| `VectorEditor`                 | `safe-pb`     | wrapper around mobile `ToolPalette` bottom bar |

### Example
```tsx
{/* Top bar — always add safe-pt */}
<div className="flex items-center px-3 h-10 bg-background border-b shrink-0 safe-pt">
  ...
</div>

{/* Bottom bar — always add safe-pb */}
<div className="safe-pb">
  <BottomToolbar />
</div>
```

The background color of the bar naturally extends under the system chrome
because padding (not margin) is used, keeping the element background visible
in the inset area while pushing interactive content into the safe zone.

---

## Project Architecture

Loki is a cross-platform office suite built on Tauri 2 (Rust backend) and
React 19 (TypeScript frontend). It targets desktop (macOS, Windows, Linux)
and mobile (Android, iPadOS) via a single codebase.

### Repository layout
```
src/                        React 19 frontend (TypeScript, Vite, Tailwind)
src-tauri/                  Tauri 2 host and Rust workspace
  src/                      Tauri commands and application entry point
    commands/               Tauri IPC command handlers
      fs.rs                 Text document open/save
      pdf.rs                PDF/X export and validation commands
      vector.rs             Vector document commands
    commands/mod.rs         Module declarations
    fonts.rs                Compile-time font registry (include_bytes!)
    lib.rs                  Tauri builder, menu wiring, invoke_handler!
  formats/
    common-core/            Format-agnostic shared types (Block, Inline,
                            StyleDefinition, Metadata, colour_management)
    odt/                    ODT/FODT parser and writer
    vector-core/            Vector document model, SVG parser/writer
    pdf/                    PDF/X-4 and PDF/X-1a exporter (loki-pdf crate)
  epub-logic/               EPUB 3 export
  odt-logic/                Deprecated — legacy TipTap bridge, kept only
                            for epub-logic dependency. Do not add new code
                            here.
```

### Technology stack

| Layer         | Technology                                      |
|---------------|-------------------------------------------------|
| Frontend      | React 19, TypeScript, Vite, Tailwind CSS        |
| UI primitives | shadcn/ui (Radix UI), lucide-react icons        |
| State         | Zustand 5                                       |
| Text editor   | Lexical (Meta)                                  |
| Vector editor | Konva.js / react-konva                          |
| IPC           | Tauri 2 `invoke()` / `tauri::command`           |
| Document I/O  | Custom Rust crates (see `formats/`)             |
| Colour mgmt   | LCMS2 via `lcms2` crate (feature-gated)         |
| Font handling | `ttf-parser`, `rustybuzz`, `subsetter`          |
| PDF output    | `pdf-writer` crate                              |

---

## Code Quality Rules

These rules are enforced by CI and must not be violated.

### File size ceiling

**Every source file must be ≤ 300 lines.** This applies to both Rust (`.rs`)
and TypeScript/TSX (`.ts`, `.tsx`) files. CI enforces this via
`scripts/check-file-sizes.sh`. When a file approaches the limit, proactively
split it into focused submodules before it is exceeded — do not wait until CI
fails.

Split strategy:
- Rust: create a subdirectory (`foo/mod.rs`, `foo/bar.rs`) and declare
  submodules with `mod bar;` in `mod.rs`.
- TypeScript: extract hooks to `useXxx.ts`, pure helpers to `xxxUtils.ts`,
  and sub-components to their own files.

### Rust quality gates (run before every commit)
```bash
cd src-tauri
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash ../scripts/check-file-sizes.sh rust
```

### TypeScript quality gates (run before every commit)
```bash
npx tsc --noEmit
npm test -- --run
bash scripts/check-file-sizes.sh typescript
```

### No unsafe Rust

`unsafe` blocks are never permitted. Zero occurrences is enforced by code
review — Clippy does not catch all unsafe usage.

### Apache 2.0 license header

Every new `.rs` file must begin with the Apache 2.0 license header. Copy the
exact header from an existing file in `formats/odt/src/` — match the comment
style character for character.

---

## Rust Workspace Crate Responsibilities

### `common-core`

Shared types used by all format crates. **Do not add format-specific logic
here.** Contains:
- `Block`, `Inline` — document content model
- `StyleDefinition`, `Metadata` — document metadata and style
- `LexicalDocument`, `LexicalNode` — Lexical editor serialisation types
- `colour_management/` — `Colour` enum, `ColourContext`, `IccProfileStore`,
  `SwatchLibrary`, Pantone lookup table (feature-gated behind
  `colour-management`)

The `colour-management` feature enables LCMS2. All workspace crates that need
colour conversion must declare:
```toml
common-core = { path = "../common-core", features = ["colour-management"] }
```

### `odt-format` (`formats/odt`)

ODT and FODT parser and writer. Reads/writes the ODF text document format.
Bridges between ODF XML and `common-core` types via the Lexical JSON IPC
format. Contains integration tests, round-trip tests, security tests, and
property-based tests via `proptest`.

The Loki namespace extension (`xmlns:loki="http://appthere.com/ns/loki/1.0"`)
is used to preserve non-RGB colour values alongside standard ODF sRGB
approximations for interoperability. Non-Loki readers (LibreOffice, etc.)
see and ignore the `loki:` attributes.

### `vector-core` (`formats/vector-core`)

Vector document model. Contains `VectorDocument`, `Layer`, `VectorObject`
variants (`Rect`, `Ellipse`, `Line`, `Path`, `Group`), object styles, and
transforms. SVG parser (using `roxmltree`) and SVG writer (using `quick-xml`)
with Loki namespace extensions for CMYK/Lab/Spot colour preservation.

### `loki-pdf` (`formats/pdf`)

PDF/X-4 and PDF/X-1a exporter. Two entry points:
- `write_vector_pdf` — exports a `VectorDocument`
- `write_text_pdf` — exports text document blocks + styles + metadata

Pipeline: validate conformance → pre-export colour conversion → transparency
flattening (X-1a only) → font subsetting (via `subsetter` crate) → content
stream generation → PDF structure assembly.

See `formats/pdf/COLOUR_MANAGEMENT.md` for the ICC colour pipeline
architecture and instructions for adding new bundled press profiles.

### `epub-logic`

EPUB 3 exporter. Uses `common-core` types. The legacy JSON round-trip through
`odt-logic` types was eliminated — `epub-logic` now depends directly on
`common-core`.

### `odt-logic` (deprecated)

**Do not add any new code to this crate.** It is retained only because
removing it would require additional migration work. It carries a deprecation
notice in `lib.rs`. All new format code belongs in `odt-format` or
`common-core`.

---

## Colour Management

Loki implements professional ICC colour management via LCMS2.

### Colour type

`common_core::colour_management::Colour` is a tagged enum:
```rust
pub enum Colour {
    Rgb  { r: f32, g: f32, b: f32, a: f32 },        // 0.0–1.0
    Cmyk { c: f32, m: f32, y: f32, k: f32, alpha: f32 },
    Lab  { l: f32, a: f32, b: f32, alpha: f32 },
    Spot { name: String, tint: f32, lab_ref: [f32; 3], cmyk_fallback: Box<Colour> },
    Linked { id: String },
}
```

It serialises with `#[serde(tag = "type")]` — the TypeScript discriminated
union uses `type: 'Rgb' | 'Cmyk' | 'Lab' | 'Spot' | 'Linked'` with field
names matching the Rust variants exactly. **Never add a `Colour` variant
without updating both the Rust serde attributes and the TypeScript type in
`src/lib/types/colour.ts`.**

### Document colour mode

`VectorDocument.colour_settings` carries the working colour space
(`ColourSpace::Srgb`, `ColourSpace::Cmyk { profile }`, etc.) and rendering
intent. All colours in a document are stored and blended in this space.

Text documents (`StyleDefinition.font_colour`, `.background_colour`) use
`Option<Colour>` with `#[serde(default)]` for backward compatibility.

### IPC colour conversion

The frontend cannot call LCMS2 directly. For non-RGB colours the frontend
calls `batch_convert_colours` (a Tauri command) to convert a batch of
`Colour` values to display sRGB `[f32; 4]` arrays. Results are cached in a
`Map<string, string>` keyed by `JSON.stringify(colour)`. The `useDisplayColours`
hook manages this cache; `getDisplayColour(colour, cache, softProofOverrides)`
in `colourUtils.ts` is the single lookup point for the canvas renderer.

### Bundled ICC profiles

Four press profiles are embedded in the binary via `include_bytes!` in
`formats/common-core/icc/`:

| Profile         | Use case                            |
|-----------------|-------------------------------------|
| sRGB IEC61966   | Screen and digital delivery         |
| ISO Coated v2   | European offset print (coated)      |
| SWOP v2         | North American offset print         |
| GRACoL 2006     | North American high-quality offset  |

To add a new profile see `formats/pdf/COLOUR_MANAGEMENT.md`.

---

## PDF/X Export

### Conformance levels

| Level       | Colours allowed      | Transparency | Typical use              |
|-------------|---------------------|--------------|--------------------------|
| PDF/X-4     | CMYK, RGB + ICC, Spot | Yes        | Modern print workflows   |
| PDF/X-1a    | CMYK and Spot only   | No (flattened) | Legacy print / newspapers |

### Export pipeline
```
VectorDocument / text blocks
  │
  ├─ validate()           conformance check — errors block export
  ├─ prepare_for_export() expand Colour::Linked, convert RGB→CMYK (X-1a)
  ├─ flatten_document()   rasterise transparent groups (X-1a only, via resvg)
  ├─ collect_used_glyphs() Pass 1 — build per-font UsedGlyphs sets
  ├─ create_subset()      subset each font via `subsetter` crate
  ├─ embed_font()         write Type0 + CIDFont + ToUnicode CMap + FontFile2
  ├─ emit_blocks() /      Pass 2 — generate content stream
  │  generate_layer_content()
  └─ write PDF structure  catalog, pages, output intent, XMP metadata
```

### Validator-first rule

**The conformance validator (`validate()` / `validate_text()`) must be
implemented and its tests must pass before any writer code is written or
modified.** The validator is the specification made executable. Writing the
writer against a passing validator is what guarantees conformance.

### Font handling

Fonts are embedded using the `subsetter` crate (MIT, from the Typst project).
`subsetter::Profile::pdf(&glyph_ids)` handles variable font table removal,
GSUB/GPOS subsetting, and correct GID preservation — original GIDs are
preserved in the subset, so no GID remapping is needed in the content stream.

The compile-time font registry (`src-tauri/src/fonts.rs`) embeds all 22
bundled fonts via `include_bytes!` and constructs a `MapFontResolver`. The
resolver is passed into `write_text_pdf` as a `&dyn FontResolver` trait
object. **Add new fonts to both `src/assets/fonts/` and
`src-tauri/src/fonts.rs`.**

### Known PDF/X limitations (Phase 9 state)

- Variable font axis evaluation: bold/italic variants of variable fonts share
  the same font bytes. Visual weight difference requires OpenType `gvar`
  evaluation, deferred to a future phase.
- Images in text documents: `Block::Image` emits a warning and is skipped.
- Complex table layout: equal-width columns, no spanning, simplified borders.
- Footnotes and endnotes: not implemented.
- Multi-column layout: not implemented.
- Bidirectional text: not implemented.

---

## Text Layout Engine (PDF export)

The text layout engine (`formats/pdf/src/writer/text/`) uses a two-pass
architecture:

**Pass 1 — glyph collection** (`collect_used_glyphs`): Walk all blocks and
inlines, recording which Unicode characters are needed from which font
family/weight/italic combination.

**Pass 2 — layout and emission** (`emit_blocks`): For each block, resolve
style properties via `resolve_paragraph_props` (which merges named style
defaults from `named_styles.rs` with explicitly set style properties), break
text into lines via greedy word-wrap, apply alignment/spacing/indentation, and
emit PDF content stream operators.

### Style property resolution order

1. Named style defaults (`named_styles.rs`) — e.g. Heading 1 is 24pt bold
2. Parent style properties (one level of ODF style inheritance)
3. Explicitly set properties on the style itself (always win)

ODF length values (`fo:space-before`, `fo:margin-left`, etc.) are normalised
to PDF points by `parse_length_to_pt` in `style_props.rs`.

### Text alignment

Alignment is applied at line emission time in `operators.rs`:
- Left: text starts at the left margin
- Right: text offset so the right edge aligns with the right margin
- Centre: text centred in the usable width
- Justify: left-aligned x position with non-zero `Tw` (word spacing) on all
  but the last line of each paragraph. The last line is always left-aligned.

### Page breaking

Page breaks occur when:
1. The next line would overflow the bottom margin.
2. A block's style has `fo:break-before="page"` (unconditional break).
3. A block with `style:keep-with-next="always"` is at the bottom of a page
   and the following block's first line would not fit (conditional break).

Orphan control: if fewer than `props.orphans` lines of a paragraph fit on the
current page, the entire paragraph is moved to the next page.

---

## Vector Editor

The vector editor (`src/components/VectorEditor/`) is built on Konva.js with
`react-konva`. It renders to an HTML Canvas 2D surface (not SVG DOM).

### Scene graph
```
Konva Stage
  ├── Layer (grid — non-exportable)
  ├── Layer (per document layer, one Konva Layer per VectorDocument Layer)
  │     └── ObjectRenderer (one per VectorObject)
  └── Layer (selection handles — non-exportable)
```

### Tool modes

`ToolMode` in `src/lib/vector/store.ts`: `select`, `rect`, `ellipse`, `line`,
`pan`, `zoom`. Space held temporarily activates pan and releases back on key-up.

### Colour display

The Konva canvas always receives sRGB CSS colour strings. For non-RGB colours,
`useDisplayColours` pre-converts via `batch_convert_colours` (Tauri IPC) and
caches results. `getDisplayColour(colour, displayCache, softProofOverrides)`
in `colourUtils.ts` is the single lookup point — use it everywhere in
`ObjectRenderer` and related components. Never convert colours inline in
render code.

Soft-proof mode (`useSoftProof`) overrides the display cache with
output-profile-converted colours, simulating how the document will look in
print. The `softProofOverrides` map takes precedence over `displayCache`.

### Export format

SVG is the primary vector format. ODG is a secondary export. Both are written
by the Rust `svg_writer` / `odg_writer` (not yet implemented — Phase 3 of
the vector roadmap). The Loki namespace (`xmlns:loki=...`) in SVG preserves
non-RGB colour values for lossless Loki-to-Loki round-trips; non-Loki SVG
readers see sRGB approximations.

---

## Responsive Layout Rules

Loki targets phone (<640px), tablet (640–1023px), and desktop (≥1024px).
The layout switches at these Tailwind breakpoints.

### Desktop (≥1024px)

Tool palette: left sidebar, 48px wide, icon-only with hover tooltips.
Properties panel: right sidebar, 280px wide, persistent.
Canvas: fills remaining horizontal space.

### Tablet (640–1023px)

Tool palette: left sidebar (narrowed).
Properties panel: bottom sheet (collapsed by default, slides up on object
selection).
Canvas: fills remaining space.

### Mobile (<640px)

Tool palette: horizontal scrollable strip at the bottom, 56px tall.
Properties panel: bottom sheet above the tool strip.
Canvas: fills the full screen.

The bottom sheet is implemented with CSS `transform: translateY(...)` and
pointer event drag handling — **do not add a third-party bottom sheet
library.**

Touch targets: all interactive elements must have a minimum 44×44px hit area
on mobile. Use `hitStrokeWidth` on Konva nodes and padding on HTML elements
to achieve this without changing visual size.

---

## IPC Conventions

### Tauri command naming

Rust commands use `snake_case`. TypeScript wrappers in `src/lib/vector/commands.ts`,
`src/lib/tauri/commands.ts` etc. use `camelCase` wrappers around `invoke()`.
```ts
// TypeScript wrapper pattern
export async function saveVectorDocument(
  path: string,
  document: VectorDocument,
): Promise<void> {
  return invoke('save_vector_document', { path, document });
}
```

**`PdfExportSettings` and `ConformanceViolation` use
`#[serde(rename_all = "camelCase")]` on the Rust side** so the TypeScript
receives camelCase field names. All other structs use serde defaults
(snake_case). Do not add `rename_all` to structs that did not have it before
without updating both sides.

### Error handling

Tauri commands return `Result<T, String>` where the `String` is the
`Display` output of the structured `PdfExportError` enum (via `From<PdfExportError>
for String`). The frontend wraps these in try/catch and surfaces them via the
toast notification system (`useToast`). **Every `await invoke(...)` call in
production code must be inside a try/catch.** Silent failures are not
acceptable.

### Content URIs (Android)

On Android, file paths may be `content://` URIs rather than filesystem paths.
Commands that write files check for the `content://` prefix and return the
file bytes to the frontend (via `Ok(Some(Vec<u8>))`) instead of writing
directly. The frontend then calls `writeFile` from `@tauri-apps/plugin-fs` to
complete the write. This pattern is used in `fs.rs`, `export.rs`, and
`pdf.rs` — follow it for any new file-writing command.

---

## Session and Autosave

Each opened document gets a `SessionManager` instance that writes to the
app data directory (not the user's original file). Autosave runs every 30
seconds; snapshots run every 5 minutes. Only an explicit Ctrl+S / File → Save
writes to the original file.

Session files are ODT-format bytes serialised via `serialize_document` /
`deserialize_document` Tauri commands. The session directory is cleaned up
on normal close; orphaned sessions (from crashes) are discoverable via
`SessionManager::find_recoverable()` for crash recovery.

---

## Adding a New Document Type

When adding a new application module (spreadsheet, presentation, etc.):

1. **Rust crate**: create `formats/{type}-core/` following the same structure
   as `vector-core`. Add to `src-tauri/Cargo.toml` workspace members.
2. **Tauri commands**: add `src-tauri/src/commands/{type}.rs`, declare in
   `commands/mod.rs`, register in `lib.rs` `invoke_handler!`.
3. **Frontend store**: create `src/lib/{type}/store.ts` following the
   `VectorEditorState` Zustand pattern.
4. **Frontend commands**: create `src/lib/{type}/commands.ts` following the
   typed wrapper pattern.
5. **Landing page**: wire the document type button in
   `src/components/LandingPage.tsx` to open a new document dialog and mount
   the editor component.
6. **Session autosave**: adapt `useAutoSave` or create a type-specific hook.
7. **Safe area**: apply `safe-pt` and `safe-pb` to the top and bottom bars of
   the new editor (see Android Safe Area section above).
8. **File size**: ensure every new file is ≤ 300 lines from day one.

---

## Bundled Assets

### Fonts

22 variable and static fonts are bundled in `src/assets/fonts/`. They serve
two purposes: display in the Lexical/Konva editors, and PDF embedding via
`include_bytes!` in `src-tauri/src/fonts.rs`.

**When adding a new font:**
1. Add the `.ttf` / `.otf` file to `src/assets/fonts/`.
2. Add an `include_bytes!` constant in `src-tauri/src/fonts.rs`.
3. Register the family with `bold` and `italic` variants in
   `build_font_resolver()`.
4. Add `@font-face` declarations in `src/index.css` if the font should be
   available in the editor UI.

### ICC profiles

Four ICC profiles are embedded in `formats/common-core/icc/` via
`include_bytes!`. They are freely distributable from the ICC registry and
ECI. See `formats/pdf/COLOUR_MANAGEMENT.md` for instructions on adding
additional profiles.

### Document templates

`src/assets/templates/standard.fodt` is the default template for new text
documents. It is loaded as a raw string via Vite's `?raw` import and sent to
the Rust backend as bytes for parsing. When modifying the template, verify
that all named styles used by the PDF layout engine (`Heading 1`,
`Heading 2`, etc.) are present with appropriate `fo:font-family`,
`fo:font-size`, and `fo:text-align` attributes.