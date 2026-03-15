import type { Colour, DocumentColourSettings, VectorObject } from './types';

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
 * value in the display colour cache. The cache is populated by
 * useDisplayColours before the render pass.
 *
 * If the colour is not in the cache (e.g. during initial load), returns
 * a transparent fallback and logs a warning.
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
