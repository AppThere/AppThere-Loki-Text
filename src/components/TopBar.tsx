import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import { FolderOpen, Save, FileDown, PencilLine, Menu, XCircle, Share } from 'lucide-react';
import { useDocumentStore } from '@/lib/stores/documentStore';
import { SaveIndicator } from '@/components/SaveIndicator';

interface TopBarProps {
    onOpen: () => void;
    onNew: () => void;
    onSave: () => void;
    onSaveAs: () => void;
    onClose: () => void;
    onExportEPUB: () => void;
    isLoading: boolean;
    onMetadataClick: () => void;
}

export function TopBar({ onOpen, onNew, onSave, onSaveAs, onClose, onExportEPUB, isLoading, onMetadataClick }: TopBarProps) {
    const { currentContent, currentPath, metadata } = useDocumentStore();
    const hasContent = !!currentContent;

    const currentTitle = metadata.title || (currentPath ? currentPath.split('/').pop() : 'Untitled Document');

    return (
        <div className="flex items-center justify-between px-4 py-2 bg-sky-100 dark:bg-blue-950 border-b select-none text-foreground safe-pt">
            {/* Title & Metadata Left */}
            <div className="flex items-center justify-start gap-2 overflow-hidden mr-4">
                {hasContent && (
                    <>
                        <h2 className="text-sm font-semibold truncate max-w-[200px] text-slate-800 dark:text-slate-200">
                            {currentTitle}
                        </h2>
                        <SaveIndicator />
                        <Button variant="ghost" size="sm" onClick={onMetadataClick} title="Edit Metadata" className="h-8 w-8 p-0 ml-1">
                            <PencilLine className="h-4 w-4" />
                        </Button>
                    </>
                )}
            </div>

            {/* Menu Right */}
            <div className="flex items-center justify-end">
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm" className="px-2" title="File Menu">
                            <Menu className="h-5 w-5" />
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end" className="w-48">
                        <DropdownMenuItem onClick={onNew}>
                            New Document
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={onOpen} disabled={isLoading}>
                            <FolderOpen className="mr-2 h-4 w-4" />
                            <span>Open...</span>
                        </DropdownMenuItem>

                        <DropdownMenuSeparator />

                        <DropdownMenuItem onClick={onSave} disabled={isLoading || !hasContent}>
                            <Save className="mr-2 h-4 w-4" />
                            <span>Save</span>
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={onSaveAs} disabled={isLoading || !hasContent}>
                            <FileDown className="mr-2 h-4 w-4" />
                            <span>Save As...</span>
                        </DropdownMenuItem>

                        <DropdownMenuSeparator />

                        <DropdownMenuItem onClick={onExportEPUB} disabled={isLoading || !hasContent}>
                            <Share className="mr-2 h-4 w-4" />
                            <span>Export to EPUB</span>
                        </DropdownMenuItem>

                        <DropdownMenuSeparator />

                        <DropdownMenuItem onClick={onClose} className="text-red-600 focus:text-red-700">
                            <XCircle className="mr-2 h-4 w-4" />
                            <span>Close Document</span>
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </div>
        </div>
    );
}
