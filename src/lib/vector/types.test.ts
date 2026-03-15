import { describe, it, expect } from 'vitest';
import type { Colour, DocumentColourSettings } from './types';

describe('Colour JSON contract', () => {
    it('Rgb variant has correct field names', () => {
        const c: Colour = { type: 'Rgb', r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
        const json = JSON.stringify(c);
        const parsed = JSON.parse(json);
        expect(parsed.type).toBe('Rgb');
        expect(parsed.r).toBe(1.0);
        expect(parsed.g).toBe(0.0);
        expect(parsed.b).toBe(0.0);
        expect(parsed.a).toBe(1.0);
    });

    it('Cmyk variant has correct field names', () => {
        const c: Colour = { type: 'Cmyk', c: 0.0, m: 1.0, y: 1.0, k: 0.0, alpha: 1.0 };
        const json = JSON.stringify(c);
        const parsed = JSON.parse(json);
        expect(parsed.type).toBe('Cmyk');
        expect(parsed.alpha).toBeDefined(); // not 'a' — must be 'alpha'
        expect(parsed.a).toBeUndefined();  // 'a' is a Lab field, not Cmyk
    });

    it('Lab variant uses alpha not a for opacity', () => {
        const c: Colour = { type: 'Lab', l: 50.0, a: 25.0, b: -30.0, alpha: 0.8 };
        const json = JSON.stringify(c);
        const parsed = JSON.parse(json);
        expect(parsed.type).toBe('Lab');
        expect(parsed.a).toBe(25.0);    // Lab 'a' channel
        expect(parsed.alpha).toBe(0.8); // opacity is 'alpha'
    });

    it('Linked variant uses id field', () => {
        const c: Colour = { type: 'Linked', id: 'swatch-001' };
        expect(JSON.parse(JSON.stringify(c)).id).toBe('swatch-001');
    });

    it('Spot variant has all required fields', () => {
        const c: Colour = {
            type: 'Spot',
            name: 'PANTONE 186 C',
            tint: 1.0,
            lab_ref: [38.0, 56.0, 28.0],
            cmyk_fallback: { type: 'Cmyk', c: 0.0, m: 0.91, y: 0.76, k: 0.06, alpha: 1.0 },
        };
        const parsed = JSON.parse(JSON.stringify(c));
        expect(parsed.type).toBe('Spot');
        expect(parsed.name).toBe('PANTONE 186 C');
        expect(parsed.tint).toBe(1.0);
        expect(parsed.lab_ref).toHaveLength(3);
        expect(parsed.cmyk_fallback.type).toBe('Cmyk');
    });
});

describe('DocumentColourSettings JSON contract', () => {
    it('default sRGB settings serialise correctly', () => {
        const settings: DocumentColourSettings = {
            working_space: { type: 'Srgb' },
            rendering_intent: 'RelativeColorimetric',
            blackpoint_compensation: true,
        };
        const parsed = JSON.parse(JSON.stringify(settings));
        expect(parsed.working_space.type).toBe('Srgb');
        expect(parsed.rendering_intent).toBe('RelativeColorimetric');
        expect(parsed.blackpoint_compensation).toBe(true);
    });
});
