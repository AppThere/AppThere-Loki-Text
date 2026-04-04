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

/**
 * Band height for the inter-page gap: sum of bottom margin of the ending page
 * and top margin of the starting page.  This makes the band occupy exactly the
 * margin whitespace that would surround the break in a real paginated renderer,
 * and its background colour matches the outer grey workspace so the two merge
 * visually.
 *
 * Note — this is still the CSS illusion: the band overlays continuous content
 * rather than pushing it.  Content that overflows into the margin area (e.g. a
 * large table) will be hidden behind the band.  That is an accepted trade-off
 * of the single-Lexical-instance approach.
 */
export function PagedEditorContainer({
    children,
    currentPageIsOdd = false,
}: PagedEditorContainerProps) {
    const { pageStyle } = usePageStyleStore();
    const contentRef = useRef<HTMLDivElement>(null);

    const { width: pageWidthMm, height: pageHeightMm } = effectivePageDimensions(pageStyle);
    const { margins, duplex } = pageStyle;

    const paddingLeft = duplex
        ? (currentPageIsOdd ? margins.inner : margins.outer)
        : margins.inner;
    const paddingRight = duplex
        ? (currentPageIsOdd ? margins.outer : margins.inner)
        : margins.outer;

    const bodyHeightMm = pageHeightMm - margins.top - margins.bottom;
    const { breaks, forcedBreaks } = usePageBreaks(contentRef, bodyHeightMm, margins.top);

    // Band geometry: covers the bottom margin of page N and the top margin of
    // page N+1.  Centered on the break position (which sits at the end of the
    // body-content area, i.e. the start of the bottom margin).
    const bandHeightPx = Math.max(mmToPx(margins.top + margins.bottom), 24);
    const bottomMarginPx = mmToPx(margins.bottom);

    // Inject dark-mode CSS custom properties once.
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
  }
}`;
        return () => { el?.remove(); };
    }, []);

    const pageWidthPx = mmToPx(pageWidthMm);
    const pageHeightPx = mmToPx(pageHeightMm);

    // Render an inter-page gap band at a given break position.
    // `pos` is the top of where the bottom margin begins (end of body content).
    const renderBand = (pos: number, key: string, isForced: boolean) => (
        <div
            key={key}
            aria-hidden="true"
            title={isForced ? 'Manual page break' : undefined}
            style={{
                position: 'absolute',
                // Align the top of the band with the start of the bottom margin
                top: pos - bottomMarginPx,
                // Bleed 32 px into the outer grey area on both sides so the band
                // merges seamlessly with the workspace background.
                left: -32,
                right: -32,
                height: bandHeightPx,
                // Same colour as the outer grey workspace — this is what makes
                // the illusion convincing in light mode.
                background: 'var(--page-gap-bg, #c8c8c8)',
                // Hairline borders mark the page edges inside the band.
                borderTop: isForced
                    ? '2px dashed rgba(0,0,0,0.25)'
                    : '1px solid rgba(0,0,0,0.12)',
                borderBottom: '1px solid rgba(0,0,0,0.12)',
                pointerEvents: 'none',
                zIndex: 1,
            }}
        />
    );

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

                {/* Regular page-height interval bands */}
                {breaks.map((pos) => renderBand(pos, `interval-${pos}`, false))}

                {/* Forced page breaks from PageBreakNode */}
                {forcedBreaks.map((pos) => renderBand(pos, `forced-${pos}`, true))}
            </div>
        </div>
    );
}
