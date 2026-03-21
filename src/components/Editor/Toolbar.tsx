import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { UNDO_COMMAND, REDO_COMMAND, CAN_UNDO_COMMAND, CAN_REDO_COMMAND } from 'lexical';
import { INSERT_ORDERED_LIST_COMMAND, INSERT_UNORDERED_LIST_COMMAND } from '@lexical/list';
import { $getSelection, $isRangeSelection } from 'lexical';
import { TOGGLE_LINK_COMMAND, $isLinkNode } from '@lexical/link';
import { mergeRegister } from '@lexical/utils';
import { $createPageBreakNode } from '@/lib/editor/nodes/PageBreakNode';
import { $createParagraphStyleNode } from '@/lib/editor/nodes/ParagraphStyleNode';
import { useState, useEffect, useCallback } from 'react';

import {
    Undo,
    Redo,
    ClipboardPaste,
    List,
    ListOrdered,
    PlusSquare,
    PencilRuler,
    Link as LinkIcon
} from 'lucide-react';
import { INSERT_TABLE_COMMAND } from '@lexical/table';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { LinkDialog } from '../Dialogs/LinkDialog';
import { ImageDialog } from '../Dialogs/ImageDialog';
import { INSERT_IMAGE_COMMAND } from './plugins/ImagePlugin';
import { ToolbarStyleSelector } from './ToolbarStyleSelector';

import type { StyleDefinition } from '@/lib/types/odt';

interface ToolbarProps {
    styles: Record<string, StyleDefinition>;
    currentStyle?: string;
    onStyleChange?: (styleName: string) => void;
    onStylesClick?: () => void;
}

