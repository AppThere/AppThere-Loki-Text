import { create } from 'zustand';
import type { VectorDocument, VectorObject } from './types';
import {
    addObjectToLayer,
    updateObjectInLayers,
    deleteObjectsFromLayers,
} from './storeHelpers';

export type ToolMode =
    | 'select'
    | 'rect'
    | 'ellipse'
    | 'line'
    | 'pan'
    | 'zoom';

interface VectorEditorState {
    document: VectorDocument | null;
    currentPath: string | null;
    isDirty: boolean;
    activeLayerIndex: number;
    selectedIds: Set<string>;
    toolMode: ToolMode;
    zoom: number;
    panX: number;
    panY: number;
    showGrid: boolean;
    snapToGrid: boolean;
    gridSpacingPx: number;

    // Actions
    setDocument: (doc: VectorDocument) => void;
    setPath: (path: string | null) => void;
    markDirty: () => void;
    markClean: () => void;
    setTool: (tool: ToolMode) => void;
    setZoom: (zoom: number) => void;
    setPan: (x: number, y: number) => void;
    setSelectedIds: (ids: Set<string>) => void;
    addObject: (obj: VectorObject) => void;
    updateObject: (id: string, patch: Partial<VectorObject>) => void;
    deleteSelected: () => void;
    setActiveLayer: (index: number) => void;
    toggleGrid: () => void;
    toggleSnap: () => void;
    reset: () => void;
}

const initialState = {
    document: null,
    currentPath: null,
    isDirty: false,
    activeLayerIndex: 0,
    selectedIds: new Set<string>(),
    toolMode: 'select' as ToolMode,
    zoom: 1.0,
    panX: 0,
    panY: 0,
    showGrid: true,
    snapToGrid: false,
    gridSpacingPx: 10,
};

export const useVectorStore = create<VectorEditorState>((set, get) => ({
    ...initialState,

    setDocument: (doc) => set({ document: doc, isDirty: false }),
    setPath: (path) => set({ currentPath: path }),
    markDirty: () => set({ isDirty: true }),
    markClean: () => set({ isDirty: false }),

    setTool: (tool) => set({ toolMode: tool }),
    setZoom: (zoom) => set({ zoom: Math.max(0.05, Math.min(50, zoom)) }),
    setPan: (x, y) => set({ panX: x, panY: y }),
    setSelectedIds: (ids) => set({ selectedIds: ids }),

    addObject: (obj) => {
        const { document, activeLayerIndex } = get();
        if (!document) return;
        const layers = addObjectToLayer(document.layers, activeLayerIndex, obj);
        set({ document: { ...document, layers }, isDirty: true });
    },

    updateObject: (id, patch) => {
        const { document } = get();
        if (!document) return;
        const layers = updateObjectInLayers(document.layers, id, patch);
        set({ document: { ...document, layers }, isDirty: true });
    },

    deleteSelected: () => {
        const { document, selectedIds } = get();
        if (!document || selectedIds.size === 0) return;
        const layers = deleteObjectsFromLayers(document.layers, selectedIds);
        set({
            document: { ...document, layers },
            selectedIds: new Set(),
            isDirty: true,
        });
    },

    setActiveLayer: (index) => set({ activeLayerIndex: index }),
    toggleGrid: () => set((s) => ({ showGrid: !s.showGrid })),
    toggleSnap: () => set((s) => ({ snapToGrid: !s.snapToGrid })),

    reset: () => set({ ...initialState, selectedIds: new Set<string>() }),
}));
