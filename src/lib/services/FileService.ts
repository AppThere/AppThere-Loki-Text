import type { TiptapResponse, Metadata, StyleDefinition } from '../types';
import { addDebugLog } from '../debugStore';

export class FileService {
	async saveDocument(
		path: string,
		tiptapJson: string,
		styles: Record<string, StyleDefinition>,
		metadata: Metadata
	): Promise<void> {
		addDebugLog(`FileService: Saving to ${path}`);
		const { invoke } = await import('@tauri-apps/api/core');
		const { writeFile } = await import('@tauri-apps/plugin-fs');

		// Backend: save_document(path, tiptap_json, styles, metadata)
		const result = await invoke<number[] | null>('save_document', {
			path,
			tiptapJson: tiptapJson,
			styles,
			metadata
		});

		if (result && path.startsWith('content://')) {
			addDebugLog(`FileService: Received ${result.length} bytes for content URI write`);
			await writeFile(path, new Uint8Array(result));
			addDebugLog('FileService: Wrote bytes to content URI via plugin-fs');
		} else {
			if (result) {
				addDebugLog('FileService: Received bytes but path is not content://?');
			}
		}
	}

	async openDocument(path: string, fileContent?: Uint8Array): Promise<TiptapResponse> {
		const { invoke } = await import('@tauri-apps/api/core');
		let content = fileContent;
		if (!content && path.startsWith('content://')) {
			try {
				addDebugLog(`FileService: Reading content:// URI: ${path}`);
				content = await this.readBinaryFile(path);
				addDebugLog(`FileService: Read ${content.length} bytes from content URI`);
			} catch (e) {
				addDebugLog(`FileService: Failed to read content:// URI: ${e}`);
				throw e;
			}
		}

		addDebugLog(
			`FileService: Invoking open_document with path and ${content ? content.length : 'null'} bytes`
		);
		// Backend: open_document(path, file_content)
		// arg name in Rust is file_content, so JS key must be fileContent
		return await invoke<TiptapResponse>('open_document', {
			path,
			fileContent: content ? Array.from(content) : null
		});
	}

	async saveEpub(
		path: string,
		tiptapJson: string,
		styles: Record<string, StyleDefinition>,
		metadata: Metadata,
		fontPaths: string[]
	): Promise<void> {
		addDebugLog(`FileService: Exporting EPUB to ${path}`);
		const { invoke } = await import('@tauri-apps/api/core');
		const { writeFile } = await import('@tauri-apps/plugin-fs');

		const result = await invoke<number[] | null>('save_epub', {
			path,
			tiptapJson: tiptapJson,
			styles,
			metadata,
			fontPaths: fontPaths
		});

		if (result && path.startsWith('content://')) {
			addDebugLog(`FileService: Received ${result.length} bytes for EPUB content URI write`);
			await writeFile(path, new Uint8Array(result));
		}
	}

	// Wrappers for tauri-plugin-fs
	async readBinaryFile(path: string): Promise<Uint8Array> {
		const { readFile } = await import('@tauri-apps/plugin-fs');
		return await readFile(path);
	}

	async readTextFile(path: string, options?: any): Promise<string> {
		const { readTextFile } = await import('@tauri-apps/plugin-fs');
		return await readTextFile(path, options);
	}

	async writeTextFile(path: string, contents: string, options?: any): Promise<void> {
		const { writeTextFile } = await import('@tauri-apps/plugin-fs');
		await writeTextFile(path, contents, options);
	}

	async mkdir(path: string, options?: any): Promise<void> {
		const { mkdir } = await import('@tauri-apps/plugin-fs');
		await mkdir(path, options);
	}

	async remove(path: string, options?: any): Promise<void> {
		const { remove } = await import('@tauri-apps/plugin-fs');
		await remove(path, options);
	}

	async exists(path: string, options?: any): Promise<boolean> {
		const { exists } = await import('@tauri-apps/plugin-fs');
		return await exists(path, options);
	}

	// Dialog wrappers
	async promptSave(options: any): Promise<string | null> {
		const { save } = await import('@tauri-apps/plugin-dialog');
		return await save(options);
	}

	async promptOpen(options: any): Promise<string | string[] | null> {
		const { open } = await import('@tauri-apps/plugin-dialog');
		return await open(options);
	}

	async confirm(message: string, options?: any): Promise<boolean> {
		const { ask } = await import('@tauri-apps/plugin-dialog');
		return await ask(message, options);
	}
	async syncDocument(
		tiptapJson: string,
		styles: Record<string, StyleDefinition>,
		metadata: Metadata
	): Promise<void> {
		const { invoke } = await import('@tauri-apps/api/core');
		await invoke('sync_document', {
			tiptapJson: tiptapJson,
			styles,
			metadata
		});
	}
}

export const fileService = new FileService();
