import { describe, it, expect } from 'vitest';
import { deriveWindowTitle } from './useWindowTitle';

describe('deriveWindowTitle', () => {
    it('uses metadata title when present', () => {
        expect(deriveWindowTitle('My Novel', '/docs/novel.odt', false)).toBe('My Novel');
    });

    it('appends " *" when dirty', () => {
        expect(deriveWindowTitle('My Novel', '/docs/novel.odt', true)).toBe('My Novel *');
    });

    it('falls back to filename when title is null', () => {
        expect(deriveWindowTitle(null, '/home/user/report.odt', false)).toBe('report.odt');
    });

    it('falls back to filename when title is empty string', () => {
        expect(deriveWindowTitle('', '/home/user/report.odt', false)).toBe('report.odt');
    });

    it('falls back to "Untitled Document" when title and path are null', () => {
        expect(deriveWindowTitle(null, null, false)).toBe('Untitled Document');
    });

    it('marks "Untitled Document" dirty when no title and no path', () => {
        expect(deriveWindowTitle(null, null, true)).toBe('Untitled Document *');
    });
});
