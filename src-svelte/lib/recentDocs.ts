import { fileService } from './services/FileService';

const RECENT_DOCS_FILE = 'recent-documents.json';

export interface RecentDoc {
	path: string;
	title: string;
	lastOpened: string; // ISO date string
}

export const recentDocs = {
	async add(path: string, title?: string) {
		try {
			const { BaseDirectory } = await import('@tauri-apps/plugin-fs');
			const list = await this.get();
			const existingIndex = list.findIndex((doc) => doc.path === path);

			const newDoc: RecentDoc = {
				path,
				title:
					title ||
					(existingIndex >= 0 ? list[existingIndex].title : path.split('/').pop() || 'Untitled'),
				lastOpened: new Date().toISOString()
			};

			let newList = [newDoc, ...list.filter((d) => d.path !== path)];

			// Limit to 10 items
			if (newList.length > 10) {
				newList = newList.slice(0, 10);
			}

			// Ensure directory exists
			await fileService.mkdir('', { baseDir: BaseDirectory.AppLocalData, recursive: true });

			await fileService.writeTextFile(RECENT_DOCS_FILE, JSON.stringify(newList, null, 2), {
				baseDir: BaseDirectory.AppLocalData
			});
			return newList;
		} catch (e) {
			console.error('Failed to add recent doc', e);
			return [];
		}
	},

	async get(): Promise<RecentDoc[]> {
		try {
			const { BaseDirectory } = await import('@tauri-apps/plugin-fs');
			const fileExists = await fileService.exists(RECENT_DOCS_FILE, {
				baseDir: BaseDirectory.AppLocalData
			});

			if (!fileExists) {
				return [];
			}

			const content = await fileService.readTextFile(RECENT_DOCS_FILE, {
				baseDir: BaseDirectory.AppLocalData
			});
			return JSON.parse(content);
		} catch (e) {
			console.error('Failed to get recent docs', e);
			return [];
		}
	},

	async clear() {
		try {
			const { BaseDirectory } = await import('@tauri-apps/plugin-fs');
			await fileService.writeTextFile(RECENT_DOCS_FILE, '[]', {
				baseDir: BaseDirectory.AppLocalData
			});
		} catch (e) {
			console.error('Failed to clear recent docs', e);
		}
	},
	async remove(path: string) {
		try {
			const { BaseDirectory } = await import('@tauri-apps/plugin-fs');
			const list = await this.get();
			const newList = list.filter((doc) => doc.path !== path);
			await fileService.writeTextFile(RECENT_DOCS_FILE, JSON.stringify(newList, null, 2), {
				baseDir: BaseDirectory.AppLocalData
			});
			return newList;
		} catch (e) {
			console.error('Failed to remove recent doc', e);
			return [];
		}
	},
	async deleteFile(path: string) {
		try {
			// First remove from list
			const newList = await this.remove(path);

			// Then delete from disk
			// Normalize path for Tauri FS (decode URI component)
			const cleanPath = path.startsWith('file://') ? decodeURIComponent(path.slice(7)) : path;

			await fileService.remove(cleanPath);
			return newList;
		} catch (e) {
			console.error('Failed to delete recent doc file', e);
			return await this.get(); // Return current list even if delete fails
		}
	}
};
