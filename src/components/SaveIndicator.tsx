import { useDocumentStore } from '@/lib/stores/documentStore';
import { Cloud, CloudOff, Loader2 } from 'lucide-react';

export function SaveIndicator() {
    const { isDirty, isSaving, currentPath } = useDocumentStore();

    if (!currentPath) {
        return null; // Don't show indicator for files that have never been saved
    }

    if (isSaving) {
        return (
            <div className="flex items-center text-xs text-slate-500 gap-1 animate-pulse ml-2" title="Saving...">
                <Loader2 className="h-3 w-3 animate-spin" />
                <span className="hidden sm:inline">Saving</span>
            </div>
        );
    }

    if (isDirty) {
        return (
            <div className="flex items-center text-xs text-slate-500 gap-1 ml-2" title="Unsaved changes">
                <CloudOff className="h-3 w-3" />
                <span className="hidden sm:inline">Unsaved</span>
            </div>
        );
    }

    return (
        <div className="flex items-center text-xs text-slate-500 gap-1 ml-2" title="Saved to disk">
            <Cloud className="h-3 w-3" />
            <span className="hidden sm:inline">Saved</span>
        </div>
    );
}
