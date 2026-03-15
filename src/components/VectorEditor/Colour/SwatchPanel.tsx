import { useState, useRef } from 'react';
import { useVectorStore, selectSwatchLibrary } from '@/lib/vector/store';
import type { ColourSwatch, Colour } from '@/lib/vector/types';
import { displayRgbToCss, getDisplayRgba, colourSpaceBadge } from '@/lib/vector/colourUtils';
import { PantoneSearchDialog } from './PantoneSearchDialog';

interface Props {
    displayCache: Map<string, string>;
    /** Colour to add when "Add from fill" is clicked */
    fillColour: Colour | null;
    /** Colour to add when "Add from stroke" is clicked */
    strokeColour: Colour | null;
    /** Called when a swatch is clicked (apply to current selection) */
    onApply: (colour: Colour) => void;
}

let nextId = 1;
function makeId(): string {
    return `swatch-${Date.now()}-${nextId++}`;
}

function makeSwatch(colour: Colour, name: string): ColourSwatch {
    return {
        id: { id: makeId() },
        name,
        colour,
        is_spot: colour.type === 'Spot',
    };
}

function SwatchItem({
    swatch,
    displayCache,
    onApply,
    onDelete,
    onDragStart,
    onDragOver,
    onDrop,
}: {
    swatch: ColourSwatch;
    displayCache: Map<string, string>;
    onApply: (c: Colour) => void;
    onDelete: (id: string) => void;
    onDragStart: (id: string) => void;
    onDragOver: (id: string) => void;
    onDrop: () => void;
}) {
    const rgba = getDisplayRgba(swatch.colour, displayCache);
    const css = displayRgbToCss(rgba);
    const badge = colourSpaceBadge(swatch.colour);

    return (
        <div
            className="relative group cursor-pointer rounded border border-transparent hover:border-primary transition-colors"
            draggable
            onDragStart={() => onDragStart(swatch.id.id)}
            onDragOver={(e) => { e.preventDefault(); onDragOver(swatch.id.id); }}
            onDrop={() => onDrop()}
            onClick={() => onApply(swatch.colour)}
            title={`${swatch.name} (${badge})`}
        >
            <div
                className="w-full aspect-square rounded"
                style={{ background: css }}
            />
            <span className="absolute bottom-0 left-0 right-0 text-[8px] text-center bg-background/80 rounded-b truncate px-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                {swatch.name}
            </span>
            <button
                className="absolute top-0 right-0 hidden group-hover:flex items-center justify-center w-4 h-4 text-[9px] bg-destructive text-destructive-foreground rounded-bl rounded-tr"
                onClick={(e) => { e.stopPropagation(); onDelete(swatch.id.id); }}
                aria-label={`Delete ${swatch.name}`}
            >
                ×
            </button>
        </div>
    );
}

export function SwatchPanel({ displayCache, fillColour, strokeColour, onApply }: Props) {
    const { updateSwatchLibrary } = useVectorStore();
    const library = useVectorStore(selectSwatchLibrary);
    const [pantoneOpen, setPantoneOpen] = useState(false);
    const dragId = useRef<string | null>(null);
    const overId = useRef<string | null>(null);

    const updateLib = (swatches: ColourSwatch[]) => {
        updateSwatchLibrary({ swatches });
    };

    const addSwatch = (colour: Colour, name: string) => {
        const swatch = makeSwatch(colour, name);
        updateLib([...library.swatches, swatch]);
    };

    const deleteSwatch = (id: string) => {
        updateLib(library.swatches.filter((s) => s.id.id !== id));
    };

    const handleDrop = () => {
        const from = dragId.current;
        const to = overId.current;
        dragId.current = null;
        overId.current = null;
        if (!from || !to || from === to) return;
        const swatches = [...library.swatches];
        const fromIdx = swatches.findIndex((s) => s.id.id === from);
        const toIdx = swatches.findIndex((s) => s.id.id === to);
        if (fromIdx < 0 || toIdx < 0) return;
        const [item] = swatches.splice(fromIdx, 1);
        swatches.splice(toIdx, 0, item);
        updateLib(swatches);
    };

    const addFromColour = (colour: Colour | null, label: string) => {
        if (!colour) return;
        const name = label + ' colour';
        addSwatch(colour, name);
    };

    return (
        <div className="p-2 space-y-2">
            {/* Action buttons */}
            <div className="flex flex-wrap gap-1">
                <button
                    className="text-[10px] px-2 py-1 rounded border border-input hover:border-primary transition-colors"
                    onClick={() => addFromColour(fillColour, 'Fill')}
                    disabled={!fillColour}
                >
                    + Fill
                </button>
                <button
                    className="text-[10px] px-2 py-1 rounded border border-input hover:border-primary transition-colors"
                    onClick={() => addFromColour(strokeColour, 'Stroke')}
                    disabled={!strokeColour}
                >
                    + Stroke
                </button>
                <button
                    className="text-[10px] px-2 py-1 rounded border border-input hover:border-primary transition-colors"
                    onClick={() => setPantoneOpen(true)}
                >
                    Pantone…
                </button>
            </div>

            {/* Swatch grid */}
            {library.swatches.length === 0 ? (
                <p className="text-[10px] text-muted-foreground text-center py-4">
                    No swatches yet.
                </p>
            ) : (
                <div className="grid grid-cols-5 gap-1">
                    {library.swatches.map((swatch) => (
                        <SwatchItem
                            key={swatch.id.id}
                            swatch={swatch}
                            displayCache={displayCache}
                            onApply={onApply}
                            onDelete={deleteSwatch}
                            onDragStart={(id) => { dragId.current = id; }}
                            onDragOver={(id) => { overId.current = id; }}
                            onDrop={handleDrop}
                        />
                    ))}
                </div>
            )}

            <PantoneSearchDialog
                open={pantoneOpen}
                onOpenChange={setPantoneOpen}
                onSelect={(swatch) => {
                    updateLib([...library.swatches, swatch]);
                    setPantoneOpen(false);
                }}
            />
        </div>
    );
}
