# Loki — Developer Notes for Claude

## Android Safe Area (Edge-to-Edge UI)

Loki targets Android devices that use edge-to-edge rendering (status bar / camera cut-out at the top, home indicator at the bottom). The Tauri WebView extends under these system UI elements by default.

Two utility CSS classes handle this — **they must be applied to every view**:

| Class | CSS | When to use |
|-------|-----|-------------|
| `safe-pt` | `padding-top: env(safe-area-inset-top)` | Any element that forms the **top edge** of a full-screen view |
| `safe-pb` | `padding-bottom: env(safe-area-inset-bottom)` | Any element that forms the **bottom edge** of a full-screen view |

Both classes are defined in `src/index.css` under `@layer utilities`.

### Rule

> **Every new full-screen view must apply `safe-pt` to its top bar and `safe-pb` to its bottom bar/toolbar.**

### Existing usage

| Component | Class applied | Location |
|-----------|--------------|----------|
| `TopBar` (text editor) | `safe-pt` | top bar container |
| `LandingPage` | `safe-pt` | header section |
| `LandingPage` | `safe-pb` | spacer div at page bottom |
| `VectorEditor` | `safe-pt` | top bar container |
| `VectorEditor` | `safe-pb` | wrapper around mobile `ToolPalette` bottom bar |

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

The background color of the bar naturally extends under the system chrome because padding (not margin) is used, keeping the element background visible in the inset area while pushing interactive content into the safe zone.
