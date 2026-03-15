/**
 * Session recovery dialog.
 *
 * Shown on startup when the app finds session directories with an unsaved
 * `current.odt` — indicating the app was closed without saving (crash or
 * force-quit). The user can recover or discard each session.
 */

import { useEffect, useState } from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { SessionManager, SessionMeta } from '@/lib/session/SessionManager';
import { useFileOperations } from '@/lib/hooks/useFileOperations';
import { useDocumentStore } from '@/lib/stores/documentStore';

export function SessionRecovery() {
    const [open, setOpen] = useState(false);
    const [sessions, setSessions] = useState<SessionMeta[]>([]);
    const { loadDocument } = useFileOperations();
    const setSession = useDocumentStore((s) => s.setSession);

    useEffect(() => {
        SessionManager.findRecoverable().then((found) => {
            if (found.length > 0) {
                setSessions(found);
                setOpen(true);
            }
        }).catch(() => { /* no-op: app data dir may not exist on first run */ });
    }, []);

    const handleRecover = async (meta: SessionMeta) => {
        try {
            const mgr = await SessionManager.load(meta.sessionId);
            const state = await mgr.loadCurrent();
            if (!state) return handleDiscard(meta);

            // Load the recovered state into the store
            const { setContent, setStyles, setMetadata, setPath, markDirty } =
                useDocumentStore.getState();
            setPath(meta.originalPath);
            setContent(state.content);
            setStyles(state.styles);
            setMetadata(state.metadata);
            setSession(mgr);
            markDirty(); // Unsaved changes exist
        } catch (err) {
            console.error('[SessionRecovery] Failed to recover:', err);
        }
        removeSession(meta.sessionId);
    };

    const handleDiscard = async (meta: SessionMeta) => {
        try {
            const mgr = await SessionManager.load(meta.sessionId);
            await mgr.cleanup();
        } catch { /* best-effort */ }
        removeSession(meta.sessionId);
    };

    const removeSession = (sessionId: string) => {
        setSessions((prev) => {
            const remaining = prev.filter((s) => s.sessionId !== sessionId);
            if (remaining.length === 0) setOpen(false);
            return remaining;
        });
    };

    const handleDiscardAll = async () => {
        await Promise.allSettled(sessions.map(handleDiscard));
        setOpen(false);
    };

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogContent className="max-w-2xl">
                <DialogHeader>
                    <DialogTitle>Recover Unsaved Work</DialogTitle>
                </DialogHeader>

                <p className="text-sm text-muted-foreground">
                    These documents had unsaved changes when the app last closed:
                </p>

                <div className="space-y-3 max-h-72 overflow-y-auto">
                    {sessions.map((s) => (
                        <div
                            key={s.sessionId}
                            className="border rounded-lg p-4 flex items-center justify-between gap-4"
                        >
                            <div className="min-w-0">
                                <p className="font-medium truncate">{s.originalPath || 'Untitled'}</p>
                                <p className="text-xs text-muted-foreground">
                                    Last saved: {new Date(s.lastModified).toLocaleString()}
                                    {' · '}{s.autoSaveCount} autosave{s.autoSaveCount !== 1 ? 's' : ''}
                                </p>
                            </div>
                            <div className="flex gap-2 shrink-0">
                                <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => handleDiscard(s)}
                                >
                                    Discard
                                </Button>
                                <Button size="sm" onClick={() => handleRecover(s)}>
                                    Recover
                                </Button>
                            </div>
                        </div>
                    ))}
                </div>

                <DialogFooter>
                    <Button variant="ghost" onClick={handleDiscardAll}>
                        Discard All
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
