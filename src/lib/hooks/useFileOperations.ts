import { useState } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readFile, writeFile } from '@tauri-apps/plugin-fs';
import { openDocument, saveDocument, saveEpub } from '../tauri/commands';
import { useDocumentStore } from '../stores/documentStore';
import { useHistoryStore } from '../stores/historyStore';
import { useSessionPersistence } from './useSessionPersistence';
import { SessionManager } from '../session/SessionManager';
import { FileType } from '@/components/Dialogs/FileTypeDialog';
import standardTemplate from '@/assets/templates/standard.fodt?raw';

export function useFileOperations() {
    const [isLoading, setIsLoading] = useState(false);

    const {
        currentPath,
        currentContent,
        styles,
        metadata,
        session,
        setPath,
        setContent,
        setStyles,
        setMetadata,
        setSession,
        markClean,
        markDirty,
        markSaving,
        markSaved,
        reset: resetStore,
    } = useDocumentStore();
    const { addDocument, addTemplate } = useHistoryStore();
    const { clearSession } = useSessionPersistence();

    // ── Session helpers ────────────────────────────────────────────────────

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

    // ── Public handlers ────────────────────────────────────────────────────

    const handleNew = async () => {
        setIsLoading(true);
        try {
            await endSession();
            clearSession();
            const templateBytes = new TextEncoder().encode(standardTemplate);
            const response = await openDocument('internal://standard.fodt', templateBytes);
            setPath('');
            setContent(response.content);
            setStyles(response.styles);
            setMetadata({ ...response.metadata, title: 'Untitled Document', identifier: null });
            markDirty();
        } catch (error) {
            console.error('Failed to create new document:', error);
            throw error;
        } finally {
            setIsLoading(false);
        }
    };

    const handleClose = async () => {
        await endSession();
        clearSession();
        resetStore();
    };

    const loadDocument = async (path: string) => {
        setIsLoading(true);
        try {
            await endSession();

            const fileBytes = await readFile(path);
            const response = await openDocument(path, fileBytes);

            setPath(path);
            setContent(response.content);
            setStyles(response.styles);
            setMetadata(response.metadata);
            addDocument({
                path,
                name: response.metadata.title || path.split('/').pop() || 'Untitled',
                type: 'text',
            });

            // Create session AFTER content is in the store
            const mgr = await startSession(path);
            await mgr.autoSave({
                content: response.content,
                styles: response.styles,
                metadata: response.metadata,
            });

            markClean();
        } catch (error) {
            console.error('Failed to load document:', error);
            throw error;
        } finally {
            setIsLoading(false);
        }
    };

    const handleOpen = async () => {
        try {
            const selected = await open({
                title: 'Open AppThere Document',
                filters: [{ name: 'Document', extensions: ['odt', 'fodt'] }],
            });
            if (selected) {
                const path = typeof selected === 'string' ? selected : (selected as any).path;
                if (path) await loadDocument(path);
            }
        } catch (error) {
            console.error('Failed handling open dialog:', error);
        }
    };

    const handleOpenTemplate = async () => {
        try {
            const selected = await open({
                title: 'New Document from Template',
                filters: [{ name: 'Template', extensions: ['ott', 'odt', 'fodt'] }],
            });
            if (!selected) return;
            const path = typeof selected === 'string' ? selected : (selected as any).path;
            if (!path) return;

            setIsLoading(true);
            try {
                await endSession();
                clearSession();
                const fileBytes = await readFile(path);
                const response = await openDocument(path, fileBytes);

                setPath('');
                setContent(response.content);
                setStyles(response.styles);
                setMetadata({ ...response.metadata, title: 'Untitled Document', identifier: null });
                addTemplate('text', {
                    path,
                    name: response.metadata.title || path.split('/').pop() || 'Untitled',
                    type: 'text',
                });
                markDirty();
            } catch (error) {
                console.error('Failed to load template:', error);
                throw error;
            } finally {
                setIsLoading(false);
            }
        } catch (error) {
            console.error('Failed handling template open dialog:', error);
            throw error;
        }
    };

    /**
     * Save the document to disk (user-initiated).
     *
     * Routes through the active session so that autosave and explicit save
     * stay in sync. Falls back to `saveDocument` when there is no session
     * (e.g. new/unsaved document that has no original path yet).
     */
    const handleSave = async (background = false) => {
        if (!currentPath || !currentContent) return handleSaveAs();

        if (background) markSaving(); else setIsLoading(true);

        try {
            if (session && currentPath) {
                // Safe path: write through session (keeps session in sync)
                await session.saveToOriginal({
                    content: currentContent,
                    styles,
                    metadata,
                });
            } else {
                // Fallback for content:// URIs or when session is unavailable
                const bytes = await saveDocument(
                    currentPath,
                    JSON.stringify(currentContent),
                    styles,
                    metadata,
                    currentPath,
                );
                if (bytes && currentPath.startsWith('content://')) {
                    await writeFile(currentPath, bytes);
                }
            }

            if (background) markSaved(); else markClean();
        } catch (error) {
            console.error('Failed to save:', error);
            if (background) markClean();
            throw error;
        } finally {
            if (!background) setIsLoading(false);
        }
    };

    const handleSaveAs = async (explicitType?: FileType) => {
        if (!currentContent) return;

        try {
            const cleanTitle = (metadata.title || 'Untitled')
                .replace(/[<>:"/\\|?*]/g, '_')
                .trim();
            const ext = explicitType || 'odt';
            const selected = await save({
                title: 'Save AppThere Document As',
                defaultPath: `${cleanTitle}.${ext}`,
                filters: explicitType
                    ? [{ name: explicitType === 'odt' ? 'ODT Document' : 'Flat XML ODT', extensions: [ext] }]
                    : [{ name: 'Document', extensions: ['odt', 'fodt'] }],
            });
            if (!selected) return;

            setIsLoading(true);
            const path = typeof selected === 'string' ? selected : (selected as any).path;
            if (!path) return;

            const bytes = await saveDocument(
                path,
                JSON.stringify(currentContent),
                styles,
                metadata,
                currentPath || undefined,
            );
            if (bytes) {
                if (path.startsWith('content://')) await writeFile(path, bytes);
                setPath(path);
                // Start a new session for the new path
                await endSession();
                await startSession(path);
                addDocument({
                    path,
                    name: metadata.title || path.split('/').pop() || 'Untitled',
                    type: 'text',
                });
                markClean();
            }
        } catch (error) {
            console.error('Failed handling save as dialog:', error);
            throw error;
        } finally {
            setIsLoading(false);
        }
    };

    const handleExportEPUB = async () => {
        if (!currentContent) return;
        try {
            const cleanTitle = (metadata.title || 'Untitled')
                .replace(/[<>:"/\\|?*]/g, '_')
                .trim();
            const selected = await save({
                title: 'Export to EPUB',
                defaultPath: `${cleanTitle}.epub`,
                filters: [{ name: 'EPUB Ebook', extensions: ['epub'] }],
            });
            if (!selected) return;

            setIsLoading(true);
            const path = typeof selected === 'string' ? selected : (selected as any).path;
            if (!path) return;

            const bytes = await saveEpub(path, JSON.stringify(currentContent), styles, metadata, []);
            if (bytes && path.startsWith('content://')) await writeFile(path, bytes);
        } catch (error) {
            console.error('Failed to export EPUB:', error);
            throw error;
        } finally {
            setIsLoading(false);
        }
    };

    return {
        handleOpen,
        handleOpenTemplate,
        handleSave,
        handleSaveAs,
        handleNew,
        handleClose,
        handleExportEPUB,
        loadDocument,
        isLoading,
    };
}
