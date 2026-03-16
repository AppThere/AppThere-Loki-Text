// Copyright 2024 AppThere
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { describe, it, expect, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';

afterEach(cleanup);
import { ColourPreviewGrid } from './ColourPreviewGrid';
import type { ColourPreviewPair } from '@/lib/vector/types';

const redRgb: ColourPreviewPair = {
    original: { type: 'Rgb', r: 1, g: 0, b: 0, a: 1 },
    original_display: [1, 0, 0, 1],
    converted_display: [0.9, 0.05, 0.05, 1],
    delta_e: 0.5,
};

const bigDeltaPair: ColourPreviewPair = {
    original: { type: 'Cmyk', c: 0, m: 1, y: 1, k: 0, alpha: 1 },
    original_display: [1, 0, 0, 1],
    converted_display: [0.3, 0.3, 0.3, 1],
    delta_e: 8.0,
};

describe('ColourPreviewGrid', () => {
    it('renders empty state when no pairs', () => {
        render(<ColourPreviewGrid pairs={[]} />);
        expect(screen.getByText(/No colour conversion preview/i)).toBeTruthy();
    });

    it('renders loading skeleton when loading=true', () => {
        const { container } = render(<ColourPreviewGrid pairs={[]} loading={true} />);
        const items = container.querySelectorAll('.animate-pulse');
        expect(items.length).toBeGreaterThan(0);
    });

    it('renders ΔE value for each pair', () => {
        render(<ColourPreviewGrid pairs={[redRgb]} />);
        expect(screen.getByText(/ΔE 0\.5/)).toBeTruthy();
    });

    it('renders Imperceptible label for low ΔE', () => {
        render(<ColourPreviewGrid pairs={[redRgb]} />);
        expect(screen.getByText('Imperceptible')).toBeTruthy();
    });

    it('renders Significant label for high ΔE', () => {
        render(<ColourPreviewGrid pairs={[bigDeltaPair]} />);
        expect(screen.getByText('Significant')).toBeTruthy();
    });

    it('renders multiple pairs', () => {
        render(<ColourPreviewGrid pairs={[redRgb, bigDeltaPair]} />);
        expect(screen.getByText(/ΔE 0\.5/)).toBeTruthy();
        expect(screen.getByText(/ΔE 8\.0/)).toBeTruthy();
    });
});
