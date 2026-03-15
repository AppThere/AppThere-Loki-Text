import { useCallback } from 'react';
import { open as dialogOpen, save as dialogSave } from '@tauri-apps/plugin-dialog';
import { useVectorStore } from './store';
import { openVectorDocument, saveVectorDocument } from './commands';
import { toast } from '../hooks/useToast';
import { useHistoryStore } from '../stores/historyStore';

export function useVectorFileOps() {
    const { document, currentPath, setDocument, setPath, markClean } = useVectorStore();
    const { addDocument } = useHistoryStore();

    const handleOpen = useCallback(async () => {
        try {
            const selected = await dialogOpen({
                multiple: false,
                filters: [{ name: 'SVG Vector', extensions: ['svg'] }],
            });
            if (!selected || typeof selected !== 'string') return;
            const doc = await openVectorDocument(selected);
            setDocument(doc);
            setPath(selected);
            const name = selected.split('/').pop() ?? 'Untitled';
            addDocument({ path: selected, name, type: 'vector' });
        } catch (e) {
            toast({ title: 'Open failed', description: String(e), variant: 'destructive' });
        }
    }, [setDocument, setPath, toast, addDocument]);

    const handleSave = useCallback(async () => {
        if (!document) return;
        if (!currentPath) {
            await handleSaveAs();
            return;
        }
        try {
            await saveVectorDocument(currentPath, document);
            markClean();
        } catch (e) {
            toast({ title: 'Save failed', description: String(e), variant: 'destructive' });
        }
    }, [document, currentPath, markClean, toast]);

    const handleSaveAs = useCallback(async () => {
        if (!document) return;
        try {
            const path = await dialogSave({
                defaultPath: 'untitled.svg',
                filters: [{ name: 'SVG Vector', extensions: ['svg'] }],
            });
            if (!path) return;
            await saveVectorDocument(path, document);
            setPath(path);
            markClean();
            const name = path.split('/').pop() ?? 'Untitled';
            addDocument({ path, name, type: 'vector' });
        } catch (e) {
            toast({ title: 'Save failed', description: String(e), variant: 'destructive' });
        }
    }, [document, setPath, markClean, toast, addDocument]);

    return { handleOpen, handleSave, handleSaveAs };
}
