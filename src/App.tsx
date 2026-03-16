import { useState, useEffect } from 'react';
import { Editor } from './components/Editor/Editor';
import { StyleDialog } from './components/Dialogs/StyleDialog';
import { MetadataDialog } from './components/Dialogs/MetadataDialog';
import { LandingPage } from './components/LandingPage';
import { TopBar } from './components/TopBar';
import { FileTypeDialog, FileType } from './components/Dialogs/FileTypeDialog';
import { VectorEditor } from './components/VectorEditor/VectorEditor';
import { listen } from '@tauri-apps/api/event';
import { useDocumentStore } from './lib/stores/documentStore';
import { useFileOperations } from './lib/hooks/useFileOperations';
import { useSessionPersistence } from './lib/hooks/useSessionPersistence';
import { useAutoSave } from './lib/hooks/useAutoSave';
import { LoadingOverlay } from './components/ui/LoadingOverlay';
import { Toaster } from './components/ui/toaster';
import { useWindowTitle } from './lib/hooks/useWindowTitle';

export default function App() {
    const [styleDialogOpen, setStyleDialogOpen] = useState(false);
    const [metadataDialogOpen, setMetadataDialogOpen] = useState(false);
    const [fileTypeDialogOpen, setFileTypeDialogOpen] = useState(false);
    const [showVectorEditor, setShowVectorEditor] = useState(false);

    const {
        currentContent,
        styles,
        metadata,
        currentStyle,
        setContent,
        setStyles,
        setMetadata,
        restoreState,
    } = useDocumentStore();

    const { loadSession } = useSessionPersistence();

    const {
        handleOpen,
        handleOpenTemplate,
        handleSave,
        handleSaveAs,
        handleNew,
        handleClose,
        handleExportEPUB,
        handleExportPDF,
        loadDocument,
        isLoading
    } = useFileOperations();

    // Enable auto-save (saves every 15 seconds if dirty)
    useAutoSave({ intervalMs: 15000 });

    useWindowTitle();

    // Restore session on mount
    useEffect(() => {
        const session = loadSession();
        if (session) {
            console.log('[App] Restoring session from localStorage');
            restoreState(session);
        }
    }, [loadSession, restoreState]);

    // Tauri Menu Events and Keyboard shortcuts
    useEffect(() => {
        const withErrorHandling = (fn: () => Promise<any>, errorMessage: string) => async () => {
            try {
                await fn();
            } catch (err) {
                console.error(errorMessage, err);
                alert(`${errorMessage}\n\n${err instanceof Error ? err.message : String(err)}`);
            }
        };

        const unlistenPromises = [
            listen('menu-new', withErrorHandling(handleNew, "Failed to create new document")),
            listen('menu-open', withErrorHandling(handleOpen, "Failed to open document")),
            listen('menu-save', withErrorHandling(handleSave, "Failed to save document")),
            listen('menu-save-as', () => setFileTypeDialogOpen(true)),
            listen('menu-export-epub', withErrorHandling(handleExportEPUB, "Failed to export EPUB")),
            listen('menu-export-pdf', withErrorHandling(handleExportPDF, "Failed to export PDF/X")),
            listen('menu-close', () => handleClose()),
        ];

        const handleKeyDown = (e: KeyboardEvent) => {
            const isMod = e.ctrlKey || e.metaKey;

            if (isMod && e.key === 'n') {
                e.preventDefault();
                withErrorHandling(handleNew, "Failed to create new document")();
            }
            if (isMod && e.key === 'o') {
                e.preventDefault();
                withErrorHandling(handleOpen, "Failed to open document")();
            }
            if (isMod && e.key === 's') {
                e.preventDefault();
                if (e.shiftKey) {
                    withErrorHandling(() => handleSaveAs(), "Failed to save document")();
                } else {
                    withErrorHandling(handleSave, "Failed to save document")();
                }
            }
            if (isMod && e.key === 'w') {
                e.preventDefault();
                handleClose();
            }
        };

        window.addEventListener('keydown', handleKeyDown);

        return () => {
            unlistenPromises.forEach(async (p) => (await p)());
            window.removeEventListener('keydown', handleKeyDown);
        };
    }, [handleNew, handleOpen, handleSave, handleSaveAs, handleClose, handleExportEPUB, handleExportPDF]);

    // Visual Viewport handling for mobile keyboard
    useEffect(() => {
        if (!window.visualViewport) return;

        const handleResize = () => {
            if (window.visualViewport) {
                document.documentElement.style.setProperty(
                    '--viewport-height',
                    `${window.visualViewport.height}px`
                );

                // Nuclear fix for Android panning: 
                // Force scroll to 0,0 after a tiny delay to override native browser "helpfulness"
                setTimeout(() => {
                    window.scrollTo(0, 0);
                    if (document.body) document.body.scrollTop = 0;
                }, 50);
            }
        };

        window.visualViewport.addEventListener('resize', handleResize);
        handleResize(); // Initial call

        return () => window.visualViewport?.removeEventListener('resize', handleResize);
    }, []);

    // Mobile Back Button handling
    useEffect(() => {
        const handleBack = () => {
            // If we have content open, "back" should close it (go to Landing Page)
            if (currentContent) {
                // Prevent default if possible to stop app from exiting
                // Note: Standard web 'backbutton' event or popstate
                handleClose();
            }
        };

        window.addEventListener('popstate', handleBack);
        // Specialized event for some mobile environments
        document.addEventListener('backbutton', handleBack);

        return () => {
            window.removeEventListener('popstate', handleBack);
            document.removeEventListener('backbutton', handleBack);
        };
    }, [currentContent, handleClose]);

    return (
        <div
            className="app flex flex-col font-sans overflow-hidden bg-background text-foreground"
            style={{ height: 'var(--viewport-height, 100vh)' }}
        >
            {/* Conditional Top bar based on empty state */}
            {currentContent && (
                <TopBar
                    onNew={handleNew}
                    onOpen={handleOpen}
                    onSave={handleSave}
                    onSaveAs={() => setFileTypeDialogOpen(true)}
                    onClose={handleClose}
                    onExportEPUB={handleExportEPUB}
                    onExportPDF={handleExportPDF}
                    isLoading={isLoading}
                    onMetadataClick={() => setMetadataDialogOpen(true)}
                />
            )}

            {/* Editor Area / Landing Page */}
            <div className="flex-1 overflow-hidden w-full min-h-0">
                {showVectorEditor ? (
                    <VectorEditor onClose={() => setShowVectorEditor(false)} />
                ) : currentContent ? (
                    <Editor
                        key={metadata.identifier || 'doc'}  // Ensure Editor remounts on new documents
                        initialContent={JSON.stringify(currentContent)}
                        styles={styles}
                        onContentChange={(str) => {
                            setContent(JSON.parse(str));
                        }}
                        onStylesClick={() => setStyleDialogOpen(true)}
                    />
                ) : (
                    <LandingPage
                        onOpenClick={handleOpen}
                        onOpenTemplateClick={handleOpenTemplate}
                        onNewClick={handleNew}
                        loadDocument={loadDocument}
                        onNewVector={() => setShowVectorEditor(true)}
                    />
                )}
            </div>

            {/* Dialogs */}
            <StyleDialog
                open={styleDialogOpen}
                onOpenChange={setStyleDialogOpen}
                existingStyles={styles}
                onSave={(style) => {
                    setStyles({ ...styles, [style.name]: style });
                }}
                initialStyleName={currentStyle}
            />

            <MetadataDialog
                open={metadataDialogOpen}
                onOpenChange={setMetadataDialogOpen}
                metadata={metadata}
                onSave={setMetadata}
            />

            <FileTypeDialog
                open={fileTypeDialogOpen}
                onOpenChange={setFileTypeDialogOpen}
                onSelect={(type: FileType) => {
                    const runSaveAs = async () => {
                        try {
                            await handleSaveAs(type);
                        } catch (err) {
                            console.error("Failed to save as:", err);
                            alert(`Failed to save document\n\n${err instanceof Error ? err.message : String(err)}`);
                        }
                    };
                    runSaveAs();
                }}
            />

            {isLoading && <LoadingOverlay />}
            <Toaster />
        </div>
    );
}