export function Toolbar({ styles, currentStyle, onStyleChange, onStylesClick }: ToolbarProps) {
    const [editor] = useLexicalComposerContext();
    const [canUndo, setCanUndo] = useState(false);
    const [canRedo, setCanRedo] = useState(false);
    const [isStylePopoverOpen, setIsStylePopoverOpen] = useState(false);
    const [isLinkDialogOpen, setIsLinkDialogOpen] = useState(false);
    const [isImageDialogOpen, setIsImageDialogOpen] = useState(false);
    const [isLink, setIsLink] = useState(false);
    const [linkUrl, setLinkUrl] = useState("");

    useEffect(() => {
        return mergeRegister(
            editor.registerCommand(
                CAN_UNDO_COMMAND,
                (payload) => {
                    setCanUndo(payload);
                    return false;
                },
                1
            ),
            editor.registerCommand(
                CAN_REDO_COMMAND,
                (payload) => {
                    setCanRedo(payload);
                    return false;
                },
                1
            ),
            editor.registerUpdateListener(({ editorState }) => {
                editorState.read(() => {
                    const selection = $getSelection();
                    if ($isRangeSelection(selection)) {
                        const node = selection.getNodes()[0];
                        if (node) {
                            const parent = node.getParent();
                            if ($isLinkNode(parent)) {
                                setIsLink(true);
                                setLinkUrl(parent.getURL());
                            } else if ($isLinkNode(node)) {
                                setIsLink(true);
                                setLinkUrl(node.getURL());
                            } else {
                                setIsLink(false);
                                setLinkUrl("");
                            }
                        }
                    }
                });
            })
        );
    }, [editor]);

    const handleUndo = useCallback(() => {
        editor.dispatchCommand(UNDO_COMMAND, undefined);
    }, [editor]);

    const handleRedo = useCallback(() => {
        editor.dispatchCommand(REDO_COMMAND, undefined);
    }, [editor]);

    const handlePaste = useCallback(async () => {
        try {
            const text = await navigator.clipboard.readText();
            document.execCommand('insertText', false, text);
        } catch (err) {
            console.error('Failed to read clipboard contents: ', err);
        }
    }, []);

    const handleInsertPageBreak = useCallback(() => {
        editor.update(() => {
            const selection = $getSelection();
            if ($isRangeSelection(selection)) {
                const pbNode = $createPageBreakNode();
                selection.insertNodes([pbNode]);
                const pNode = $createParagraphStyleNode(null);
                pbNode.insertAfter(pNode);
                pNode.select();
            }
        });
    }, [editor]);

    const handleInsertLineBreak = useCallback(() => {
        // Basic soft newline usually Shift+Enter, handled natively but adding here for UI completeness
        document.execCommand('insertLineBreak');
    }, []);

    const handleInsertTable = useCallback(() => {
        editor.dispatchCommand(INSERT_TABLE_COMMAND, {
            columns: '3',
            rows: '3',
            includeHeaders: false,
        });
    }, [editor]);

    const handleSaveLink = useCallback((url: string | null) => {
        editor.dispatchCommand(TOGGLE_LINK_COMMAND, url);
    }, [editor]);

    const handleSaveImage = useCallback((src: string, alt: string) => {
        editor.dispatchCommand(INSERT_IMAGE_COMMAND, { src, altText: alt });
    }, [editor]);

    return (
        <div className="bottom-toolbar border-t bg-sky-100 dark:bg-blue-950 shrink-0 text-foreground overflow-x-auto no-scrollbar safe-pb">
            <div className="flex items-center justify-start gap-1 p-2 min-w-max px-4">

                {/* Undo / Redo */}
                <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 text-slate-700 dark:text-slate-200 disabled:opacity-30"
                    onClick={handleUndo}
                    disabled={!canUndo}
                    title="Undo (Ctrl+Z)"
                >
                    <Undo className="h-4 w-4" />
                </Button>
                <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 text-slate-700 dark:text-slate-200 disabled:opacity-30"
                    onClick={handleRedo}
                    disabled={!canRedo}
                    title="Redo (Ctrl+Y)"
                >
                    <Redo className="h-4 w-4" />
                </Button>

                {/* Paste */}
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200 ml-1" onClick={handlePaste} title="Paste (Ctrl+V)">
                    <ClipboardPaste className="h-4 w-4" />
                </Button>

                <div className="h-6 w-px bg-gray-300 mx-1" />

                {/* Style Selector */}
                <ToolbarStyleSelector
                    styles={styles}
                    currentStyle={currentStyle}
                    onStyleChange={onStyleChange}
                    isOpen={isStylePopoverOpen}
                    onOpenChange={setIsStylePopoverOpen}
                />

                <Button
                    variant="ghost"
                    size="icon"
                    className="h-8 w-8 text-slate-700 dark:text-slate-200 ml-1"
                    onClick={onStylesClick}
                    title="Edit Style"
                >
                    <PencilRuler className="h-4 w-4" />
                </Button>

                <div className="h-6 w-px bg-gray-300 mx-2" />

                {/* Link Toggle */}
                <Button
                    variant="ghost"
                    size="icon"
                    className={cn("h-8 w-8 text-slate-700 dark:text-slate-200 mr-2", isLink && "bg-slate-200 dark:bg-slate-800")}
                    onClick={() => setIsLinkDialogOpen(true)}
                    title="Insert/Edit Link"
                >
                    <LinkIcon className="h-4 w-4" />
                </Button>

                {/* Insert Menu */}
                <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm" className="h-8 text-slate-700 dark:text-slate-200 px-2 outline-none">
                            <PlusSquare className="h-4 w-4 mr-1" /> Insert
                        </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={handleInsertPageBreak}>
                            Page Break
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={handleInsertLineBreak}>
                            Line Break
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={handleInsertTable}>
                            Table (3x3)
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => setIsImageDialogOpen(true)}>
                            Image...
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>

                <div className="h-6 w-px bg-gray-300 mx-1" />

                {/* List Controls */}
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200" onClick={() => editor.dispatchCommand(INSERT_UNORDERED_LIST_COMMAND, undefined)} title="Bullet List">
                    <List className="h-4 w-4" />
                </Button>
                <Button variant="ghost" size="icon" className="h-8 w-8 text-slate-700 dark:text-slate-200" onClick={() => editor.dispatchCommand(INSERT_ORDERED_LIST_COMMAND, undefined)} title="Numbered List">
                    <ListOrdered className="h-4 w-4" />
                </Button>

            </div>

            <LinkDialog
                open={isLinkDialogOpen}
                onOpenChange={setIsLinkDialogOpen}
                initialUrl={linkUrl}
                onSave={handleSaveLink}
            />

            <ImageDialog
                open={isImageDialogOpen}
                onOpenChange={setIsImageDialogOpen}
                onSave={handleSaveImage}
            />
        </div>
    );
}
