// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useEffect } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { COMMAND_PRIORITY_EDITOR, $getSelection, $isRangeSelection, $insertNodes } from 'lexical';
import { INSERT_FOOTNOTE_COMMAND } from '@/editor/commands/footnoteCommands';
import { $createFootnoteReferenceNode } from '@/lib/editor/nodes/FootnoteReferenceNode';
import { useFootnoteStore } from '@/lib/editor/nodes/footnoteStore';
import { useFootnoteGarbageCollector } from '@/lib/editor/nodes/useFootnoteGarbageCollector';

/** Generates a UUID v4 without the `crypto.randomUUID` API for broad compat. */
function uuidv4(): string {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
        const r = (Math.random() * 16) | 0;
        return (c === 'x' ? r : (r & 0x3) | 0x8).toString(16);
    });
}

/**
 * Registers the `INSERT_FOOTNOTE_COMMAND` handler and mounts the footnote
 * garbage collector. Must be rendered inside a `LexicalComposer`.
 */
export function FootnotePlugin(): null {
    const [editor] = useLexicalComposerContext();
    const { addFootnote } = useFootnoteStore();

    useFootnoteGarbageCollector();

    useEffect(() => {
        return editor.registerCommand(
            INSERT_FOOTNOTE_COMMAND,
            () => {
                const id = uuidv4();
                addFootnote(id);
                editor.update(() => {
                    const selection = $getSelection();
                    if ($isRangeSelection(selection)) {
                        const node = $createFootnoteReferenceNode(id);
                        $insertNodes([node]);
                        node.selectNext();
                    }
                });
                return true;
            },
            COMMAND_PRIORITY_EDITOR,
        );
    }, [editor, addFootnote]);

    return null;
}
