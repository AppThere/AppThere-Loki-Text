import { describe, it, expect, vi } from 'vitest';
import {
    rgbToKonva,
    colourToKonva,
    colourCacheKey,
    isDirectlyRenderable,
    collectNonRgbColours,
    defaultColourSettings,
} from './colourUtils';
import type { Colour, VectorObject } from './types';
import { defaultStyle, identityTransform } from './types';

describe('rgbToKonva', () => {
    it('converts full red', () => {
        const c: Extract<Colour, { type: 'Rgb' }> = { type: 'Rgb', r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        expect(rgbToKonva(c)).toBe('rgba(255,0,0,1)');
    });

    it('converts mid-grey with alpha', () => {
        const c: Extract<Colour, { type: 'Rgb' }> = { type: 'Rgb', r: 0.5, g: 0.5, b: 0.5, a: 0.5 };
        expect(rgbToKonva(c)).toBe('rgba(128,128,128,0.5)');
    });
});

describe('colourToKonva', () => {
    it('converts Rgb directly without cache lookup', () => {
        const c: Colour = { type: 'Rgb', r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        const cache = new Map<string, string>();
        expect(colourToKonva(c, cache)).toBe('rgba(255,0,0,1)');
    });

    it('returns cached value for Cmyk colour', () => {
        const c: Colour = { type: 'Cmyk', c: 0.0, m: 1.0, y: 1.0, k: 0.0, alpha: 1.0 };
        const key = colourCacheKey(c);
        const cache = new Map([[key, 'rgba(255,0,0,1)']]);
        expect(colourToKonva(c, cache)).toBe('rgba(255,0,0,1)');
    });

    it('returns transparent and warns for Cmyk with empty cache', () => {
        const c: Colour = { type: 'Cmyk', c: 0.0, m: 1.0, y: 1.0, k: 0.0, alpha: 1.0 };
        const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
        const result = colourToKonva(c, new Map());
        expect(result).toBe('rgba(0,0,0,0)');
        expect(warnSpy).toHaveBeenCalled();
        warnSpy.mockRestore();
    });
});

describe('colourCacheKey', () => {
    it('returns stable string for repeated calls', () => {
        const c: Colour = { type: 'Rgb', r: 0.5, g: 0.3, b: 0.1, a: 1.0 };
        expect(colourCacheKey(c)).toBe(colourCacheKey(c));
    });

    it('returns different strings for different colours', () => {
        const c1: Colour = { type: 'Rgb', r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        const c2: Colour = { type: 'Rgb', r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
        expect(colourCacheKey(c1)).not.toBe(colourCacheKey(c2));
    });
});

describe('isDirectlyRenderable', () => {
    it('returns true for Rgb', () => {
        expect(isDirectlyRenderable({ type: 'Rgb', r: 1, g: 0, b: 0, a: 1 })).toBe(true);
    });

    it('returns false for Cmyk', () => {
        expect(isDirectlyRenderable({ type: 'Cmyk', c: 0, m: 0, y: 0, k: 0, alpha: 1 })).toBe(false);
    });

    it('returns false for Lab', () => {
        expect(isDirectlyRenderable({ type: 'Lab', l: 50, a: 0, b: 0, alpha: 1 })).toBe(false);
    });

    it('returns false for Spot', () => {
        const c: Colour = {
            type: 'Spot',
            name: 'PANTONE 186 C',
            tint: 1.0,
            lab_ref: [38.0, 56.0, 28.0],
            cmyk_fallback: { type: 'Cmyk', c: 0, m: 0.91, y: 0.76, k: 0.06, alpha: 1 },
        };
        expect(isDirectlyRenderable(c)).toBe(false);
    });

    it('returns false for Linked', () => {
        expect(isDirectlyRenderable({ type: 'Linked', id: 'swatch-001' })).toBe(false);
    });
});

function makeRectObject(id: string, fillColour: Colour): VectorObject {
    return {
        type: 'Rect',
        id,
        label: null,
        style: {
            ...defaultStyle(),
            fill: { type: 'Solid', colour: fillColour },
        },
        transform: identityTransform(),
        visible: true,
        locked: false,
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        rx: 0,
        ry: 0,
    };
}

describe('collectNonRgbColours', () => {
    it('returns empty array when all objects are RGB', () => {
        const obj = makeRectObject('r1', { type: 'Rgb', r: 1, g: 0, b: 0, a: 1 });
        expect(collectNonRgbColours([obj])).toHaveLength(0);
    });

    it('returns CMYK colour from object', () => {
        const cmyk: Colour = { type: 'Cmyk', c: 0, m: 1, y: 1, k: 0, alpha: 1 };
        const obj = makeRectObject('r1', cmyk);
        const result = collectNonRgbColours([obj]);
        expect(result).toHaveLength(1);
        expect(result[0]).toEqual(cmyk);
    });

    it('deduplicates identical colours', () => {
        const cmyk: Colour = { type: 'Cmyk', c: 0, m: 1, y: 1, k: 0, alpha: 1 };
        const obj1 = makeRectObject('r1', cmyk);
        const obj2 = makeRectObject('r2', cmyk);
        expect(collectNonRgbColours([obj1, obj2])).toHaveLength(1);
    });
});

describe('defaultColourSettings', () => {
    it('returns sRGB working space', () => {
        const settings = defaultColourSettings();
        expect(settings.working_space.type).toBe('Srgb');
    });
});
