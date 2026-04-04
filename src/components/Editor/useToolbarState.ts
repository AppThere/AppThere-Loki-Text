// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useState, useEffect, useCallback } from 'react';
import type { LexicalEditor } from 'lexical';
import {
    UNDO_COMMAND,
    REDO_COMMAND,
    CAN_UNDO_COMMAND,
    CAN_REDO_COMMAND,
    $getSelection,
    $isRangeSelection,
} from 'lexical';
import { TOGGLE_LINK_COMMAND, $isLinkNode } from '@lexical/link';
import { INSERT_TABLE_COMMAND } from '@lexical/table';
import { mergeRegister, $insertNodeToNearestRoot } from '@lexical/utils';
import { INSERT_ORDERED_LIST_COMMAND, INSERT_UNORDERED_LIST_COMMAND } from '@lexical/list';
import { $createPageBreakNode } from '@/lib/editor/nodes/PageBreakNode';
import { $createParagraphStyleNode } from '@/lib/editor/nodes/ParagraphStyleNode';
import { INSERT_IMAGE_COMMAND } from './plugins/ImagePlugin';

export interface ToolbarState {
    canUndo: boolean;
    canRedo: boolean;
    isLink: boolean;
    linkUrl: string;
    isStylePopoverOpen: boolean;
    setIsStylePopoverOpen: (open: boolean) => void;
    isLinkDialogOpen: boolean;
    setIsLinkDialogOpen: (open: boolean) => void;
    isImageDialogOpen: boolean;
    setIsImageDialogOpen: (open: boolean) => void;
    isPageStyleOpen: boolean;
    setIsPageStyleOpen: (open: boolean) => void;
    handleUndo: () => void;
    handleRedo: () => void;
    handlePaste: () => Promise<void>;
    handleInsertPageBreak: () => void;
    handleInsertLineBreak: () => void;
    handleInsertTable: () => void;
    handleSaveLink: (url: string | null) => void;
    handleSaveImage: (src: string, alt: string) => void;
}

export function useToolbarState(editor: LexicalEditor): ToolbarState {
    const [canUndo, setCanUndo] = useState(false);
    const [canRedo, setCanRedo] = useState(false);
    const [isStylePopoverOpen, setIsStylePopoverOpen] = useState(false);
    const [isLinkDialogOpen, setIsLinkDialogOpen] = useState(false);
    const [isImageDialogOpen, setIsImageDialogOpen] = useState(false);
    const [isPageStyleOpen, setIsPageStyleOpen] = useState(false);
    const [isLink, setIsLink] = useState(false);
    const [linkUrl, setLinkUrl] = useState('');

    useEffect(() => {
        return mergeRegister(
            editor.registerCommand(CAN_UNDO_COMMAND, (payload) => { setCanUndo(payload); return false; }, 1),
            editor.registerCommand(CAN_REDO_COMMAND, (payload) => { setCanRedo(payload); return false; }, 1),
            editor.registerUpdateListener(({ editorState }) => {
                editorState.read(() => {
                    const selection = $getSelection();
                    if ($isRangeSelection(selection)) {
                        const node = selection.getNodes()[0];
                        if (node) {
                            const parent = node.getParent();
                            if ($isLinkNode(parent)) { setIsLink(true); setLinkUrl(parent.getURL()); }
                            else if ($isLinkNode(node)) { setIsLink(true); setLinkUrl(node.getURL()); }
                            else { setIsLink(false); setLinkUrl(''); }
                        }
                    }
                });
            }),
        );
    }, [editor]);

    const handleUndo = useCallback(() => { editor.dispatchCommand(UNDO_COMMAND, undefined); }, [editor]);
    const handleRedo = useCallback(() => { editor.dispatchCommand(REDO_COMMAND, undefined); }, [editor]);

    const handlePaste = useCallback(async () => {
        try {
            const text = await navigator.clipboard.readText();
            document.execCommand('insertText', false, text);
        } catch (err) { console.error('Failed to read clipboard contents: ', err); }
    }, []);

    const handleInsertPageBreak = useCallback(() => {
        editor.update(() => {
            const selection = $getSelection();
            if ($isRangeSelection(selection)) {
                const pbNode = $createPageBreakNode();
                $insertNodeToNearestRoot(pbNode);
                const pNode = $createParagraphStyleNode(null);
                pbNode.insertAfter(pNode);
                pNode.select();
            }
        });
    }, [editor]);

    const handleInsertLineBreak = useCallback(() => { document.execCommand('insertLineBreak'); }, []);

    const handleInsertTable = useCallback(() => {
        editor.dispatchCommand(INSERT_TABLE_COMMAND, { columns: '3', rows: '3', includeHeaders: false });
    }, [editor]);

    const handleSaveLink = useCallback((url: string | null) => {
        editor.dispatchCommand(TOGGLE_LINK_COMMAND, url);
    }, [editor]);

    const handleSaveImage = useCallback((src: string, alt: string) => {
        editor.dispatchCommand(INSERT_IMAGE_COMMAND, { src, altText: alt });
    }, [editor]);

    return {
        canUndo, canRedo, isLink, linkUrl,
        isStylePopoverOpen, setIsStylePopoverOpen,
        isLinkDialogOpen, setIsLinkDialogOpen,
        isImageDialogOpen, setIsImageDialogOpen,
        isPageStyleOpen, setIsPageStyleOpen,
        handleUndo, handleRedo, handlePaste,
        handleInsertPageBreak, handleInsertLineBreak, handleInsertTable,
        handleSaveLink, handleSaveImage,
    };
}
