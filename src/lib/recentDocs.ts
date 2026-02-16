import { BaseDirectory, readTextFile, writeTextFile, exists } from '@tauri-apps/plugin-fs';

const RECENT_DOCS_FILE = 'recent-documents.json';

export interface RecentDoc {
    path: string;
    title: string;
    lastOpened: string; // ISO date string
}

export const recentDocs = {
    async add(path: string, title?: string) {
        try {
            const list = await this.get();
            const existingIndex = list.findIndex(doc => doc.path === path);

            const newDoc: RecentDoc = {
                path,
                title: title || (existingIndex >= 0 ? list[existingIndex].title : path.split('/').pop() || 'Untitled'),
                lastOpened: new Date().toISOString()
            };

            let newList = [newDoc, ...list.filter(d => d.path !== path)];

            // Limit to 10 items
            if (newList.length > 10) {
                newList = newList.slice(0, 10);
            }

            await writeTextFile(RECENT_DOCS_FILE, JSON.stringify(newList, null, 2), { baseDir: BaseDirectory.AppLocalData });
            return newList;
        } catch (e) {
            console.error('Failed to add recent doc', e);
            return [];
        }
    },

    async get(): Promise<RecentDoc[]> {
        try {
            const fileExists = await exists(RECENT_DOCS_FILE, { baseDir: BaseDirectory.AppLocalData });
            if (!fileExists) {
                return [];
            }
            const content = await readTextFile(RECENT_DOCS_FILE, { baseDir: BaseDirectory.AppLocalData });
            return JSON.parse(content);
        } catch (e) {
            console.error('Failed to get recent docs', e);
            return [];
        }
    },

    async clear() {
        try {
            await writeTextFile(RECENT_DOCS_FILE, '[]', { baseDir: BaseDirectory.AppLocalData });
        } catch (e) {
            console.error('Failed to clear recent docs', e);
        }
    }
};
