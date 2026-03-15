// Utility for parsing user-typed unit input strings.
// Used by UnitInput component.

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
