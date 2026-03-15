import { describe, it, expect } from 'vitest';
import { formatError } from './notifyError';

describe('formatError', () => {
    it('extracts Error.message as description for Error instances', () => {
        const result = formatError('Could not open document', new Error('file not found'));
        expect(result.title).toBe('Could not open document');
        expect(result.description).toBe('file not found');
        expect(result.variant).toBe('destructive');
    });

    it('stringifies non-Error values as description', () => {
        const result = formatError('Save failed', 'network timeout');
        expect(result.title).toBe('Save failed');
        expect(result.description).toBe('network timeout');
        expect(result.variant).toBe('destructive');
    });
});
