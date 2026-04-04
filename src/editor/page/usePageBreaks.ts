// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useState, useEffect, useRef } from 'react';
import { mmToPx } from './pageGeometry';

export interface PageBreakPositions {
    /** Top-of-band px offsets for regular page-height intervals. */
    breaks: number[];
    /** Top-of-band px offsets for explicit PageBreakNode elements. */
    forcedBreaks: number[];
}

/**
 * Observes the content div and returns two sets of break positions:
 *
 * `breaks` — regular intervals derived from page geometry (body height).
 * `forcedBreaks` — positions of `.page-break` DOM nodes (from PageBreakNode),
 *   which represent explicitly inserted page breaks in the document.
 *
 * Each position is the top edge of where the inter-page gap band should be
 * rendered.  The caller is responsible for subtracting half the band height to
 * centre the band around each break position.
 */
export function usePageBreaks(
    contentRef: React.RefObject<HTMLDivElement | null>,
    bodyHeightMm: number,
    topMarginMm: number,
): PageBreakPositions {
    const [breaks, setBreaks] = useState<number[]>([]);
    const [forcedBreaks, setForcedBreaks] = useState<number[]>([]);
    const observerRef = useRef<ResizeObserver | null>(null);

    useEffect(() => {
        const el = contentRef.current;
        if (!el) return;

        const bodyHeightPx = mmToPx(bodyHeightMm);
        const topMarginPx = mmToPx(topMarginMm);

        const recompute = () => {
            // ── Regular interval breaks ─────────────────────────────────────
            const totalHeight = el.scrollHeight;
            const positions: number[] = [];
            let nextBreak = bodyHeightPx + topMarginPx;
            while (nextBreak < totalHeight) {
                positions.push(nextBreak);
                nextBreak += bodyHeightPx;
            }
            setBreaks(positions);

            // ── Forced breaks from PageBreakNode (.page-break elements) ─────
            const pbElements = el.querySelectorAll<HTMLElement>('.page-break');
            const forced: number[] = [];
            pbElements.forEach((pb) => {
                // offsetTop is relative to the nearest positioned ancestor = el
                forced.push(pb.offsetTop);
            });
            setForcedBreaks(forced);
        };

        recompute();
        observerRef.current = new ResizeObserver(recompute);
        observerRef.current.observe(el);

        // Also watch for DOM mutations so new page-break nodes are caught
        const mutObs = new MutationObserver(recompute);
        mutObs.observe(el, { childList: true, subtree: true });

        return () => {
            observerRef.current?.disconnect();
            observerRef.current = null;
            mutObs.disconnect();
        };
    }, [contentRef, bodyHeightMm, topMarginMm]);

    return { breaks, forcedBreaks };
}
