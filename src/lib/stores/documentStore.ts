import { create } from 'zustand';
import type { StyleDefinition, Metadata, LexicalDocumentData } from '../types/odt';
import type { SessionManager } from '../session/SessionManager';
import type { DocumentViewMode } from '@/editor/views/DocumentView';

interface DocumentState {
    currentPath: string | null;
    currentContent: LexicalDocumentData | null;
    styles: Record<string, StyleDefinition>;
    metadata: Metadata;
    currentStyle: string;
    isDirty: boolean;
    isSaving: boolean;
    lastSaved: Date | null;
    /** Active session manager — null when no document is open. */
    session: SessionManager | null;
    viewMode: DocumentViewMode;

    setPath: (path: string) => void;
    setContent: (content: LexicalDocumentData) => void;
    setStyles: (styles: Record<string, StyleDefinition>) => void;
    setMetadata: (metadata: Metadata) => void;
    setStyle: (style: string) => void;
    setSession: (session: SessionManager | null) => void;
    setViewMode: (mode: DocumentViewMode) => void;
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
    session: null,
    viewMode: 'scroll',

    setPath: (path) => set({ currentPath: path }),
    setContent: (content) => set({ currentContent: content, isDirty: true }),
    setStyles: (styles) => set({ styles, isDirty: true }),
    setMetadata: (metadata) => set({ metadata, isDirty: true }),
    setStyle: (style) => set({ currentStyle: style }),
    setSession: (session) => set({ session }),
    setViewMode: (viewMode) => set({ viewMode }),
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
        session: null,
        viewMode: 'scroll',
    }),
}));
