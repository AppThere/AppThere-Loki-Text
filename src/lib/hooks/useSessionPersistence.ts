import { useEffect, useCallback } from 'react';
import { useDocumentStore } from '../stores/documentStore';

const SESSION_KEY = 'loki_session';

export function useSessionPersistence() {
    const store = useDocumentStore();

    const saveSession = useCallback(() => {
        // We only save if there is content or we are in a named file
        if (!store.currentContent && !store.currentPath) {
            return;
        }

        try {
            const sessionData = {
                currentPath: store.currentPath,
                currentContent: store.currentContent,
                styles: store.styles,
                metadata: store.metadata,
                isDirty: store.isDirty,
                timestamp: Date.now(),
            };
            localStorage.setItem(SESSION_KEY, JSON.stringify(sessionData));
            console.log('[Session] Saved to localStorage');
        } catch (e) {
            console.error('[Session] Failed to save:', e);
        }
    }, [store]);

    const loadSession = useCallback(() => {
        const data = localStorage.getItem(SESSION_KEY);
        if (!data) return null;

        try {
            const session = JSON.parse(data);
            // Optional: Check timestamp if you want to expire old sessions
            return session;
        } catch (e) {
            console.error('[Session] Failed to load:', e);
            localStorage.removeItem(SESSION_KEY);
            return null;
        }
    }, []);

    const clearSession = useCallback(() => {
        localStorage.removeItem(SESSION_KEY);
        console.log('[Session] Cleared');
    }, []);

    // Setup lifecycle listeners
    useEffect(() => {
        const handleVisibilityChange = () => {
            if (document.visibilityState === 'hidden') {
                saveSession();
            }
        };

        const handleBeforeUnload = () => {
            saveSession();
        };

        document.addEventListener('visibilitychange', handleVisibilityChange);
        window.addEventListener('beforeunload', handleBeforeUnload);

        return () => {
            document.removeEventListener('visibilitychange', handleVisibilityChange);
            window.removeEventListener('beforeunload', handleBeforeUnload);
        };
    }, [saveSession]);

    // Periodically save if dirty (every 30 seconds as a safety net)
    useEffect(() => {
        if (!store.isDirty) return;

        const interval = setInterval(() => {
            saveSession();
        }, 30000);

        return () => clearInterval(interval);
    }, [store.isDirty, saveSession]);

    return {
        saveSession,
        loadSession,
        clearSession,
    };
}
