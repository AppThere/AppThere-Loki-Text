// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { PagedEditorContainer } from '@/editor/page/PagedEditorContainer';
import { FootnotePanel } from '@/editor/page/FootnotePanel';

interface ScrollViewProps {
    children: React.ReactNode;
}

/**
 * Default authoring view: a continuous scrollable page surface with the
 * footnote / endnote panel below the content area.
 *
 * Children are the Lexical editor plugins and ContentEditable element.
 */
export function ScrollView({ children }: ScrollViewProps) {
    return (
        <div className="scroll-view flex-1 flex flex-col min-h-0">
            <PagedEditorContainer>
                {children}
            </PagedEditorContainer>
            <FootnotePanel />
        </div>
    );
}
