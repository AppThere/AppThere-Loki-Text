import type { Editor } from '@tiptap/core';
import type { FileService } from '../services/FileService';

export async function insertImageAction(editor: Editor, fileService: FileService) {
	try {
		const selected = await fileService.promptOpen({
			multiple: false,
			filters: [
				{
					name: 'Images',
					extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp']
				}
			]
		});
		if (selected) {
			const path = Array.isArray(selected) ? selected[0] : selected;

			try {
				// Strategy: Read file directly and use Data URI to bypass asset:// protocol issues
				const contents = await fileService.readBinaryFile(path);

				// Determine mime type from extension
				const ext = path.split('.').pop()?.toLowerCase() || 'png';
				const mimeType =
					ext === 'jpg' || ext === 'jpeg'
						? 'image/jpeg'
						: ext === 'gif'
							? 'image/gif'
							: ext === 'webp'
								? 'image/webp'
								: 'image/png';

				// distinct approach for buffer conversion to avoid stack overflow on large files
				const blob = new Blob([contents as any], {
					type: mimeType
				});
				const reader = new FileReader();

				reader.onload = (e) => {
					const src = e.target?.result as string;
					console.log('Image loaded as Data URI', {
						path,
						length: src.length
					});
					editor.chain().focus().setImage({ src, alt: path }).run();
				};

				reader.readAsDataURL(blob);
			} catch (readErr) {
				console.error('Failed to read image file', readErr);
				// Fallback to old method just in case
				const src = `asset://localhost${encodeURI(path)}`;
				editor.chain().focus().setImage({ src, alt: path }).run();
			}
		}
	} catch (e) {
		console.error('Failed to insert image', e);
	}
}

export function insertTableAction(editor: Editor) {
	const rows = prompt('Rows:', '3');
	const cols = prompt('Columns:', '3');
	if (rows && cols) {
		editor
			.chain()
			.focus()
			.insertTable({
				rows: parseInt(rows),
				cols: parseInt(cols),
				withHeaderRow: true
			})
			.run();
	}
}
