import { useEffect, useRef, useState } from 'react';
import { X, FileText, Save, FolderOpen, Cloud, CloudOff, Palette, FileOutput } from 'lucide-react';
import { useVectorStore } from '@/lib/vector/store';
import { VectorCanvas } from './Canvas/VectorCanvas';
import { ToolPalette } from './Tools/ToolPalette';
import { PropertiesPanel } from './Properties/PropertiesPanel';
import { NewDocumentDialog } from './Dialogs/NewDocumentDialog';
import { ColourModeDialog } from './Dialogs/ColourModeDialog';
import { ExportPdfDialog } from './Dialogs/ExportPdfDialog';
import { SoftProofToggle } from './SoftProofToggle';
import { Button } from '../ui/button';
import { useVectorFileOps } from '@/lib/vector/useVectorFileOps';
import { useSoftProof } from '@/lib/vector/useSoftProof';

interface VectorEditorProps {
    onClose?: () => void;
}

export function VectorEditor({ onClose }: VectorEditorProps) {
    const { document: doc, isDirty, setSelectedIds, deleteSelected } = useVectorStore();
    const [newDocDialogOpen, setNewDocDialogOpen] = useState(!doc);
    const [colourModeOpen, setColourModeOpen] = useState(false);
    const [exportPdfOpen, setExportPdfOpen] = useState(false);
    const softProofOverrides = useSoftProof();
    const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });
    const containerRef = useRef<HTMLDivElement>(null);
    const { handleSave, handleOpen } = useVectorFileOps();

    // ResizeObserver to track canvas container size
    useEffect(() => {
        const el = containerRef.current;
        if (!el) return;
        const observer = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                setContainerSize({ width: Math.floor(width), height: Math.floor(height) });
            }
        });
        observer.observe(el);
        return () => observer.disconnect();
    }, []);

    // Keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            const isMod = e.ctrlKey || e.metaKey;
            if (e.key === 'Delete' || e.key === 'Backspace') {
                if (!(e.target instanceof HTMLInputElement)) {
                    deleteSelected();
                }
            }
            if (e.key === 'Escape') setSelectedIds(new Set());
            if (isMod && e.key === 's') { e.preventDefault(); handleSave(); }
            if (isMod && e.key === 'z') { e.preventDefault(); console.log('[VectorEditor] Undo stub'); }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [deleteSelected, setSelectedIds, handleSave]);

    const title = doc?.metadata.title as string | null ?? 'Untitled Vector';

    return (
        <div className="flex flex-col h-full bg-background overflow-hidden">
            {/* Top bar — outer div owns safe-pt so the bar grows to cover the status bar */}
            <div className="shrink-0 bg-background border-b border-border safe-pt">
                <div className="flex items-center gap-2 px-3 h-10">
                    <span className="text-sm font-medium truncate flex-1">{title}</span>
                    <span title={isDirty ? 'Unsaved changes' : 'Saved'}>
                        {isDirty ? (
                            <CloudOff className="h-3.5 w-3.5 text-muted-foreground" />
                        ) : (
                            <Cloud className="h-3.5 w-3.5 text-muted-foreground" />
                        )}
                    </span>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setNewDocDialogOpen(true)} title="New">
                        <FileText className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handleOpen} title="Open">
                        <FolderOpen className="h-4 w-4" />
                    </Button>
                    <Button variant="ghost" size="icon" className="h-7 w-7" onClick={handleSave} title="Save">
                        <Save className="h-4 w-4" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7"
                        onClick={() => setColourModeOpen(true)}
                        title="Document Colour Mode…"
                        disabled={!doc}
                    >
                        <Palette className="h-4 w-4" />
                    </Button>
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7"
                        onClick={() => setExportPdfOpen(true)}
                        title="Export PDF/X…"
                        disabled={!doc}
                    >
                        <FileOutput className="h-4 w-4" />
                    </Button>
                    <SoftProofToggle className="hidden sm:flex" />
                    {onClose && (
                        <Button variant="ghost" size="icon" className="h-7 w-7" onClick={onClose} title="Close">
                            <X className="h-4 w-4" />
                        </Button>
                    )}
                </div>
            </div>

            {/* Main layout area */}
            <div className="flex-1 min-h-0 flex overflow-hidden">
                {/* Sidebar tool palette — hidden on mobile */}
                <div className="hidden sm:flex">
                    <ToolPalette variant="sidebar" />
                </div>

                {/* Canvas area */}
                <div ref={containerRef} className="flex-1 min-w-0 min-h-0 relative overflow-hidden">
                    {containerSize.width > 0 && containerSize.height > 0 && (
                        <VectorCanvas
                            width={containerSize.width}
                            height={containerSize.height}
                            softProofOverrides={softProofOverrides}
                        />
                    )}
                </div>

                {/* Right properties panel — desktop only */}
                <div className="hidden lg:flex">
                    <PropertiesPanel variant="sidebar" />
                </div>
            </div>

            {/* Mobile bottom bar */}
            <div className="flex sm:hidden flex-col">
                {/* Properties bottom sheet for tablet/mobile */}
                <div className="lg:hidden">
                    <PropertiesPanel variant="bottomsheet" />
                </div>
                {/* Tool palette at bottom on mobile */}
                <div className="safe-pb bg-background">
                    <ToolPalette variant="bottombar" />
                </div>
            </div>

            <NewDocumentDialog
                open={newDocDialogOpen}
                onOpenChange={setNewDocDialogOpen}
            />
            <ColourModeDialog
                open={colourModeOpen}
                onOpenChange={setColourModeOpen}
            />
            <ExportPdfDialog
                open={exportPdfOpen}
                onOpenChange={setExportPdfOpen}
            />
        </div>
    );
}
