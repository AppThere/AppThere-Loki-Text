import { create } from 'zustand';
import type { StyleDefinition, Metadata, LexicalDocumentData } from '../types/odt';

interface DocumentState {
    currentPath: string | null;
    currentContent: LexicalDocumentData | null;
    styles: Record<string, StyleDefinition>;
    metadata: Metadata;
    currentStyle: string;
    isDirty: boolean;
    isSaving: boolean;
    lastSaved: Date | null;

    setPath: (path: string) => void;
    setContent: (content: LexicalDocumentData) => void;
    setStyles: (styles: Record<string, StyleDefinition>) => void;
    setMetadata: (metadata: Metadata) => void;
    setStyle: (style: string) => void;
    markDirty: () => void;
    markClean: () => void;
    markSaving: () => void;
    markSaved: () => void;
    reset: () => void;
    restoreState: (state: Partial<DocumentState>) => void;
}

export const useDocumentStore = create<DocumentState>((set) => ({
    currentPath: null,
    currentContent: null,
    styles: {},
    metadata: {
        identifier: null,
        title: null,
        language: null,
        description: null,
        subject: null,
        creator: null,
        creationDate: null,
        generator: 'AppThere Loki Text',
    },
    currentStyle: 'Standard',
    isDirty: false,
    isSaving: false,
    lastSaved: null,

    setPath: (path) => set({ currentPath: path }),
    setContent: (content) => set({ currentContent: content, isDirty: true }),
    setStyles: (styles) => set({ styles, isDirty: true }),
    setMetadata: (metadata) => set({ metadata, isDirty: true }),
    setStyle: (style) => set({ currentStyle: style }),
    markDirty: () => set({ isDirty: true }),
    markClean: () => set({ isDirty: false, isSaving: false }),
    markSaving: () => set({ isSaving: true, isDirty: false }),
    markSaved: () => set({ isSaving: false, isDirty: false, lastSaved: new Date() }),

    restoreState: (state) => set((prev) => ({ ...prev, ...state })),

    reset: () => set({
        currentPath: null,
        currentContent: null,
        styles: {},
        currentStyle: 'Standard',
        isDirty: false,
        isSaving: false,
        lastSaved: null,
    }),
}));
