# Phase 1 Manual Verification Tests

## SVG Colour Round-Trip

1. Launch the vector editor.
2. Open `src/assets/test-fixtures/colours.svg`. It contains:
   - A red rect: `fill="#ff0000"`
   - A semi-transparent blue ellipse: `fill="rgba(0,0,255,0.5)"`
   - A named-colour green line: `stroke="green"`
3. Verify in the properties panel:
   - Red rect shows Fill: RGB(255, 0, 0, 1.0)
   - Blue ellipse shows Fill: RGB(0, 0, 255, 0.5)
   - Green line shows Stroke: RGB(0, 128, 0, 1.0)
4. Save the document to a new SVG file.
5. Open the saved file in a browser — verify colours match the original.
6. Reload the saved file in Loki — verify colours still match.

## CMYK Display (Phase 1 preview)

The following cannot be tested until Phase 2 enables CMYK document creation.
Verify that if a document's `colour_settings.working_space` is manually set to
`Cmyk` in the JSON, the editor does not crash and displays the CMYK badge in
the colour picker.

## TypeScript Contract Verification

Run the colour serde integration tests to confirm the JSON shapes match:

```bash
cd src-tauri
cargo test -p vector-core colour_json_shape_matches_typescript_contract
```

This confirms the Rust serialisation matches the TypeScript discriminated union
in `src/lib/vector/types.ts`.
