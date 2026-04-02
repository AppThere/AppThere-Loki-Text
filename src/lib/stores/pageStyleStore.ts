// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { type PageStyle, defaultPageStyle } from '@/editor/page/pageGeometry';

function isValidPageStyle(v: unknown): v is PageStyle {
    if (!v || typeof v !== 'object') return false;
    const s = v as Record<string, unknown>;
    return (
        typeof s.paperSize === 'string' &&
        (s.orientation === 'portrait' || s.orientation === 'landscape') &&
        typeof s.margins === 'object' && s.margins !== null &&
        typeof s.duplex === 'boolean' &&
        (s.footnotePlacement === 'footnote' || s.footnotePlacement === 'endnote')
    );
}

interface PageStyleState {
    pageStyle: PageStyle;
    setPageStyle: (style: PageStyle) => void;
}

export const usePageStyleStore = create<PageStyleState>()(
    persist(
        (set) => ({
            pageStyle: defaultPageStyle,
            setPageStyle: (style) => set({ pageStyle: style }),
        }),
        {
            name: 'loki.pageStyle',
            onRehydrateStorage: () => (state) => {
                if (state && !isValidPageStyle(state.pageStyle)) {
                    state.pageStyle = defaultPageStyle;
                }
            },
        }
    )
);
