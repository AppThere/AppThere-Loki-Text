import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface HistoryItem {
    path: string;
    name: string;
    timestamp: number;
    type: 'text' | 'spreadsheet' | 'vector' | 'presentation' | 'image';
}

interface HistoryState {
    recentDocuments: HistoryItem[];
    recentTemplates: Record<string, HistoryItem[]>; // type -> items

    addDocument: (item: Omit<HistoryItem, 'timestamp'>) => void;
    addTemplate: (type: string, item: Omit<HistoryItem, 'timestamp'>) => void;
    removeDocument: (path: string) => void;
    removeTemplate: (type: string, path: string) => void;
    clearHistory: () => void;
}

export const useHistoryStore = create<HistoryState>()(
    persist(
        (set) => ({
            recentDocuments: [],
            recentTemplates: {},

            addDocument: (item) => set((state) => {
                const newItem = { ...item, timestamp: Date.now() };
                const filtered = state.recentDocuments.filter(d => d.path !== item.path);
                return {
                    recentDocuments: [newItem, ...filtered].slice(0, 10)
                };
            }),

            addTemplate: (type, item) => set((state) => {
                const newItem = { ...item, timestamp: Date.now() };
                const existing = state.recentTemplates[type] || [];
                const filtered = existing.filter(t => t.path !== item.path);
                return {
                    recentTemplates: {
                        ...state.recentTemplates,
                        [type]: [newItem, ...filtered].slice(0, 5)
                    }
                };
            }),

            removeDocument: (path) => set((state) => ({
                recentDocuments: state.recentDocuments.filter(d => d.path !== path)
            })),

            removeTemplate: (type, path) => set((state) => {
                const existing = state.recentTemplates[type] || [];
                return {
                    recentTemplates: {
                        ...state.recentTemplates,
                        [type]: existing.filter(t => t.path !== path)
                    }
                };
            }),

            clearHistory: () => set({ recentDocuments: [], recentTemplates: {} }),
        }),
        {
            name: 'loki_history',
        }
    )
);
