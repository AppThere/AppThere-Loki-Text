// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useState, useEffect, useMemo } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { $getRoot } from 'lexical';
import { FootnoteReferenceNode } from './FootnoteReferenceNode';

/**
 * Returns the 1-based document-order index of the footnote with the given id,
 * or '?' if it is not currently found in the editor state.
 */
export function useFootnoteNumber(footnoteId: string): string {
    const [editor] = useLexicalComposerContext();
    const [ids, setIds] = useState<string[]>([]);

    useEffect(() => {
        const unregister = editor.registerUpdateListener(() => {
            editor.getEditorState().read(() => {
                const found: string[] = [];
                function walk(node: ReturnType<typeof $getRoot>) {
                    const children = node.getChildren?.();
                    if (!children) return;
                    for (const child of children) {
                        if (child instanceof FootnoteReferenceNode) {
                            found.push(child.getFootnoteId());
                        } else {
                            walk(child as ReturnType<typeof $getRoot>);
                        }
                    }
                }
                walk($getRoot());
                setIds(found);
            });
        });
        return unregister;
    }, [editor]);

    return useMemo(() => {
        const idx = ids.indexOf(footnoteId);
        return idx === -1 ? '?' : String(idx + 1);
    }, [ids, footnoteId]);
}
