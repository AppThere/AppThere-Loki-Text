import { useEffect, useRef } from 'react';
import { useDocumentStore } from '../stores/documentStore';

interface UseAutoSaveOptions {
    intervalMs?: number;
    snapshotIntervalMs?: number;
    enabled?: boolean;
}

/**
 * Session-safe autosave hook.
 *
 * Saves editor state to the session directory every `intervalMs` milliseconds
 * (default 30 s) and creates a snapshot every `snapshotIntervalMs` milliseconds
 * (default 5 min).
 *
 * **The user's original file is never written by this hook.** The only code
 * path that writes to the original file is the explicit user-initiated save in
 * `useFileOperations.handleSave`.
 */
export function useAutoSave({
    intervalMs = 30_000,
    snapshotIntervalMs = 300_000,
    enabled = true,
}: UseAutoSaveOptions = {}) {
    const autoSaveIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
    const snapshotIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
    // Access session from the document store (set by useFileOperations on open)
    const getSession = () => useDocumentStore.getState().session;

    useEffect(() => {
        if (!enabled) return;

        // ── Autosave timer (30 s) ───────────────────────────────────────────
        const doAutoSave = async () => {
            const state = useDocumentStore.getState();
            const session = getSession();
            if (!state.isDirty || !state.currentContent || !session) return;
            try {
                await session.autoSave({
                    content: state.currentContent,
                    styles: state.styles,
                    metadata: state.metadata,
                });
            } catch (err) {
                console.error('[AutoSave] Session autosave failed:', err);
            }
        };

        autoSaveIntervalRef.current = setInterval(doAutoSave, intervalMs);

        // ── Snapshot timer (5 min) ──────────────────────────────────────────
        const doSnapshot = async () => {
            const state = useDocumentStore.getState();
            const session = getSession();
            if (!state.currentContent || !session) return;
            try {
                await session.createSnapshot({
                    content: state.currentContent,
                    styles: state.styles,
                    metadata: state.metadata,
                });
            } catch (err) {
                console.error('[AutoSave] Snapshot failed:', err);
            }
        };

        snapshotIntervalRef.current = setInterval(doSnapshot, snapshotIntervalMs);

        return () => {
            if (autoSaveIntervalRef.current) clearInterval(autoSaveIntervalRef.current);
            if (snapshotIntervalRef.current) clearInterval(snapshotIntervalRef.current);
        };
    }, [enabled, intervalMs, snapshotIntervalMs]);
}
