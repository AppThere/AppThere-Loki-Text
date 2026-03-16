import type {
    BuiltInProfile,
    Colour,
    DocumentColourSettings,
    IccProfileRef,
    VectorObject,
} from './types';

/**
 * Convert a Colour::Rgb to a CSS rgba() string for direct use in Konva
 * and HTML/CSS contexts.
 *
 * Only valid for Rgb variants. For other variants, use the display colour
 * cache populated by useDisplayColours.
 */
export function rgbToKonva(colour: Extract<Colour, { type: 'Rgb' }>): string {
    const { r, g, b, a } = colour;
    return `rgba(${Math.round(r * 255)},${Math.round(g * 255)},${Math.round(b * 255)},${a})`;
}

/**
 * Convert any Colour to a CSS string for display purposes.
 *
 * For Rgb variants: converts directly in TypeScript (no IPC call needed).
 * For all other variants: returns the key used to look up the pre-converted
 * value in the display colour cache.
 */
export function colourToKonva(
    colour: Colour,
    displayCache: Map<string, string>,
): string {
    if (colour.type === 'Rgb') {
        return rgbToKonva(colour);
    }
    const key = colourCacheKey(colour);
    const cached = displayCache.get(key);
    if (!cached) {
        console.warn('[colourToKonva] Cache miss for colour:', colour);
        return 'rgba(0,0,0,0)';
    }
    return cached;
}

/**
 * Produce a stable string key for a Colour value, suitable for use as a
 * Map key or cache key.
 */
export function colourCacheKey(colour: Colour): string {
    return JSON.stringify(colour);
}

/**
 * Returns true if this colour can be converted to display RGB without
 * an IPC call (i.e. it is already sRGB).
 */
export function isDirectlyRenderable(colour: Colour): boolean {
    return colour.type === 'Rgb';
}

/**
 * Collect all non-RGB colours from a list of objects that need IPC conversion.
 * Returns an array of unique colours (deduplicated by cache key).
 */
export function collectNonRgbColours(objects: VectorObject[]): Colour[] {
    const seen = new Set<string>();
    const result: Colour[] = [];

    function visit(colour: Colour): void {
        if (isDirectlyRenderable(colour)) return;
        const key = colourCacheKey(colour);
        if (!seen.has(key)) {
            seen.add(key);
            result.push(colour);
        }
    }

    function visitObject(obj: VectorObject): void {
        const style = obj.style;
        if (style.fill.type === 'Solid') visit(style.fill.colour);
        if (style.stroke.paint.type === 'Solid') visit(style.stroke.paint.colour);
        if (obj.type === 'Group') {
            obj.children.forEach(visitObject);
        }
    }

    objects.forEach(visitObject);
    return result;
}

/**
 * Returns the default DocumentColourSettings (sRGB, relative colorimetric).
 */
export function defaultColourSettings(): DocumentColourSettings {
    return {
        working_space: { type: 'Srgb' },
        rendering_intent: 'RelativeColorimetric',
        blackpoint_compensation: true,
    };
}

/**
 * Convert a display sRGB [r, g, b, a] tuple (0.0–1.0) to a CSS rgba() string.
 * Used to render ColourPreviewPair swatches without a Konva context.
 */
export function displayRgbToCss(rgba: [number, number, number, number]): string {
    const [r, g, b, a] = rgba;
    return `rgba(${Math.round(r * 255)},${Math.round(g * 255)},${Math.round(b * 255)},${a})`;
}

/**
 * Returns a human-readable label for a Colour value.
 *
 * Examples:
 *   Rgb { r:1, g:0, b:0, a:1 }  → "RGB 255 0 0"
 *   Cmyk { c:0, m:1, y:1, k:0 } → "CMYK 0% 100% 100% 0%"
 *   Spot { name: "PANTONE 186 C", tint: 1.0 } → "PANTONE 186 C"
 *   Linked { id: "swatch-001" }  → "Linked"
 */
export function colourLabel(colour: Colour): string {
    switch (colour.type) {
        case 'Rgb': {
            const r = Math.round(colour.r * 255);
            const g = Math.round(colour.g * 255);
            const b = Math.round(colour.b * 255);
            return `RGB ${r} ${g} ${b}`;
        }
        case 'Cmyk': {
            const c = Math.round(colour.c * 100);
            const m = Math.round(colour.m * 100);
            const y = Math.round(colour.y * 100);
            const k = Math.round(colour.k * 100);
            return `CMYK ${c}% ${m}% ${y}% ${k}%`;
        }
        case 'Lab':
            return `Lab ${colour.l.toFixed(1)} ${colour.a.toFixed(1)} ${colour.b.toFixed(1)}`;
        case 'Spot':
            return colour.name;
        case 'Linked':
            return 'Linked';
    }
}

/**
 * Returns a short colour space badge label for display next to a swatch.
 */
