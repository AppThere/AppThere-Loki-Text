import { useState } from 'react';
import { open, save } from '@tauri-apps/plugin-dialog';
import { readFile, writeFile } from '@tauri-apps/plugin-fs';
import { openDocument, saveDocument, saveEpub } from '../tauri/commands';
import { useDocumentStore } from '../stores/documentStore';
import { useHistoryStore } from '../stores/historyStore';
import { useSessionPersistence } from './useSessionPersistence';
import { FileType } from '@/components/Dialogs/FileTypeDialog';
import standardTemplate from '@/assets/templates/standard.fodt?raw';

let originalFileBytes: Uint8Array | null = null;

export function useFileOperations() {
    const [isLoading, setIsLoading] = useState(false);

    const {
        currentPath,
        currentContent,
        styles,
        metadata,
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

    const handleNew = async () => {
        setIsLoading(true);
        try {
            clearSession();

            // Generate bytes from the raw template string
            const templateBytes = new TextEncoder().encode(standardTemplate);

            // Use openDocument to process the template
            const response = await openDocument('internal://standard.fodt', templateBytes);

            setPath('');
            originalFileBytes = null;

            setContent(response.content);
            setStyles(response.styles);
            // Default to 'Untitled Document' for NEW documents from standard template
            setMetadata({ ...response.metadata, title: 'Untitled Document', identifier: undefined });
            markDirty();
        } catch (error) {
            console.error("Failed to create new document from template:", error);
            throw error;
        } finally {
            setIsLoading(false);
        }
    };

    const handleClose = () => {
        clearSession();
        resetStore();
        originalFileBytes = null;
    };

    const loadDocument = async (path: string) => {
        setIsLoading(true);
        try {
            const fileBytes = await readFile(path);
            originalFileBytes = fileBytes;

            const response = await openDocument(path, fileBytes);

            setPath(path);
            setContent(response.content);
            setStyles(response.styles);
            setMetadata(response.metadata);
            addDocument({
                path,
                name: response.metadata.title || path.split('/').pop() || 'Untitled',
                type: 'text'
            });
            markClean();
        } catch (error) {
            console.error("Failed to load document:", error);
            throw error;
        } finally {
            setIsLoading(false);
        }
    };

    const handleOpen = async () => {
        try {
            const selected = await open({
                title: 'Open AppThere Document',
                filters: [{ name: 'Document', extensions: ['odt', 'fodt'] }]
            });

            if (selected) {
                const path = typeof selected === 'string' ? selected : (selected as any).path;
                if (path) {
                    clearSession(); // Added clearSession
                    await loadDocument(path);
                }
            }
        } catch (error) {
            console.error("Failed handling open dialog:", error);
        }
    };

    const handleOpenTemplate = async () => {
        try {
            const selected = await open({
                title: 'New Document from Template',
                filters: [{ name: 'Template', extensions: ['ott', 'odt', 'fodt'] }]
            });

            if (selected) {
                const path = typeof selected === 'string' ? selected : (selected as any).path;
                if (path) {
                    clearSession(); // Added clearSession
                    setIsLoading(true);
                    try {
                        const fileBytes = await readFile(path);
                        const response = await openDocument(path, fileBytes);

                        // Treat as new unsaved document
                        setPath('');
                        originalFileBytes = null;

                        setContent(response.content);
                        setStyles(response.styles);
                        setMetadata({ ...response.metadata, title: 'Untitled Document', identifier: undefined });
                        addTemplate('text', {
                            path,
                            name: response.metadata.title || path.split('/').pop() || 'Untitled',
                            type: 'text'
                        });
                        markDirty(); // Unsaved immediately
                    } catch (error) {
                        console.error("Failed to load template:", error);
                        throw error;
                    } finally {
                        setIsLoading(false);
                    }
                }
            }
        } catch (error) {
            console.error("Failed handling template open dialog:", error);
            throw error;
        }
    };

    const handleSave = async (background: boolean = false) => {
        if (!currentPath || !currentContent) {
            return handleSaveAs();
        }

        if (background) {
            markSaving();
        } else {
            setIsLoading(true);
        }

        try {
            const newBytes = await saveDocument(
                currentPath,
                JSON.stringify(currentContent),
                styles,
                metadata,
                currentPath,
                originalFileBytes || undefined
            );

            if (newBytes) {
                if (currentPath.startsWith('content://')) {
                    await writeFile(currentPath, newBytes);
                }
                originalFileBytes = newBytes;

                if (background) {
                    markSaved();
                } else {
                    markClean();
                }
            } else if (background) {
                // If it aborted or failed quietly
                markClean();
            }
        } catch (error) {
            console.error("Failed to save:", error);
            if (background) {
                markClean(); // Don't leave in saving state
            }
            throw error;
        } finally {
            if (!background) {
                setIsLoading(false);
            }
        }
    };

    const handleSaveAs = async (explicitType?: FileType) => {
        if (!currentContent) return;

        try {
            // Sanitize title for filename
            const cleanTitle = (metadata.title || 'Untitled')
                .replace(/[<>:"/\\|?*]/g, '_') // Remove illegal chars
                .trim();

            const ext = explicitType || 'odt';
            const defaultPath = `${cleanTitle}.${ext}`;

            const selected = await save({
                title: 'Save AppThere Document As',
                defaultPath,
                filters: explicitType
                    ? [{ name: explicitType === 'odt' ? 'ODT Document' : 'Flat XML ODT', extensions: [ext] }]
                    : [{ name: 'Document', extensions: ['odt', 'fodt'] }]
            });

            if (selected) {
                setIsLoading(true);
                const path = typeof selected === 'string' ? selected : (selected as any).path;
                if (!path) return;

                const newBytes = await saveDocument(
                    path,
                    JSON.stringify(currentContent),
                    styles,
                    metadata,
                    currentPath || undefined,
                    originalFileBytes || undefined
                );

                if (newBytes) {
                    if (path.startsWith('content://')) {
                        await writeFile(path, newBytes);
                    }
                    originalFileBytes = newBytes;
                    setPath(path);
                    addDocument({
                        path,
                        name: metadata.title || path.split('/').pop() || 'Untitled',
                        type: 'text'
                    });
                    markClean();
                }
            }
        } catch (error) {
            console.error("Failed handling save as dialog:", error);
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
                filters: [{ name: 'EPUB Ebook', extensions: ['epub'] }]
            });

            if (selected) {
                setIsLoading(true);
                const path = typeof selected === 'string' ? selected : (selected as any).path;
                if (!path) return;

                // For now, we don't have a sophisticated font locator,
                // but we can pass names or empty list if fonts are bundled in the binary
                const fontPaths: string[] = [];

                const newBytes = await saveEpub(
                    path,
                    JSON.stringify(currentContent),
                    styles,
                    metadata,
                    fontPaths
                );

                if (newBytes && path.startsWith('content://')) {
                    await writeFile(path, newBytes);
                }
                // Don't mark clean as EPUB is an export, not the source file
            }
        } catch (error) {
            console.error("Failed to export EPUB:", error);
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
        isLoading
    };
}
