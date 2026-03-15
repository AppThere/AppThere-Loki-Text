// Utility for parsing user-typed unit input strings.
// Used by UnitInput component and colour channel inputs.

import type { LengthUnit } from './types';
import { toPx, parseSuffix } from './unitConverter';

/**
 * Parse a string typed by the user (e.g. "32mm", "100", "1.5in")
 * into a pixel value.
 *
 * @param raw      - The raw string typed by the user.
 * @param unit     - The current display unit (used if no suffix found).
 * @param dpi      - Screen DPI.
 * @param fallback - Returned if parsing fails.
 */
export function parseUnitInput(
    raw: string,
    unit: LengthUnit,
    dpi = 96,
    fallback = 0,
): number {
    const s = raw.trim();
    if (!s) return fallback;

    // Try to extract trailing unit suffix
    const match = s.match(/^(-?[\d.]+)\s*([a-zA-Z]*)$/);
    if (!match) return fallback;

    const numStr = match[1];
    const suffixStr = match[2].toLowerCase();

    const num = parseFloat(numStr);
    if (isNaN(num)) return fallback;

    if (suffixStr) {
        const parsedUnit = parseSuffix(suffixStr);
        if (!parsedUnit) return fallback;
        return toPx(num, parsedUnit, dpi);
    }

    return toPx(num, unit, dpi);
}

/**
 * Parse a user-typed RGB channel string (0–255 integer) into a 0.0–1.0 float.
 * Accepts strings like "128", "255", "0".
 * Returns null if the input is not a valid integer in [0, 255].
 */
export function parseRgbChannel(raw: string): number | null {
    const s = raw.trim();
    if (!s) return null;
    const num = Math.round(parseFloat(s));
    if (isNaN(num) || num < 0 || num > 255) return null;
    return num / 255;
}

/**
 * Parse a user-typed CMYK channel string (0–100 percentage) into a 0.0–1.0 float.
 * Accepts strings like "50", "100", "0", "75.5".
 * Returns null if the input is not a valid number in [0, 100].
 */
export function parseCmykChannel(raw: string): number | null {
    const s = raw.trim().replace(/%$/, '');
    if (!s) return null;
    const num = parseFloat(s);
    if (isNaN(num) || num < 0 || num > 100) return null;
    return num / 100;
}

/**
 * Format a 0.0–1.0 RGB channel value as a 0–255 integer string.
 */
export function formatRgbChannel(value: number): string {
    return String(Math.round(Math.max(0, Math.min(1, value)) * 255));
}

/**
 * Format a 0.0–1.0 CMYK channel value as a 0–100 percentage string (no % suffix).
 */
export function formatCmykChannel(value: number): string {
    return String(Math.round(Math.max(0, Math.min(1, value)) * 100));
}
