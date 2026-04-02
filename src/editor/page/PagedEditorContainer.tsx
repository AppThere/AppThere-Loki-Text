// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useRef, useEffect } from 'react';
import { usePageStyleStore } from '@/lib/stores/pageStyleStore';
import { mmToPx, effectivePageDimensions } from './pageGeometry';
import { usePageBreaks } from './usePageBreaks';

interface PagedEditorContainerProps {
    children: React.ReactNode;
    currentPageIsOdd?: boolean;
}

export function PagedEditorContainer({
    children,
    currentPageIsOdd = false,
}: PagedEditorContainerProps) {
    const { pageStyle } = usePageStyleStore();
    const contentRef = useRef<HTMLDivElement>(null);

    const { width: pageWidthMm, height: pageHeightMm } = effectivePageDimensions(pageStyle);
    const { margins, duplex } = pageStyle;

    // Simplex vs duplex: for duplex, odd pages get inner on the left.
    const paddingLeft = duplex
        ? (currentPageIsOdd ? margins.inner : margins.outer)
        : margins.inner;
    const paddingRight = duplex
        ? (currentPageIsOdd ? margins.outer : margins.inner)
        : margins.outer;

    const bodyHeightMm = pageHeightMm - margins.top - margins.bottom;
    const { breaks } = usePageBreaks(contentRef, bodyHeightMm, margins.top);

    // Apply dark mode CSS variables via style element
    useEffect(() => {
        const styleId = 'paged-editor-vars';
        let el = document.getElementById(styleId) as HTMLStyleElement | null;
        if (!el) {
            el = document.createElement('style');
            el.id = styleId;
            document.head.appendChild(el);
        }
        el.textContent = `
@media (prefers-color-scheme: dark) {
  :root {
    --page-gap-bg: #2a2a2a;
    --page-surface: #1e1e1e;
    --page-break-color: #3a4050;
  }
}`;
        return () => {
            el?.remove();
        };
    }, []);

    const pageWidthPx = mmToPx(pageWidthMm);
    const pageHeightPx = mmToPx(pageHeightMm);

    return (
        <div
            className="paged-editor-outer flex-1 overflow-y-auto min-h-0"
            style={{
                background: 'var(--page-gap-bg, #c8c8c8)',
                paddingTop: 32,
                paddingBottom: 32,
            }}
        >
            <div
                ref={contentRef}
                className="paged-editor-surface relative mx-auto"
                style={{
                    width: pageWidthPx,
                    minHeight: pageHeightPx,
                    background: 'var(--page-surface, #ffffff)',
                    boxShadow: '0 2px 12px rgba(0,0,0,0.18)',
                    paddingTop: mmToPx(margins.top),
                    paddingBottom: mmToPx(margins.bottom),
                    paddingLeft: mmToPx(paddingLeft),
                    paddingRight: mmToPx(paddingRight),
                }}
            >
                {children}

                {/* Page break indicators */}
                {breaks.map((pos) => (
                    <div
                        key={pos}
                        aria-hidden="true"
                        style={{
                            position: 'absolute',
                            top: pos,
                            left: -32,
                            right: -32,
                            height: 2,
                            background: 'var(--page-break-color, #b0b8c8)',
                            pointerEvents: 'none',
                        }}
                    />
                ))}
            </div>
        </div>
    );
}
