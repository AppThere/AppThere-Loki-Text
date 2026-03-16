import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ColourPicker } from './ColourPicker';
import type { Colour, DocumentColourSettings } from '@/lib/vector/types';

// Mock @tauri-apps/api/core so the component can render in jsdom
vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn().mockResolvedValue([]),
}));

const srgbSettings: DocumentColourSettings = {
    working_space: { type: 'Srgb' },
    rendering_intent: 'RelativeColorimetric',
    blackpoint_compensation: true,
};

const emptyCache = new Map<string, string>();

describe('ColourPicker smoke tests', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders RGB sliders for an Rgb colour', () => {
        const colour: Colour = { type: 'Rgb', r: 1, g: 0, b: 0, a: 1 };
        render(
            <ColourPicker
                colour={colour}
                onChange={vi.fn()}
                colourSettings={srgbSettings}
                displayCache={emptyCache}
            />,
        );
        expect(screen.getByLabelText('R')).toBeDefined();
        expect(screen.getByLabelText('G')).toBeDefined();
        expect(screen.getByLabelText('B')).toBeDefined();
        expect(screen.getByLabelText('A')).toBeDefined();
    });

    it('renders CMYK sliders for a Cmyk colour', () => {
        const colour: Colour = { type: 'Cmyk', c: 0, m: 1, y: 1, k: 0, alpha: 1 };
        render(
            <ColourPicker
                colour={colour}
                onChange={vi.fn()}
                colourSettings={srgbSettings}
                displayCache={emptyCache}
            />,
        );
        expect(screen.getByLabelText('C')).toBeDefined();
        expect(screen.getByLabelText('M')).toBeDefined();
        expect(screen.getByLabelText('Y')).toBeDefined();
        expect(screen.getByLabelText('K')).toBeDefined();
    });

    it('renders spot colour name for a Spot colour', () => {
        const colour: Colour = {
            type: 'Spot',
            name: 'PANTONE 186 C',
            tint: 1.0,
            lab_ref: [38, 56, 28],
            cmyk_fallback: { type: 'Cmyk', c: 0, m: 0.91, y: 0.76, k: 0.06, alpha: 1 },
        };
        render(
            <ColourPicker
                colour={colour}
                onChange={vi.fn()}
                colourSettings={srgbSettings}
                displayCache={emptyCache}
            />,
        );
        expect(screen.getAllByText('PANTONE 186 C').length).toBeGreaterThan(0);
    });
});
