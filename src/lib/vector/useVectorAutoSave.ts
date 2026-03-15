import { useEffect, useRef } from 'react';
import { useVectorStore } from './store';
import { serializeVectorDocument } from './commands';

/**
 * Vector-editor autosave hook.
 * Serialises the document every `intervalMs` ms (default 30s) to session storage.
 * Never writes to the user's original file.
 */
export function useVectorAutoSave({
    intervalMs = 30_000,
    enabled = true,
}: {
    intervalMs?: number;
    enabled?: boolean;
} = {}) {
    const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

    useEffect(() => {
        if (!enabled) return;

        const doSave = async () => {
            const { document, isDirty, currentPath } = useVectorStore.getState();
            if (!document || !isDirty) return;
            try {
                const bytes = await serializeVectorDocument(document);
                const key = `vector_autosave_${currentPath ?? 'untitled'}`;
                // Store as base64 in sessionStorage
                const b64 = btoa(String.fromCharCode(...bytes));
                sessionStorage.setItem(key, b64);
                sessionStorage.setItem(`${key}_ts`, Date.now().toString());
            } catch (err) {
                console.error('[VectorAutoSave] Failed:', err);
            }
        };

        timerRef.current = setInterval(doSave, intervalMs);
        return () => {
            if (timerRef.current) clearInterval(timerRef.current);
        };
    }, [enabled, intervalMs]);
}
