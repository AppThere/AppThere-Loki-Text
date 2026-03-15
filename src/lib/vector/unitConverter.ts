// TypeScript mirror of the Rust UnitConverter.
// Mirrors conversion factors exactly: 1in=96px, 1mm=96/25.4px, etc.

import type { LengthUnit } from './types';

export const UNIT_SUFFIXES: Record<LengthUnit, string> = {
    Px: 'px',
    Mm: 'mm',
    Cm: 'cm',
    In: 'in',
    Pt: 'pt',
    Pc: 'pc',
};

function factor(unit: LengthUnit, dpi: number): number {
    const scale = dpi / 96;
    switch (unit) {
        case 'Px': return 1;
        case 'Mm': return (96 / 25.4) * scale;
        case 'Cm': return (960 / 25.4) * scale;
        case 'In': return 96 * scale;
        case 'Pt': return (96 / 72) * scale;
        case 'Pc': return 16 * scale;
    }
}

export function toPx(value: number, unit: LengthUnit, dpi = 96): number {
    return value * factor(unit, dpi);
}

export function fromPx(value: number, unit: LengthUnit, dpi = 96): number {
    return value / factor(unit, dpi);
}

/** Format a pixel value with unit suffix, rounded to 3 significant figures. */
export function formatValue(px: number, unit: LengthUnit, dpi = 96): string {
    const v = fromPx(px, unit, dpi);
    const sig = parseFloat(v.toPrecision(3));
    return `${sig}${UNIT_SUFFIXES[unit]}`;
}

/** Parse a unit suffix string into a LengthUnit, or null if unrecognised. */
export function parseSuffix(suffix: string): LengthUnit | null {
    const map: Record<string, LengthUnit> = {
        px: 'Px', mm: 'Mm', cm: 'Cm', in: 'In', pt: 'Pt', pc: 'Pc',
    };
    return map[suffix.toLowerCase()] ?? null;
}
