// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useState, useEffect, useRef, useCallback } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { $getRoot } from 'lexical';
import { useDocumentStore } from '@/lib/stores/documentStore';
import { DocumentViewContext } from './DocumentView';
import type { DocumentViewMode } from './DocumentView';
import { ScrollView } from './ScrollView';
import { PrintPreviewView } from './PrintPreviewView';

interface DocumentViewProviderProps {
    children: React.ReactNode;
}

/**
 * Provides the `DocumentViewContext` and switches between `ScrollView` and
 * `PrintPreviewView` based on `viewMode` in the document store.
 *
 * When entering preview mode a static HTML snapshot is captured from the
 * Lexical editor state and passed to `PrintPreviewView` for rendering.
 * The snapshot is re-captured every time the user enters preview mode.
 *
 * Must be mounted inside a `LexicalComposer`.
 */
export function DocumentViewProvider({ children }: DocumentViewProviderProps) {
    const [editor] = useLexicalComposerContext();
    const { viewMode, setViewMode } = useDocumentStore();
    const [snapshotHtml, setSnapshotHtml] = useState('');
    const snapshotContainerRef = useRef<HTMLDivElement>(null);

    // Capture a static HTML snapshot when entering preview mode.
    useEffect(() => {
        if (viewMode !== 'preview') return;
        captureSnapshot(editor, snapshotContainerRef, setSnapshotHtml);
    }, [editor, viewMode]);

    const contextValue = { viewMode, setViewMode };

    return (
        <DocumentViewContext.Provider value={contextValue}>
            {/* Hidden container used for snapshot rendering */}
            <div
                ref={snapshotContainerRef}
                aria-hidden="true"
                style={{
                    position: 'absolute',
                    visibility: 'hidden',
                    pointerEvents: 'none',
                    top: -9999,
                    left: -9999,
                    width: 600,
                }}
            />

            {viewMode === 'scroll' ? (
                <ScrollView>{children}</ScrollView>
            ) : (
                <PrintPreviewView snapshotHtml={snapshotHtml} />
            )}
        </DocumentViewContext.Provider>
    );
}

// ---------------------------------------------------------------------------
// Snapshot helper
// ---------------------------------------------------------------------------

/**
 * Walk the Lexical editor state and serialise each top-level block node's
 * text to HTML.  This is a best-effort plain-text snapshot; rich formatting
 * is preserved via `exportDOM` on nodes that support it.
 *
 * For fidelity we use the live DOM of the ContentEditable directly, cloning
 * its content without the contenteditable attribute.
 */
function captureSnapshot(
    editor: ReturnType<typeof useLexicalComposerContext>[0],
    containerRef: React.RefObject<HTMLDivElement | null>,
    setHtml: (html: string) => void,
): void {
    editor.getEditorState().read(() => {
        const root = $getRoot();
        const text = root.getTextContent();
        if (!text && root.getChildrenSize() === 0) {
            setHtml('');
            return;
        }
    });

    // Use the live DOM as source of truth — it carries rendered text styles.
    const editableEl = editor.getRootElement();
    if (editableEl) {
        const clone = editableEl.cloneNode(true) as HTMLElement;
        clone.removeAttribute('contenteditable');
        clone.removeAttribute('role');
        clone.removeAttribute('aria-label');
        setHtml(clone.innerHTML);
        return;
    }

    setHtml('');
}

/**
 * Re-export `useDocumentView` convenience hook from this module so callers
 * only need one import.
 */
export { useDocumentView } from './DocumentView';
