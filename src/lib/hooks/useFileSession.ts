import { useDocumentStore } from '../stores/documentStore';
import { SessionManager } from '../session/SessionManager';

export function useFileSession() {
    const { setSession } = useDocumentStore();

    const startSession = async (originalPath: string): Promise<SessionManager> => {
        const mgr = await SessionManager.create(originalPath);
        setSession(mgr);
        return mgr;
    };

    const endSession = async () => {
        const current = useDocumentStore.getState().session;
        if (current) {
            try { await current.cleanup(); } catch { /* best-effort */ }
            setSession(null);
        }
    };

    return { startSession, endSession };
}
