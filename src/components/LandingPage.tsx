import { FileText, Table, Shapes, Presentation, Image as ImageIcon, Clock, SquarePlus, Library, X } from 'lucide-react';
import { useHistoryStore } from '@/lib/stores/historyStore';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface LandingPageProps {
    onOpenClick: () => void;
    onOpenTemplateClick: () => void;
    onNewClick: () => void;
    loadDocument: (path: string) => Promise<void>;
    onNewVector?: () => void;
}

const DOCUMENT_TYPES = [
    {
        id: 'text',
        label: 'Text Document',
        shortLabel: 'Text',
        icon: FileText,
        color: 'text-white',
        bg: 'bg-blue-600 dark:bg-blue-700',
        hoverBg: 'hover:bg-blue-500 dark:hover:bg-blue-600',
    },
    {
        id: 'spreadsheet',
        label: 'Spreadsheet',
        shortLabel: 'Sheets',
        icon: Table,
        color: 'text-white',
        bg: 'bg-green-600 dark:bg-green-700',
        hoverBg: 'hover:bg-green-500 dark:hover:bg-green-600',
    },
    {
        id: 'vector',
        label: 'Vector Image',
        shortLabel: 'Vector',
        icon: Shapes,
        color: 'text-white',
        bg: 'bg-orange-600 dark:bg-orange-700',
        hoverBg: 'hover:bg-orange-500 dark:hover:bg-orange-600',
    },
    {
        id: 'presentation',
        label: 'Presentation',
        shortLabel: 'Slides',
        icon: Presentation,
        color: 'text-white',
        bg: 'bg-red-600 dark:bg-red-700',
        hoverBg: 'hover:bg-red-500 dark:hover:bg-red-600',
    },
    {
        id: 'image',
        label: 'Photo / Image',
        shortLabel: 'Images',
        icon: ImageIcon,
        color: 'text-white',
        bg: 'bg-indigo-600 dark:bg-indigo-700',
        hoverBg: 'hover:bg-indigo-500 dark:hover:bg-indigo-600',
    },
];

export function LandingPage({ onOpenClick, onOpenTemplateClick, onNewClick, loadDocument, onNewVector }: LandingPageProps) {
    const { recentDocuments, removeDocument } = useHistoryStore();

    const formatTimestamp = (ts: number) => {
        const now = Date.now();
        const diff = now - ts;
        if (diff < 60000) return 'Just now';
        if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`;
        if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`;
        return new Date(ts).toLocaleDateString();
    };

    return (
        <div className="flex flex-col h-full bg-background transition-colors duration-300 overflow-y-auto">
            {/* Header */}
            <div className="px-6 py-8 md:px-12 md:py-12 shrink-0 safe-pt">
                <h1 className="text-3xl font-extrabold tracking-tight lg:text-4xl text-foreground">AppThere Loki</h1>
            </div>

            {/* Create New Grid */}
            <div className="px-6 md:px-12 space-y-4">
                <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60 px-1">Start Something New</h2>
                <div className="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-4 md:grid-cols-5 gap-4">
                    {DOCUMENT_TYPES.map((type) => (
                        <div key={type.id} className={cn(
                            "group flex flex-col rounded-2xl overflow-hidden border border-transparent shadow-sm transition-all",
                            type.bg,
                            "hover:shadow-md"
                        )}>
                            {/* Icon Area */}
                            <div className="flex-1 flex flex-col items-center justify-center p-4">
                                <type.icon className={cn("h-8 w-8 mb-2", type.color)} />
                                <span className={cn("text-xs font-bold tracking-tight", type.color)}>
                                    {type.shortLabel}
                                </span>
                            </div>

                            {/* Action Buttons */}
                            <div className="bg-white/10 grid grid-cols-2 divide-x divide-white/10">
                                <Button
                                    variant="ghost"
                                    className="h-10 rounded-none text-white hover:bg-white/10 border-0"
                                    onClick={type.id === 'text' ? onNewClick : type.id === 'vector' ? onNewVector : undefined}
                                    title="New Blank"
                                >
                                    <SquarePlus className="h-4 w-4" />
                                </Button>
                                <Button
                                    variant="ghost"
                                    className="h-10 rounded-none text-white hover:bg-white/10 border-0"
                                    onClick={onOpenTemplateClick}
                                    title="From Template"
                                >
                                    <Library className="h-4 w-4" />
                                </Button>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Recent Documents */}
            <div className="px-6 md:px-12 py-12 flex-1 flex flex-col min-h-0">
                <div className="flex items-center justify-between mb-4">
                    <h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60 px-1">Recent Documents</h2>
                    <Button variant="ghost" size="sm" className="text-xs h-7 text-muted-foreground hover:text-foreground" onClick={onOpenClick}>
                        Open Existing...
                    </Button>
                </div>

                {recentDocuments.length > 0 ? (
                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
                        {recentDocuments.map((doc) => (
                            <div
                                key={doc.path}
                                className="group relative flex items-center p-3 rounded-xl border border-border bg-card hover:border-blue-500/50 hover:shadow-sm cursor-pointer transition-all"
                                onClick={async () => {
                                    try {
                                        await loadDocument(doc.path);
                                    } catch (error) {
                                        // If the file is missing or corrupted, offer to remove it
                                        if (window.confirm(`Failed to load "${doc.name}". This file may have been moved or deleted. Would you like to remove it from your recents?`)) {
                                            removeDocument(doc.path);
                                        }
                                    }
                                }}
                            >
                                <div className="h-10 w-10 shrink-0 rounded-lg bg-slate-100 dark:bg-slate-800 flex items-center justify-center text-slate-400 group-hover:text-blue-500 transition-colors">
                                    <FileText className="h-6 w-6" />
                                </div>
                                <div className="ml-3 min-w-0 flex-1">
                                    <h4 className="text-sm font-semibold truncate leading-tight text-foreground">{doc.name}</h4>
                                    <p className="text-[10px] text-muted-foreground mt-0.5 truncate flex items-center">
                                        <Clock className="h-3 w-3 mr-1 inline" />
                                        {formatTimestamp(doc.timestamp)} • {doc.path.split('/').pop()}
                                    </p>
                                </div>

                                <Button
                                    variant="ghost"
                                    size="icon"
                                    className="absolute top-2 right-2 h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity hover:bg-slate-100 dark:hover:bg-slate-800 text-muted-foreground hover:text-destructive"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        removeDocument(doc.path);
                                    }}
                                    title="Remove from Recents"
                                >
                                    <X className="h-3 w-3" />
                                </Button>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className="flex-1 flex flex-col items-center justify-center border-2 border-dashed border-muted rounded-2xl py-12 opacity-40">
                        <Clock className="h-12 w-12 text-muted-foreground mb-4" />
                        <p className="text-sm font-medium">No recent documents found.</p>
                        <p className="text-xs">Your recently opened files will appear here.</p>
                    </div>
                )}
            </div>

            {/* Footer space */}
            <div className="h-12 shrink-0 safe-pb" />
        </div>
    );
}
