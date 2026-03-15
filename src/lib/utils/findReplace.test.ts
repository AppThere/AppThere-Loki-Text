import { describe, it, expect } from 'vitest';
import { findMatches } from './findReplace';

describe('findMatches', () => {
    it('returns empty array for empty search term', () => {
        expect(findMatches('hello world', '', false, false)).toEqual([]);
    });

    it('returns empty array when search term is longer than text', () => {
        expect(findMatches('hi', 'hello', false, false)).toEqual([]);
    });

    it('finds a basic substring match', () => {
        const matches = findMatches('hello world', 'world', false, false);
        expect(matches).toEqual([{ start: 6, end: 11 }]);
    });

    it('finds multiple matches in a single text node', () => {
        const matches = findMatches('abcabc', 'abc', false, false);
        expect(matches).toHaveLength(2);
        expect(matches[0]).toEqual({ start: 0, end: 3 });
        expect(matches[1]).toEqual({ start: 3, end: 6 });
    });

    it('case-insensitive match (default)', () => {
        const matches = findMatches('Hello WORLD', 'hello', false, false);
        expect(matches).toHaveLength(1);
        expect(matches[0]).toEqual({ start: 0, end: 5 });
    });

    it('case-sensitive match does not match wrong case', () => {
        const matches = findMatches('Hello WORLD', 'hello', true, false);
        expect(matches).toHaveLength(0);
    });

    it('case-sensitive match finds correct case', () => {
        const matches = findMatches('Hello hello', 'hello', true, false);
        expect(matches).toHaveLength(1);
        expect(matches[0]).toEqual({ start: 6, end: 11 });
    });

    it('whole-word match does not match mid-word occurrences', () => {
        const matches = findMatches('foobar foo', 'foo', false, true);
        expect(matches).toHaveLength(1);
        expect(matches[0]).toEqual({ start: 7, end: 10 });
    });

    it('whole-word match finds standalone words', () => {
        const matches = findMatches('the theory is there', 'the', false, true);
        expect(matches).toHaveLength(1);
        expect(matches[0]).toEqual({ start: 0, end: 3 });
    });
});
