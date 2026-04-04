import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { INSERT_ORDERED_LIST_COMMAND, INSERT_UNORDERED_LIST_COMMAND } from '@lexical/list';
import { INSERT_FOOTNOTE_COMMAND } from '@/editor/commands/footnoteCommands';
import { useDocumentStore } from '@/lib/stores/documentStore';
import { useTranslation } from 'react-i18next';
import { useToolbarState } from './useToolbarState';

import {
    Undo, Redo, ClipboardPaste, List, ListOrdered,
    PlusSquare, PencilRuler, Link as LinkIcon,
    Eye, Pencil, Settings,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import {
    DropdownMenu, DropdownMenuContent,
    DropdownMenuItem, DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { LinkDialog } from '../Dialogs/LinkDialog';
import { ImageDialog } from '../Dialogs/ImageDialog';
import { ToolbarStyleSelector } from './ToolbarStyleSelector';
import { PageStyleDialog } from '@/editor/dialogs/PageStyleDialog';

import type { StyleDefinition } from '@/lib/types/odt';

interface ToolbarProps {
    styles: Record<string, StyleDefinition>;
    currentStyle?: string;
    onStyleChange?: (styleName: string) => void;
    onStylesClick?: () => void;
}

export function Toolbar({ styles, currentStyle, onStyleChange, onStylesClick }: ToolbarProps) {
    const { t } = useTranslation('common');
    const [editor] = useLexicalComposerContext();
    const { viewMode, setViewMode } = useDocumentStore();
    const isPreview = viewMode === 'preview';

    const {
        canUndo, canRedo, isLink, linkUrl,
        isStylePopoverOpen, setIsStylePopoverOpen,
        isLinkDialogOpen, setIsLinkDialogOpen,
        isImageDialogOpen, setIsImageDialogOpen,
        isPageStyleOpen, setIsPageStyleOpen,
        handleUndo, handleRedo, handlePaste,
        handleInsertPageBreak, handleInsertLineBreak, handleInsertTable,
        handleSaveLink, handleSaveImage,
    } = useToolbarState(editor);

    return (
        <div className="bottom-toolbar border-t bg-sky-100 dark:bg-blue-950 shrink-0 text-foreground overflow-x-auto no-scrollbar safe-pb">
            <div className="flex items-center justify-start gap-1 p-2 min-w-max px-4">

                {/* Undo / Redo */}
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200 disabled:opacity-30"
                    onClick={handleUndo} disabled={isPreview || !canUndo} title="Undo (Ctrl+Z)">
                    <Undo className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200 disabled:opacity-30"
                    onClick={handleRedo} disabled={isPreview || !canRedo} title="Redo (Ctrl+Y)">
                    <Redo className="h-4 w-4" />
                </Button>

                {/* Paste */}
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200 ml-1"
                    onClick={handlePaste} disabled={isPreview} title="Paste (Ctrl+V)">
                    <ClipboardPaste className="h-4 w-4" />
                </Button>

                <div className="h-6 w-px bg-gray-300 mx-1" />

                {/* Style Selector */}
                <ToolbarStyleSelector
                    styles={styles}
                    currentStyle={currentStyle}
                    onStyleChange={onStyleChange}
                    isOpen={isStylePopoverOpen && !isPreview}
                    onOpenChange={(open) => !isPreview && setIsStylePopoverOpen(open)}
                />
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200 ml-1"
                    onClick={onStylesClick} disabled={isPreview} title="Edit Style">
                    <PencilRuler className="h-4 w-4" />
                </Button>

                <div className="h-6 w-px bg-gray-300 mx-2" />

                {/* Link */}
                <Button variant="ghost" size="icon"
                    className={cn('h-8 w-8 text-slate-700 dark:text-slate-200 mr-2', isLink && 'bg-slate-200 dark:bg-slate-800')}
                    onClick={() => setIsLinkDialogOpen(true)} disabled={isPreview} title="Insert/Edit Link">
                    <LinkIcon className="h-4 w-4" />
                </Button>

                {/* Insert Menu */}
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm" className="h-8 text-slate-700 dark:text-slate-200 px-2 outline-none" disabled={isPreview}>
                            <PlusSquare className="h-4 w-4 mr-1" /> Insert
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={handleInsertPageBreak}>Page Break</DropdownMenuItem>
                        <DropdownMenuItem onClick={handleInsertLineBreak}>Line Break</DropdownMenuItem>
                        <DropdownMenuItem onClick={handleInsertTable}>Table (3x3)</DropdownMenuItem>
                        <DropdownMenuItem onClick={() => setIsImageDialogOpen(true)}>Image...</DropdownMenuItem>
                        <DropdownMenuItem onClick={() => editor.dispatchCommand(INSERT_FOOTNOTE_COMMAND, undefined)}>
                            Footnote
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>

                <div className="h-6 w-px bg-gray-300 mx-1" />

                {/* Lists */}
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200"
                    onClick={() => editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined)}
                    disabled={isPreview} title="Bullet List">
                    <List className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200"
                    onClick={() => editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined)}
                    disabled={isPreview} title="Numbered List">
                    <ListOrdered className="h-4 w-4" />
                </Button>

                <div className="h-6 w-px bg-gray-300 mx-1" />

                {/* Page Style */}
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200"
                    onClick={() => setIsPageStyleOpen(true)} disabled={isPreview} title="Page Style">
                    <Settings className="h-4 w-4" />
                </Button>

                <div className="h-6 w-px bg-gray-300 mx-1" />

                {/* Preview / Edit toggle */}
                <Button
                    variant={isPreview ? 'secondary' : 'ghost'}
                    size="sm"
                    className="h-8 px-2 text-slate-700 dark:text-slate-200 gap-1"
                    onClick={() => setViewMode(isPreview ? 'scroll' : 'preview')}
                    title={isPreview ? t('toolbar.editMode') : t('toolbar.previewMode')}
                >
                    {isPreview
                        ? <><Pencil className="h-4 w-4" /><span className="hidden sm:inline">{t('toolbar.editMode')}</span></>
                        : <><Eye className="h-4 w-4" /><span className="hidden sm:inline">{t('toolbar.previewMode')}</span></>
                    }
                </Button>

            </div>

            <LinkDialog open={isLinkDialogOpen} onOpenChange={setIsLinkDialogOpen} initialUrl={linkUrl} onSave={handleSaveLink} />
            <ImageDialog open={isImageDialogOpen} onOpenChange={setIsImageDialogOpen} onSave={handleSaveImage} />
            <PageStyleDialog open={isPageStyleOpen} onOpenChange={setIsPageStyleOpen} />
        </div>
    );
}