export function colourSpaceBadge(colour: Colour): string {
    switch (colour.type) {
        case 'Rgb': return 'RGB';
        case 'Cmyk': return 'CMYK';
        case 'Lab': return 'Lab';
        case 'Spot': return 'Spot';
        case 'Linked': return 'Linked';
    }
}

/** Returns true if the document's working space is CMYK. */
export function isCmykDocument(settings: DocumentColourSettings): boolean {
    return settings.working_space.type === 'Cmyk';
}

/**
 * Returns the IccProfileRef for a CMYK document's working space.
 * Returns null for non-CMYK documents.
 */
export function cmykProfile(settings: DocumentColourSettings): IccProfileRef | null {
    if (settings.working_space.type === 'Cmyk') {
        return settings.working_space.profile;
    }
    return null;
}

/** Construct a DocumentColourSettings for a CMYK document with a built-in profile. */
export function cmykSettings(profile: BuiltInProfile): DocumentColourSettings {
    return {
        working_space: { type: 'Cmyk', profile: { type: 'BuiltIn', profile } },
        rendering_intent: 'RelativeColorimetric',
        blackpoint_compensation: true,
    };
}

/**
 * Parse a hex colour string into a Colour::Rgb value.
 * Accepts #rgb, #rrggbb, or #rrggbbaa (with or without the # prefix).
 * Returns null for invalid input.
 */
export function parseHexColour(hex: string): Colour | null {
    let s = hex.trim().replace(/^#/, '');
    if (s.length === 3) s = s.split('').map((c) => c + c).join('');
    if (s.length === 6) s = s + 'ff';
    if (s.length !== 8) return null;
    const num = parseInt(s, 16);
    if (isNaN(num)) return null;
    const r = ((num >>> 24) & 0xff) / 255;
    const g = ((num >>> 16) & 0xff) / 255;
    const b = ((num >>> 8) & 0xff) / 255;
    const a = (num & 0xff) / 255;
    if ([r, g, b, a].some(isNaN)) return null;
    return { type: 'Rgb', r, g, b, a };
}

/**
 * Extract [r, g, b, a] (0.0–1.0) numbers from a CSS "rgba(...)" string.
 * Returns [0, 0, 0, 1] for invalid input.
 */
export function parseCssRgba(css: string): [number, number, number, number] {
    const m = css.match(/rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)\s*(?:,\s*([\d.]+)\s*)?\)/);
    if (!m) return [0, 0, 0, 1];
    return [
        parseInt(m[1]) / 255,
        parseInt(m[2]) / 255,
        parseInt(m[3]) / 255,
        m[4] !== undefined ? parseFloat(m[4]) : 1,
    ];
}

/**
 * Get display RGBA [0–1 each] from a Colour, using the display cache
 * for non-RGB colours. Falls back to [0, 0, 0, 1] while cache is loading.
 */
export function getDisplayRgba(
    colour: Colour,
    displayCache: Map<string, string>,
): [number, number, number, number] {
    if (colour.type === 'Rgb') {
        return [colour.r, colour.g, colour.b, colour.a];
    }
    const cached = displayCache.get(colourCacheKey(colour));
    if (cached) {
        return parseCssRgba(cached);
    }
    return [0, 0, 0, 1];
}

// ---------------------------------------------------------------------------
// Phase 4 additions
// ---------------------------------------------------------------------------

/** Alias for collectNonRgbColours. Used by the PDF export preview. */
export const collectUniqueColours = collectNonRgbColours;

/**
 * Collect ALL unique colours from a list of objects (both RGB and non-RGB).
 * Used for soft-proof preview where all colours need display conversion.
 */
export function collectAllUniqueColours(objects: VectorObject[]): Colour[] {
    const seen = new Set<string>();
    const result: Colour[] = [];

    function visit(colour: Colour): void {
        const key = colourCacheKey(colour);
        if (!seen.has(key)) {
            seen.add(key);
            result.push(colour);
        }
    }

    function visitObject(obj: VectorObject): void {
        const style = obj.style;
        if (style.fill.type === 'Solid') visit(style.fill.colour);
        if (style.stroke.paint.type === 'Solid') visit(style.stroke.paint.colour);
        if (obj.type === 'Group') obj.children.forEach(visitObject);
    }

    objects.forEach(visitObject);
    return result;
}

/**
 * Get the display CSS string for a colour, respecting soft-proof overrides.
 *
 * Priority: softProofOverrides > direct RGB conversion > displayCache > fallback.
 */
export function getDisplayColour(
    colour: Colour,
    displayCache: Map<string, string>,
    softProofOverrides: Map<string, string> | null,
): string {
    const key = colourCacheKey(colour);
    if (softProofOverrides?.has(key)) return softProofOverrides.get(key)!;
    if (colour.type === 'Rgb') return rgbToKonva(colour);
    return displayCache.get(key) ?? 'rgba(0,0,0,0)';
}
