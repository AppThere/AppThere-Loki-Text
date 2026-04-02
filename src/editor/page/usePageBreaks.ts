// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useState, useEffect, useRef } from 'react';
import { mmToPx } from './pageGeometry';

interface PageBreakPositions {
    breaks: number[];  // px offsets from top of content area, relative to content div
}

/**
 * Observes the content div and computes where visual page-break lines should
 * be drawn, based on body height (page height minus top+bottom margins).
 */
export function usePageBreaks(
    contentRef: React.RefObject<HTMLDivElement | null>,
    bodyHeightMm: number,
    topMarginMm: number,
): PageBreakPositions {
    const [breaks, setBreaks] = useState<number[]>([]);
    const observerRef = useRef<ResizeObserver | null>(null);

    useEffect(() => {
        const el = contentRef.current;
        if (!el) return;

        const bodyHeightPx = mmToPx(bodyHeightMm);
        const topMarginPx = mmToPx(topMarginMm);

        const recompute = () => {
            const totalHeight = el.scrollHeight;
            const positions: number[] = [];
            // First break is at one body height, offset by the top margin
            let nextBreak = bodyHeightPx + topMarginPx;
            while (nextBreak < totalHeight) {
                positions.push(nextBreak);
                nextBreak += bodyHeightPx;
            }
            setBreaks(positions);
        };

        recompute();
        observerRef.current = new ResizeObserver(recompute);
        observerRef.current.observe(el);

        return () => {
            observerRef.current?.disconnect();
            observerRef.current = null;
        };
    }, [contentRef, bodyHeightMm, topMarginMm]);

    return { breaks };
}
