// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { usePageStyleStore } from '@/lib/stores/pageStyleStore';
import { mmToPx, effectivePageDimensions } from './pageGeometry';

interface PagedEditorContainerProps {
    children: React.ReactNode;
    currentPageIsOdd?: boolean;
}

/**
 * Scroll container that hosts the Lexical editor on a white page surface.
 * The surface width and padding are derived from the active PageStyle so
 * the author sees accurate margins while writing.
 *
 * This component is intentionally simple: it does not attempt to simulate
 * page breaks visually.  Print Preview mode (PrintPreviewView) handles the
 * accurate paginated rendering when the user needs to see page breaks.
 */
export function PagedEditorContainer({
    children,
    currentPageIsOdd = false,
}: PagedEditorContainerProps) {
    const { pageStyle } = usePageStyleStore();

    const { width: pageWidthMm, height: pageHeightMm } = effectivePageDimensions(pageStyle);
    const { margins, duplex } = pageStyle;

    const paddingLeft = duplex
        ? (currentPageIsOdd ? margins.inner : margins.outer)
        : margins.inner;
    const paddingRight = duplex
        ? (currentPageIsOdd ? margins.outer : margins.inner)
        : margins.outer;

    const pageWidthPx = mmToPx(pageWidthMm);
    const pageHeightPx = mmToPx(pageHeightMm);

    return (
        <div
            className="paged-editor-outer flex-1 overflow-y-auto min-h-0"
            style={{
                background: '#c8c8c8',
                paddingTop: 32,
                paddingBottom: 32,
            }}
        >
            <div
                className="paged-editor-surface relative mx-auto"
                style={{
                    width: pageWidthPx,
                    minHeight: pageHeightPx,
                    background: '#ffffff',
                    boxShadow: '0 2px 12px rgba(0,0,0,0.18)',
                    paddingTop: mmToPx(margins.top),
                    paddingBottom: mmToPx(margins.bottom),
                    paddingLeft: mmToPx(paddingLeft),
                    paddingRight: mmToPx(paddingRight),
                }}
            >
                {children}
            </div>
        </div>
    );
}
