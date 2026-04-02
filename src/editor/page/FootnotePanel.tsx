// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useCallback } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { $getRoot } from 'lexical';
import { useTranslation } from 'react-i18next';
import { useFootnoteStore } from '@/lib/editor/nodes/footnoteStore';
import { usePageStyleStore } from '@/lib/stores/pageStyleStore';
import { FootnoteReferenceNode } from '@/lib/editor/nodes/FootnoteReferenceNode';
import { FootnoteEntry } from './FootnoteEntry';

/**
 * Panel rendered below the paged editor surface showing all footnote / endnote
 * entries in document order. Visibility is governed by `footnotes.length` and
 * `pageStyle.footnotePlacement`.
 */
export function FootnotePanel() {
    const { t } = useTranslation('common');
    const { footnotes } = useFootnoteStore();
    const { pageStyle } = usePageStyleStore();
    const [editor] = useLexicalComposerContext();

    // Build an ordered list of ids from the document
    const orderedIds: string[] = [];
    editor.getEditorState().read(() => {
        function walk(node: ReturnType<typeof $getRoot>) {
            const children = node.getChildren?.();
            if (!children) return;
            for (const child of children) {
                if (child instanceof FootnoteReferenceNode) {
                    orderedIds.push(child.getFootnoteId());
                } else {
                    walk(child as ReturnType<typeof $getRoot>);
                }
            }
        }
        walk($getRoot());
    });

    // Sort footnotes by document order
    const ordered = orderedIds
        .map((id) => footnotes.find((f) => f.id === id))
        .filter(Boolean) as typeof footnotes;

    const handleAnchorClick = useCallback((id: string) => {
        const el = document.querySelector(`[data-footnote-id="${id}"]`);
        if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }, []);

    if (ordered.length === 0) return null;

    const isEndnote = pageStyle.footnotePlacement === 'endnote';

    return (
        <div className="footnote-panel w-full shrink-0">
            {/* Section divider */}
            <div className="w-2/5 border-t border-border my-2" />

            <div className="px-4 pb-4">
                {isEndnote && (
                    <h3 className="text-sm font-semibold mb-2 text-muted-foreground">
                        {t('footnote.endnotesHeading')}
                    </h3>
                )}
                {!isEndnote && (
                    <p className="text-xs text-muted-foreground mb-1 sr-only">
                        {t('footnote.footnotesHeading')}
                    </p>
                )}

                <div className="space-y-0.5">
                    {ordered.map((fn, idx) => (
                        <FootnoteEntry
                            key={fn.id}
                            footnote={fn}
                            number={idx + 1}
                            onAnchorClick={handleAnchorClick}
                        />
                    ))}
                </div>
            </div>
        </div>
    );
}
