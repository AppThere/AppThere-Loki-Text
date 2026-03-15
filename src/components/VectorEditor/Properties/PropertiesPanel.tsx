import { useRef, useState } from 'react';
import { useVectorStore } from '@/lib/vector/store';
import { FillStrokeTab } from './FillStrokeTab';
import { TransformTab } from './TransformTab';
import { LayersTab } from './LayersTab';
import type { VectorObject } from '@/lib/vector/types';
import { cn } from '@/lib/utils';

type PanelTab = 'object' | 'layers';

interface Props {
    variant: 'sidebar' | 'bottomsheet';
}

export function PropertiesPanel({ variant }: Props) {
    const { document: doc, selectedIds } = useVectorStore();
    const [activeTab, setActiveTab] = useState<PanelTab>('object');

    // Bottom sheet drag state
    const [sheetOpen, setSheetOpen] = useState(false);
    const dragRef = useRef<{ startY: number; startOpen: boolean } | null>(null);

    const selectedObj: VectorObject | null = (() => {
        if (selectedIds.size !== 1 || !doc) return null;
        const id = [...selectedIds][0];
        for (const layer of doc.layers) {
            const obj = layer.objects.find((o) => o.id === id);
            if (obj) return obj;
        }
        return null;
    })();

    const tabBtn = (tab: PanelTab, label: string) => (
        <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={cn(
                'flex-1 py-2 text-xs font-medium border-b-2 transition-colors',
                activeTab === tab
                    ? 'border-primary text-primary'
                    : 'border-transparent text-muted-foreground hover:text-foreground',
            )}
        >
            {label}
        </button>
    );

    const content = (
        <div className="flex flex-col h-full overflow-hidden">
            <div className="flex border-b border-border shrink-0">
                {tabBtn('object', 'Object')}
                {tabBtn('layers', 'Layers')}
            </div>
            <div className="flex-1 overflow-y-auto">
                {activeTab === 'object' ? (
                    selectedObj ? (
                        <>
                            <TransformTab obj={selectedObj} />
                            <div className="border-t border-border" />
                            <FillStrokeTab obj={selectedObj} />
                        </>
                    ) : (
                        <div className="flex items-center justify-center h-24 text-xs text-muted-foreground px-4 text-center">
                            Select an object to edit its properties.
                        </div>
                    )
                ) : (
                    <LayersTab />
                )}
            </div>
        </div>
    );

    if (variant === 'sidebar') {
        return (
            <div className="w-[280px] border-l border-border bg-background h-full overflow-hidden">
                {content}
            </div>
        );
    }

    // Bottom sheet
    const handlePointerDown = (e: React.PointerEvent) => {
        dragRef.current = { startY: e.clientY, startOpen: sheetOpen };
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    };
    const handlePointerMove = (e: React.PointerEvent) => {
        if (!dragRef.current) return;
        const dy = dragRef.current.startY - e.clientY;
        if (dy > 30) setSheetOpen(true);
        if (dy < -30) setSheetOpen(false);
    };
    const handlePointerUp = () => { dragRef.current = null; };

    return (
        <div
            className={cn(
                'fixed bottom-0 left-0 right-0 bg-background border-t border-border z-10',
                'transition-transform duration-300',
                sheetOpen ? 'h-[40vh]' : 'h-12',
            )}
            style={{ transform: 'none' }}
        >
            {/* Drag handle */}
            <div
                className="flex items-center justify-center h-12 shrink-0 cursor-row-resize"
                onPointerDown={handlePointerDown}
                onPointerMove={handlePointerMove}
                onPointerUp={handlePointerUp}
                onClick={() => setSheetOpen(!sheetOpen)}
            >
                <div className="w-10 h-1 bg-muted-foreground/30 rounded-full" />
                <span className="ml-2 text-xs text-muted-foreground">
                    {selectedObj ? selectedObj.type : 'Properties'}
                </span>
            </div>
            {sheetOpen && <div className="flex-1 h-[calc(40vh-3rem)] overflow-hidden">{content}</div>}
        </div>
    );
}
