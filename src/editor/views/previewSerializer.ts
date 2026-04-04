// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { mmToPx, effectivePageDimensions } from '@/editor/page/pageGeometry';
import type { PageStyle } from '@/editor/page/pageGeometry';

export interface PreviewPage {
    /** Index of the first block node in this page (inclusive). */
    startIndex: number;
    /** Index of the last block node in this page (inclusive). */
    endIndex: number;
    /** Serialised HTML content for this page. */
    html: string;
}

/**
 * Split the static HTML snapshot of the document into pages using a heuristic
 * based on the block-level children of the editor root.
 *
 * The approach:
 * 1. Clone the snapshot container.
 * 2. Walk top-level children, accumulating their `offsetHeight`.
 * 3. Start a new page whenever the accumulated height exceeds the body height.
 * 4. Forced page breaks (elements with `data-page-break` or the `.page-break`
 *    class) also trigger a new page.
 *
 * This is a heuristic — it will not be pixel-perfect for every document, but
 * it is accurate enough for print preview purposes.
 */
export function splitIntoPages(
    container: HTMLElement,
    pageStyle: PageStyle,
): PreviewPage[] {
    const { width: pageWidthMm, height: pageHeightMm } = effectivePageDimensions(pageStyle);
    const { margins } = pageStyle;
    const bodyHeightPx = mmToPx(pageHeightMm - margins.top - margins.bottom);
    const pageWidthPx = mmToPx(pageWidthMm - margins.inner - margins.outer);

    const children = Array.from(container.children) as HTMLElement[];
    if (children.length === 0) {
        return [{ startIndex: 0, endIndex: 0, html: container.innerHTML }];
    }

    const pages: PreviewPage[] = [];
    let pageStart = 0;
    let accumulated = 0;
    const pageNodes: HTMLElement[][] = [[]];

    for (let i = 0; i < children.length; i++) {
        const child = children[i];
        const isForced =
            child.classList.contains('page-break') ||
            child.dataset['pageBreak'] === 'true';

        if (isForced) {
            // Flush current page (excluding the break node itself).
            pages.push(buildPage(pageStart, i - 1, pageNodes[pageNodes.length - 1], pageWidthPx));
            pageStart = i + 1;
            accumulated = 0;
            pageNodes.push([]);
            continue;
        }

        const childHeight = child.offsetHeight || estimateBlockHeight(child);

        if (accumulated + childHeight > bodyHeightPx && accumulated > 0) {
            // Overflow — start new page at this child.
            pages.push(buildPage(pageStart, i - 1, pageNodes[pageNodes.length - 1], pageWidthPx));
            pageStart = i;
            accumulated = 0;
            pageNodes.push([]);
        }

        pageNodes[pageNodes.length - 1].push(child);
        accumulated += childHeight;
    }

    // Flush last page.
    const lastNodes = pageNodes[pageNodes.length - 1];
    pages.push(buildPage(pageStart, children.length - 1, lastNodes, pageWidthPx));

    return pages;
}

function buildPage(
    startIndex: number,
    endIndex: number,
    nodes: HTMLElement[],
    _pageWidthPx: number,
): PreviewPage {
    const html = nodes.map((n) => n.outerHTML).join('');
    return { startIndex, endIndex, html };
}

/**
 * Rough line-height estimate when `offsetHeight` is unavailable (e.g. in
 * a hidden container).  Counts text length as a proxy for line count.
 */
function estimateBlockHeight(el: HTMLElement): number {
    const LINE_HEIGHT_PX = 20;
    const CHARS_PER_LINE = 80;
    const text = el.textContent ?? '';
    const lines = Math.max(1, Math.ceil(text.length / CHARS_PER_LINE));
    return lines * LINE_HEIGHT_PX + 8; // +8 for paragraph spacing
}
