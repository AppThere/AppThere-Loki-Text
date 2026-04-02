import { useState } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readFile, writeFile } from '@tauri-apps/plugin-fs';
import { openDocument, saveDocument, takePersistableUriPermission, openFilePicker } from '../tauri/commands';
import { useDocumentStore } from '../stores/documentStore';
import { useHistoryStore } from '../stores/historyStore';
import { useSessionPersistence } from './useSessionPersistence';
import { FileType } from '@/components/Dialogs/FileTypeDialog';
import standardTemplate from '@/assets/templates/standard.fodt?raw';
import { notifyError } from '@/lib/utils/notifyError';
import { useFileSession } from './useFileSession';
import { useFileExport } from './useFileExport';

export function useFileOperations() {
    const [isLoadingInternal, setIsLoadingInternal] = useState(false);
    const { startSession, endSession } = useFileSession();
    const { handleExportEPUB, handleExportPDF, isExporting } = useFileExport();

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
        markClean,
        markDirty,
        markSaving,
        markSaved,
        reset: resetStore,
    } = useDocumentStore();
    const { addDocument, addTemplate } = useHistoryStore();
    const { clearSession } = useSessionPersistence();

    const isLoading = isLoadingInternal || isExporting;
    const setIsLoading = setIsLoadingInternal;

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
            notifyError('Failed to create new document', error);
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

            // For Save As paths (ACTION_CREATE_DOCUMENT), persist the permission
            // so the file can be re-opened after the app process is killed.
            // Files opened via handleOpen already have the permission persisted
            // inside FilePickerPlugin's activity result callback.
            if (path.startsWith('content://')) {
                try {
                    await takePersistableUriPermission(path);
                } catch {
                    // Non-fatal: the URI may not support persistable permissions.
                }
            }

            // plugin-fs readFile uses Android's ContentResolver for content:// URIs,
            // so it works for both regular paths and SAF content:// URIs.
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
            notifyError('Failed to load document', error);
            throw error;
        } finally {
            setIsLoading(false);
        }
    };

    const handleOpen = async () => {
        try {
            let path: string | null = null;
            if (/android/i.test(navigator.userAgent)) {
                // FilePickerPlugin uses ACTION_OPEN_DOCUMENT which grants a
                // persistable permission and calls takePersistableUriPermission
                // inside the activity result callback before returning the URI.
                try {
                    path = await openFilePicker();
                } catch (err: unknown) {
                    if (String(err).includes('cancelled')) return;
                    throw err;
                }
            } else {
                const selected = await open({
                    title: 'Open AppThere Document',
                    filters: [{ name: 'Document', extensions: ['odt', 'fodt'] }],
                });
                if (selected) path = typeof selected === 'string' ? selected : (selected as any).path;
            }
            if (path) await loadDocument(path);
        } catch (error) {
            console.error('Failed handling open dialog:', error);
            notifyError('Failed to open document', error);
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
                notifyError('Failed to load template', error);
                throw error;
            } finally {
                setIsLoading(false);
            }
        } catch (error) {
            console.error('Failed handling template open dialog:', error);
            notifyError('Failed to open template', error);
            throw error;
        }
    };

    const handleSave = async (background = false) => {
        if (!currentPath || !currentContent) return handleSaveAs();

        if (background) markSaving(); else setIsLoading(true);

        try {
            if (session && currentPath) {
                await session.saveToOriginal({
                    content: currentContent,
                    styles,
                    metadata,
                });
            } else {
                // No active session: serialize and write directly.
                // save_document returns bytes for content:// paths instead of
                // writing to disk; writeFile (plugin-fs) handles both cases.
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
            notifyError('Failed to save', error);
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
            if (bytes && path.startsWith('content://')) {
                // content:// URI (Android): backend returned bytes instead of writing
                // to disk. Persist the permission before writing.
                try {
                    await takePersistableUriPermission(path);
                } catch {
                    // Non-fatal: swallow on desktop and non-persistable URIs.
                }
                await writeFile(path, bytes);
            }

            // Update app state regardless of whether bytes were returned.
            // For non-content:// (desktop) paths, saveDocument writes to disk and
            // returns null — state must still be updated.
            setPath(path);
            await endSession();
            await startSession(path);
            addDocument({
                path,
                name: metadata.title || path.split('/').pop() || 'Untitled',
                type: 'text',
            });
            markClean();
        } catch (error) {
            console.error('Failed handling save as dialog:', error);
            notifyError('Failed to save document', error);
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
        handleExportPDF,
        loadDocument,
        isLoading,
    };
}
