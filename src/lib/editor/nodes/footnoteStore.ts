// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { create } from 'zustand';

export interface FootnoteContent {
    id: string;
    serialisedState: string;
    createdAt: number;
}

interface FootnoteStore {
    footnotes: FootnoteContent[];
    addFootnote: (id: string) => void;
    removeFootnote: (id: string) => void;
    updateFootnote: (id: string, serialisedState: string) => void;
    getOrphans: (anchorIds: string[]) => FootnoteContent[];
}

const EMPTY_LEXICAL_STATE = JSON.stringify({
    root: {
        children: [{ children: [], direction: null, format: '', indent: 0, type: 'paragraph', version: 1 }],
        direction: null, format: '', indent: 0, type: 'root', version: 1,
    },
});

export const useFootnoteStore = create<FootnoteStore>((set, get) => ({
    footnotes: [],

    addFootnote: (id) => set((s) => ({
        footnotes: [
            ...s.footnotes,
            { id, serialisedState: EMPTY_LEXICAL_STATE, createdAt: Date.now() },
        ],
    })),

    removeFootnote: (id) => set((s) => ({
        footnotes: s.footnotes.filter((f) => f.id !== id),
    })),

    updateFootnote: (id, serialisedState) => set((s) => ({
        footnotes: s.footnotes.map((f) => f.id === id ? { ...f, serialisedState } : f),
    })),

    getOrphans: (anchorIds) => {
        const set = new Set(anchorIds);
        return get().footnotes.filter((f) => !set.has(f.id));
    },
}));
