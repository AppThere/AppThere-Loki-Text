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

import { describe, it, expect } from 'vitest';
import {
    deltaELabel,
    deltaEColourClass,
    isAutoFixable,
    groupViolationsByLevel,
    standardAllowsTransparency,
    standardAllowsRgb,
    standardDisplayName,
    DELTA_E_THRESHOLDS,
} from './pdfTypes';
import type { ConformanceViolation } from './pdfTypes';

describe('deltaELabel', () => {
    it('returns Imperceptible below 1.0', () => {
        expect(deltaELabel(0.9)).toBe('Imperceptible');
    });
    it('returns Slight between 1.0 and 2.0', () => {
        expect(deltaELabel(1.5)).toBe('Slight');
    });
    it('returns Perceptible between 2.0 and 5.0', () => {
        expect(deltaELabel(3.0)).toBe('Perceptible');
    });
    it('returns Significant at or above 5.0', () => {
        expect(deltaELabel(5.0)).toBe('Significant');
        expect(deltaELabel(10.0)).toBe('Significant');
    });
    it('uses DELTA_E_THRESHOLDS boundary correctly', () => {
        expect(deltaELabel(DELTA_E_THRESHOLDS.imperceptible - 0.001)).toBe('Imperceptible');
        expect(deltaELabel(DELTA_E_THRESHOLDS.slight - 0.001)).toBe('Slight');
        expect(deltaELabel(DELTA_E_THRESHOLDS.perceptible - 0.001)).toBe('Perceptible');
    });
});

describe('deltaEColourClass', () => {
    it('returns muted-foreground for imperceptible', () => {
        expect(deltaEColourClass(0.5)).toBe('text-muted-foreground');
    });
    it('returns yellow for slight', () => {
        expect(deltaEColourClass(1.5)).toContain('yellow');
    });
    it('returns orange for perceptible', () => {
        expect(deltaEColourClass(3.0)).toContain('orange');
    });
    it('returns destructive for significant', () => {
        expect(deltaEColourClass(6.0)).toBe('text-destructive');
    });
});

describe('isAutoFixable', () => {
    it('always returns false (no auto_fixable field on ConformanceViolation)', () => {
        const v: ConformanceViolation = { rule: 'X1a/no-transparency', message: 'test' };
        expect(isAutoFixable(v)).toBe(false);
    });
});

describe('groupViolationsByLevel', () => {
    it('classifies X/empty-document as a warning', () => {
        const v: ConformanceViolation = { rule: 'X/empty-document', message: 'Document has no objects' };
        const { errors, warnings } = groupViolationsByLevel([v]);
        expect(errors).toHaveLength(0);
        expect(warnings).toHaveLength(1);
        expect(warnings[0].rule).toBe('X/empty-document');
    });

    it('classifies other rules as errors', () => {
        const v: ConformanceViolation = { rule: 'X1a/no-transparency', message: 'Transparency not allowed' };
        const { errors, warnings } = groupViolationsByLevel([v]);
        expect(errors).toHaveLength(1);
        expect(warnings).toHaveLength(0);
    });

    it('handles empty violations list', () => {
        const { errors, warnings } = groupViolationsByLevel([]);
        expect(errors).toHaveLength(0);
        expect(warnings).toHaveLength(0);
    });

    it('splits mixed violations correctly', () => {
        const violations: ConformanceViolation[] = [
            { rule: 'X/empty-document', message: 'empty' },
            { rule: 'X1a/no-rgb', message: 'no rgb' },
            { rule: 'X4/missing-output-intent', message: 'no output intent' },
        ];
        const { errors, warnings } = groupViolationsByLevel(violations);
        expect(errors).toHaveLength(2);
        expect(warnings).toHaveLength(1);
    });
});

describe('standardAllowsTransparency', () => {
    it('allows transparency for X4_2008', () => {
        expect(standardAllowsTransparency('X4_2008')).toBe(true);
    });
    it('does not allow transparency for X1a2001', () => {
        expect(standardAllowsTransparency('X1a2001')).toBe(false);
    });
});

describe('standardAllowsRgb', () => {
    it('allows RGB for X4_2008', () => {
        expect(standardAllowsRgb('X4_2008')).toBe(true);
    });
    it('does not allow RGB for X1a2001', () => {
        expect(standardAllowsRgb('X1a2001')).toBe(false);
    });
});

describe('standardDisplayName', () => {
    it('returns a non-empty string for X4_2008', () => {
        expect(standardDisplayName('X4_2008')).toBeTruthy();
    });
    it('returns a non-empty string for X1a2001', () => {
        expect(standardDisplayName('X1a2001')).toBeTruthy();
    });
});
