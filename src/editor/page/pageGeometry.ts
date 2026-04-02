// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

export type PaperSizeKey = 'A4' | 'A5' | 'A3' | 'Letter' | 'Legal' | 'Tabloid';

/** All values in millimetres. Width × Height in portrait orientation. */
export const paperDimensions: Record<PaperSizeKey, { width: number; height: number }> = {
    A4:      { width: 210,   height: 297 },
    A5:      { width: 148,   height: 210 },
    A3:      { width: 297,   height: 420 },
    Letter:  { width: 215.9, height: 279.4 },
    Legal:   { width: 215.9, height: 355.6 },
    Tabloid: { width: 279.4, height: 431.8 },
};

export interface MarginSpec {
    top: number;    // mm
    bottom: number; // mm
    inner: number;  // mm — gutter/binding side; equals "left" for simplex
    outer: number;  // mm — non-binding side; equals "right" for simplex
}

export interface PageStyle {
    paperSize: PaperSizeKey | 'custom';
    customWidth?: number;   // mm, present only when paperSize === 'custom'
    customHeight?: number;  // mm
    orientation: 'portrait' | 'landscape';
    margins: MarginSpec;
    duplex: boolean;
    footnotePlacement: 'footnote' | 'endnote';
}

/** Convert millimetres to CSS pixels (96 dpi). */
export function mmToPx(mm: number): number {
    return (mm * 96) / 25.4;
}

/** Effective page dimensions in mm, respecting orientation and custom size. */
export function effectivePageDimensions(style: PageStyle): { width: number; height: number } {
    const base = style.paperSize === 'custom'
        ? { width: style.customWidth ?? 210, height: style.customHeight ?? 297 }
        : paperDimensions[style.paperSize];
    if (style.orientation === 'landscape') {
        return { width: base.height, height: base.width };
    }
    return base;
}

export const defaultPageStyle: PageStyle = {
    paperSize: 'A4',
    orientation: 'portrait',
    margins: { top: 25.4, bottom: 25.4, inner: 25.4, outer: 25.4 },
    duplex: false,
    footnotePlacement: 'footnote',
};
