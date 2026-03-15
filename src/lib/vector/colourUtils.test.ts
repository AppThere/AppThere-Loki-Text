import { describe, it, expect, vi } from 'vitest';
import {
    rgbToKonva,
    colourToKonva,
    colourCacheKey,
    isDirectlyRenderable,
    collectNonRgbColours,
    defaultColourSettings,
    colourLabel,
    colourSpaceBadge,
    isCmykDocument,
    cmykProfile,
    displayRgbToCss,
    parseHexColour,
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

describe('colourLabel', () => {
    it('formats Rgb as integer channels', () => {
        expect(colourLabel({ type: 'Rgb', r: 1, g: 0, b: 0, a: 1 })).toBe('RGB 255 0 0');
    });

    it('formats Cmyk as percentages', () => {
        expect(colourLabel({ type: 'Cmyk', c: 0, m: 1, y: 1, k: 0, alpha: 1 })).toBe('CMYK 0% 100% 100% 0%');
    });

    it('formats Lab with one decimal place', () => {
        const label = colourLabel({ type: 'Lab', l: 50, a: 25, b: -10, alpha: 1 });
        expect(label).toBe('Lab 50.0 25.0 -10.0');
    });

    it('returns spot colour name', () => {
        const c: Colour = {
            type: 'Spot',
            name: 'PANTONE 186 C',
            tint: 1.0,
            lab_ref: [38, 56, 28],
            cmyk_fallback: { type: 'Cmyk', c: 0, m: 0.91, y: 0.76, k: 0.06, alpha: 1 },
        };
        expect(colourLabel(c)).toBe('PANTONE 186 C');
    });

    it('returns "Linked" for linked colours', () => {
        expect(colourLabel({ type: 'Linked', id: 'x' })).toBe('Linked');
    });
});

describe('colourSpaceBadge', () => {
    it('returns correct badges', () => {
        expect(colourSpaceBadge({ type: 'Rgb', r: 0, g: 0, b: 0, a: 1 })).toBe('RGB');
        expect(colourSpaceBadge({ type: 'Cmyk', c: 0, m: 0, y: 0, k: 0, alpha: 1 })).toBe('CMYK');
        expect(colourSpaceBadge({ type: 'Lab', l: 0, a: 0, b: 0, alpha: 1 })).toBe('Lab');
        expect(colourSpaceBadge({ type: 'Linked', id: 'x' })).toBe('Linked');
    });
});

describe('isCmykDocument', () => {
    it('returns false for sRGB document', () => {
        expect(isCmykDocument(defaultColourSettings())).toBe(false);
    });

    it('returns true for CMYK document', () => {
        const settings = {
            working_space: { type: 'Cmyk' as const, profile: { type: 'BuiltIn' as const, profile: 'IsoCoatedV2' as const } },
            rendering_intent: 'RelativeColorimetric' as const,
            blackpoint_compensation: true,
        };
        expect(isCmykDocument(settings)).toBe(true);
    });
});

describe('cmykProfile', () => {
    it('returns null for sRGB', () => {
        expect(cmykProfile(defaultColourSettings())).toBeNull();
    });

    it('returns profile for CMYK document', () => {
        const profile = { type: 'BuiltIn' as const, profile: 'IsoCoatedV2' as const };
        const settings = {
            working_space: { type: 'Cmyk' as const, profile },
            rendering_intent: 'RelativeColorimetric' as const,
            blackpoint_compensation: true,
        };
        expect(cmykProfile(settings)).toEqual(profile);
    });
});

describe('displayRgbToCss', () => {
    it('converts opaque red', () => {
        expect(displayRgbToCss([1, 0, 0, 1])).toBe('rgba(255,0,0,1)');
    });

    it('converts semi-transparent grey', () => {
        expect(displayRgbToCss([0.5, 0.5, 0.5, 0.5])).toBe('rgba(128,128,128,0.5)');
    });
});

describe('parseHexColour', () => {
    it('parses 6-digit hex', () => {
        const c = parseHexColour('#ff0000');
        expect(c).not.toBeNull();
        expect(c!.type).toBe('Rgb');
        if (c!.type === 'Rgb') {
            expect(c!.r).toBeCloseTo(1, 5);
            expect(c!.g).toBeCloseTo(0, 5);
        }
    });

    it('parses 3-digit shorthand', () => {
        const c = parseHexColour('#f00');
        expect(c).not.toBeNull();
        expect(c!.type).toBe('Rgb');
    });

    it('parses 8-digit hex with alpha', () => {
        const c = parseHexColour('#ff000080');
        expect(c).not.toBeNull();
        if (c!.type === 'Rgb') {
            expect(c!.a).toBeCloseTo(128 / 255, 3);
        }
    });

    it('returns null for invalid input', () => {
        expect(parseHexColour('not-a-colour')).toBeNull();
        expect(parseHexColour('#gggggg')).toBeNull();
    });
});
