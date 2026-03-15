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

/**
 * TypeScript types for PDF/X export settings and conformance validation.
 * Mirrors the Rust types in `loki-pdf/src/export_settings.rs`.
 */

/** The PDF/X conformance standard to target. */
export type PdfXStandard =
    /** PDF/X-1a:2001 — CMYK only, no transparency, PDF 1.3. */
    | 'X1a2001'
    /** PDF/X-4:2008 — RGB/CMYK with ICC, transparency allowed, PDF 1.6+. */
    | 'X4_2008';

/** Settings controlling the PDF export. */
export interface PdfExportSettings {
    /** The PDF/X standard to conform to. */
    standard: PdfXStandard;
    /** Bleed amount in points (1/72 inch) on each side. 0 for no bleed. */
    bleed_pt: number;
    /** Output condition identifier (e.g. "FOGRA39" for ISO Coated v2). Required by PDF/X. */
    output_condition_identifier: string;
    /** Human-readable output condition description. */
    output_condition: string;
    /** Registry URL for the output condition. */
    registry_name: string;
}

/** A single PDF/X conformance violation. */
export interface ConformanceViolation {
    /** Short rule identifier, e.g. "X1a/no-transparency". */
    rule: string;
    /** Human-readable description of the violation. */
    message: string;
}

/** Default PDF/X-4 export settings for sRGB documents. */
export const DEFAULT_X4_SETTINGS: PdfExportSettings = {
    standard: 'X4_2008',
    bleed_pt: 0,
    output_condition_identifier: 'sRGB',
    output_condition: 'sRGB IEC61966-2.1',
    registry_name: 'http://www.color.org',
};

/** Default PDF/X-1a export settings for CMYK/print documents. */
export const DEFAULT_X1A_SETTINGS: PdfExportSettings = {
    standard: 'X1a2001',
    bleed_pt: 0,
    output_condition_identifier: 'FOGRA39',
    output_condition: 'ISO Coated v2 300% (ECI)',
    registry_name: 'http://www.color.org',
};

/** Whether this standard allows transparency (opacity < 1.0). */
export function standardAllowsTransparency(standard: PdfXStandard): boolean {
    return standard === 'X4_2008';
}

/** Whether this standard allows RGB colours. */
export function standardAllowsRgb(standard: PdfXStandard): boolean {
    return standard === 'X4_2008';
}

/** Human-readable label for a PDF/X standard. */
export function standardDisplayName(standard: PdfXStandard): string {
    switch (standard) {
        case 'X1a2001':
            return 'PDF/X-1a:2001 (CMYK print)';
        case 'X4_2008':
            return 'PDF/X-4:2008 (RGB/CMYK, transparency)';
    }
}

// ---------------------------------------------------------------------------
// Phase 4 additions — ΔE utilities and violation helpers
// ---------------------------------------------------------------------------

/** Severity threshold values for ΔE colour difference. */
export const DELTA_E_THRESHOLDS = {
    imperceptible: 1.0,
    slight: 2.0,
    perceptible: 5.0,
    // > 5.0 = significant
} as const;

/** Human-readable label for a ΔE value. */
export function deltaELabel(deltaE: number): string {
    if (deltaE < DELTA_E_THRESHOLDS.imperceptible) return 'Imperceptible';
    if (deltaE < DELTA_E_THRESHOLDS.slight) return 'Slight';
    if (deltaE < DELTA_E_THRESHOLDS.perceptible) return 'Perceptible';
    return 'Significant';
}

/** Tailwind text colour class for a ΔE value. */
export function deltaEColourClass(deltaE: number): string {
    if (deltaE < DELTA_E_THRESHOLDS.imperceptible) return 'text-muted-foreground';
    if (deltaE < DELTA_E_THRESHOLDS.slight) return 'text-yellow-600 dark:text-yellow-400';
    if (deltaE < DELTA_E_THRESHOLDS.perceptible) return 'text-orange-600 dark:text-orange-400';
    return 'text-destructive';
}

/**
 * Returns true if the violation is auto-fixable.
 *
 * Note: ConformanceViolation (from pdfTypes.ts) only has 'rule' and 'message'
 * fields — there is no auto_fixable field in the current type. This always
 * returns false until the Rust side adds that field.
 */
export function isAutoFixable(_violation: ConformanceViolation): boolean {
    return false;
}

/**
 * Group violations into errors and warnings.
 *
 * ConformanceViolation only has 'rule' and 'message' — no 'level' field.
 * We classify by rule: 'X/empty-document' → warning, everything else → error.
 */
export function groupViolationsByLevel(violations: ConformanceViolation[]): {
    errors: ConformanceViolation[];
    warnings: ConformanceViolation[];
} {
    const errors: ConformanceViolation[] = [];
    const warnings: ConformanceViolation[] = [];
    for (const v of violations) {
        if (v.rule === 'X/empty-document') warnings.push(v);
        else errors.push(v);
    }
    return { errors, warnings };
}
