import { useState } from 'react';
import { Eye, EyeOff, Lock, Unlock, Plus } from 'lucide-react';
import { useVectorStore } from '@/lib/vector/store';
import type { Layer } from '@/lib/vector/types';
import { cn } from '@/lib/utils';

export function LayersTab() {
    const { document, setDocument, activeLayerIndex, setActiveLayer } = useVectorStore();
    const [editingId, setEditingId] = useState<string | null>(null);
    const [editName, setEditName] = useState('');

    if (!document) return null;

    const updateLayer = (id: string, patch: Partial<Layer>) => {
        const layers = document.layers.map((l) => l.id === id ? { ...l, ...patch } : l);
        setDocument({ ...document, layers });
    };

    const addLayer = () => {
        const newLayer: Layer = {
            id: `layer-${Date.now()}`,
            name: `Layer ${document.layers.length + 1}`,
            visible: true,
            locked: false,
            objects: [],
        };
        setDocument({ ...document, layers: [...document.layers, newLayer] });
        setActiveLayer(document.layers.length);
    };

    return (
        <div className="flex flex-col h-full">
            <div className="flex-1 overflow-y-auto">
                {document.layers.map((layer, index) => (
                    <div
                        key={layer.id}
                        className={cn(
                            'flex items-center gap-1 px-2 py-1.5 cursor-pointer border-b border-border/50',
                            index === activeLayerIndex ? 'bg-accent' : 'hover:bg-muted/50',
                        )}
                        onClick={() => setActiveLayer(index)}
                    >
                        {/* Visibility */}
                        <button
                            onClick={(e) => { e.stopPropagation(); updateLayer(layer.id, { visible: !layer.visible }); }}
                            className="text-muted-foreground hover:text-foreground"
                        >
                            {layer.visible ? <Eye className="h-3.5 w-3.5" /> : <EyeOff className="h-3.5 w-3.5" />}
                        </button>

                        {/* Lock */}
                        <button
                            onClick={(e) => { e.stopPropagation(); updateLayer(layer.id, { locked: !layer.locked }); }}
                            className="text-muted-foreground hover:text-foreground"
                        >
                            {layer.locked ? <Lock className="h-3.5 w-3.5" /> : <Unlock className="h-3.5 w-3.5" />}
                        </button>

                        {/* Name */}
                        {editingId === layer.id ? (
                            <input
                                autoFocus
                                className="flex-1 text-xs bg-background border border-input rounded px-1 py-0.5"
                                value={editName}
                                onChange={(e) => setEditName(e.target.value)}
                                onBlur={() => { updateLayer(layer.id, { name: editName }); setEditingId(null); }}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter') { updateLayer(layer.id, { name: editName }); setEditingId(null); }
                                    if (e.key === 'Escape') setEditingId(null);
                                }}
                                onClick={(e) => e.stopPropagation()}
                            />
                        ) : (
                            <span
                                className="flex-1 text-xs truncate select-none"
                                onDoubleClick={(e) => { e.stopPropagation(); setEditingId(layer.id); setEditName(layer.name); }}
                            >
                                {layer.name}
                            </span>
                        )}

                        <span className="text-[9px] text-muted-foreground">{layer.objects.length}</span>
                    </div>
                ))}
            </div>

            <div className="border-t border-border p-2">
                <button
                    onClick={addLayer}
                    className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground w-full justify-center py-1"
                >
                    <Plus className="h-3.5 w-3.5" />
                    Add Layer
                </button>
            </div>
        </div>
    );
}
