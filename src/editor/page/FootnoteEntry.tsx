// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useCallback } from 'react';
import { LexicalComposer } from '@lexical/react/LexicalComposer';
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin';
import { ContentEditable } from '@lexical/react/LexicalContentEditable';
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary';
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin';
import type { EditorState } from 'lexical';
import { useFootnoteStore } from '@/lib/editor/nodes/footnoteStore';
import type { FootnoteContent } from '@/lib/editor/nodes/footnoteStore';

interface FootnoteEntryProps {
    footnote: FootnoteContent;
    number: number;
    onAnchorClick: (id: string) => void;
}

const miniEditorConfig = {
    namespace: 'FootnoteMini',
    theme: {
        paragraph: 'text-sm',
        text: { bold: 'font-bold', italic: 'italic', underline: 'underline' },
    },
    onError: (err: Error) => console.error('Footnote editor error:', err),
};

export function FootnoteEntry({ footnote, number, onAnchorClick }: FootnoteEntryProps) {
    const { updateFootnote } = useFootnoteStore();

    const config = {
        ...miniEditorConfig,
        editorState: footnote.serialisedState || undefined,
    };

    const handleChange = useCallback((editorState: EditorState) => {
        updateFootnote(footnote.id, JSON.stringify(editorState.toJSON()));
    }, [footnote.id, updateFootnote]);

    return (
        <div
            id={`footnote-entry-${footnote.id}`}
            className="flex gap-2 items-start py-1"
        >
            <button
                type="button"
                className="text-xs font-semibold text-blue-600 shrink-0 w-5 text-right leading-5 hover:underline"
                onClick={() => onAnchorClick(footnote.id)}
                title="Jump to anchor"
            >
                {number}.
            </button>
            <div className="flex-1 min-w-0">
                <LexicalComposer initialConfig={config}>
                    <div className="relative">
                        <RichTextPlugin
                            contentEditable={
                                <ContentEditable
                                    className="text-sm outline-none min-h-[1.5em] leading-normal"
                                />
                            }
                            placeholder={<span className="absolute top-0 left-0 text-sm text-muted-foreground pointer-events-none">Note…</span>}
                            ErrorBoundary={LexicalErrorBoundary}
                        />
                    </div>
                    <OnChangePlugin onChange={handleChange} ignoreSelectionChange />
                </LexicalComposer>
            </div>
        </div>
    );
}
