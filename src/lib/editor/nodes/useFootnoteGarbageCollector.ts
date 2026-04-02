// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useEffect } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { $getRoot } from 'lexical';
import { FootnoteReferenceNode } from './FootnoteReferenceNode';
import { useFootnoteStore } from './footnoteStore';

const GRACE_MS = 5000;

/**
 * Registers an editor update listener that removes footnote store entries
 * whose anchor nodes have been absent for more than 5 seconds.
 */
export function useFootnoteGarbageCollector(): void {
    const [editor] = useLexicalComposerContext();
    const { getOrphans, removeFootnote } = useFootnoteStore();

    useEffect(() => {
        const unregister = editor.registerUpdateListener(() => {
            editor.getEditorState().read(() => {
                const anchorIds: string[] = [];
                function walk(node: ReturnType<typeof $getRoot>) {
                    const children = node.getChildren?.();
                    if (!children) return;
                    for (const child of children) {
                        if (child instanceof FootnoteReferenceNode) {
                            anchorIds.push(child.getFootnoteId());
                        } else {
                            walk(child as ReturnType<typeof $getRoot>);
                        }
                    }
                }
                walk($getRoot());

                const now = Date.now();
                const orphans = getOrphans(anchorIds);
                for (const orphan of orphans) {
                    if (now - orphan.createdAt > GRACE_MS) {
                        removeFootnote(orphan.id);
                    }
                }
            });
        });
        return unregister;
    }, [editor, getOrphans, removeFootnote]);
}
