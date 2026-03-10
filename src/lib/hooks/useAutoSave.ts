import { useEffect, useRef } from 'react';
import { useDocumentStore } from '../stores/documentStore';
import { useFileOperations } from './useFileOperations';

interface UseAutoSaveOptions {
    intervalMs?: number;
    enabled?: boolean;
}

export function useAutoSave({ intervalMs = 30000, enabled = true }: UseAutoSaveOptions = {}) {
    const { handleSave } = useFileOperations();
    const intervalRef = useRef<NodeJS.Timeout | null>(null);
    const handleSaveRef = useRef(handleSave);

    // Keep handleSave ref updated
    useEffect(() => {
        handleSaveRef.current = handleSave;
    }, [handleSave]);

    useEffect(() => {
        if (!enabled) return;

        const doAutoSave = async () => {
            // Read latest state from store to avoid effect dependency churn
            const state = useDocumentStore.getState();
            if (!state.isDirty || !state.currentPath || state.isSaving) {
                return;
            }

            try {
                await handleSaveRef.current(true); // background = true
            } catch (error) {
                console.error('Auto-save failed:', error);
            }
        };

        intervalRef.current = setInterval(doAutoSave, intervalMs);

        return () => {
            if (intervalRef.current) {
                clearInterval(intervalRef.current);
            }
        };
    }, [enabled, intervalMs]);
}
