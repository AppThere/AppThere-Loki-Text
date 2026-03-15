import { describe, it, expect } from 'vitest';
import {
    parseRgbChannel,
    parseCmykChannel,
    formatRgbChannel,
    formatCmykChannel,
} from './parseUnitInput';

describe('parseRgbChannel', () => {
    it('parses integer values within [0, 255]', () => {
        expect(parseRgbChannel('128')).toBeCloseTo(128 / 255, 5);
        expect(parseRgbChannel('0')).toBe(0);
        expect(parseRgbChannel('255')).toBeCloseTo(1, 5);
    });

    it('rounds float input to nearest integer', () => {
        expect(parseRgbChannel('127.6')).toBeCloseTo(128 / 255, 5);
    });

    it('returns null for values outside [0, 255]', () => {
        expect(parseRgbChannel('256')).toBeNull();
        expect(parseRgbChannel('-1')).toBeNull();
    });

    it('returns null for non-numeric input', () => {
        expect(parseRgbChannel('')).toBeNull();
        expect(parseRgbChannel('abc')).toBeNull();
    });
});

describe('parseCmykChannel', () => {
    it('parses percentage values within [0, 100]', () => {
        expect(parseCmykChannel('50')).toBeCloseTo(0.5, 5);
        expect(parseCmykChannel('0')).toBe(0);
        expect(parseCmykChannel('100')).toBeCloseTo(1, 5);
    });

    it('strips trailing % sign', () => {
        expect(parseCmykChannel('75%')).toBeCloseTo(0.75, 5);
    });

    it('handles decimal values', () => {
        expect(parseCmykChannel('33.3')).toBeCloseTo(0.333, 3);
    });

    it('returns null for values outside [0, 100]', () => {
        expect(parseCmykChannel('101')).toBeNull();
        expect(parseCmykChannel('-1')).toBeNull();
    });

    it('returns null for non-numeric input', () => {
        expect(parseCmykChannel('')).toBeNull();
        expect(parseCmykChannel('abc')).toBeNull();
    });
});

describe('formatRgbChannel', () => {
    it('formats 0.0–1.0 as 0–255 integer string', () => {
        expect(formatRgbChannel(0)).toBe('0');
        expect(formatRgbChannel(1)).toBe('255');
        expect(formatRgbChannel(0.5)).toBe('128');
    });

    it('clamps values outside [0, 1]', () => {
        expect(formatRgbChannel(-0.5)).toBe('0');
        expect(formatRgbChannel(1.5)).toBe('255');
    });
});

describe('formatCmykChannel', () => {
    it('formats 0.0–1.0 as 0–100 integer string', () => {
        expect(formatCmykChannel(0)).toBe('0');
        expect(formatCmykChannel(1)).toBe('100');
        expect(formatCmykChannel(0.5)).toBe('50');
    });

    it('clamps values outside [0, 1]', () => {
        expect(formatCmykChannel(-1)).toBe('0');
        expect(formatCmykChannel(2)).toBe('100');
    });
});
