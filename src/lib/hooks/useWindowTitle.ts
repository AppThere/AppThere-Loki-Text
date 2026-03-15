import { useEffect } from 'react';
import { useDocumentStore } from '../stores/documentStore';

/**
 * Derives the window title from document state.
 * Priority: metadata.title > filename from path > "Untitled Document"
 * Appends " *" when the document has unsaved changes.
 */
export function deriveWindowTitle(
    title: string | null | undefined,
    path: string | null | undefined,
    isDirty: boolean,
): string {
    let base: string;
    if (title && title.trim()) {
        base = title.trim();
    } else if (path && path.trim()) {
        const parts = path.replace(/\\/g, '/').split('/');
        base = parts[parts.length - 1] || 'Untitled Document';
    } else {
        base = 'Untitled Document';
    }
    return isDirty ? `${base} *` : base;
}

/**
 * Sets the Tauri window title whenever document state changes.
 * Uses a dynamic import so the hook remains testable under jsdom.
 */
export function useWindowTitle(): void {
    const { metadata, currentPath, isDirty } = useDocumentStore();

    useEffect(() => {
        const title = deriveWindowTitle(metadata?.title, currentPath, isDirty);
        import('@tauri-apps/api/window')
            .then(({ getCurrentWindow }) => {
                getCurrentWindow().setTitle(title).catch(() => {/* ignore in non-Tauri env */});
            })
            .catch(() => {/* ignore in non-Tauri env */});
    }, [metadata?.title, currentPath, isDirty]);
}
