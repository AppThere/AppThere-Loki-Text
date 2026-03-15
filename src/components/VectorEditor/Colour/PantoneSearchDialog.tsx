import { useState, useEffect, useRef } from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import type { ColourSwatch } from '@/lib/vector/types';
import { searchPantone } from '@/lib/vector/commands';

interface SearchResult {
    name: string;
    lab_ref: [number, number, number];
}

interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSelect: (swatch: ColourSwatch) => void;
}

let nextId = 1;

function makeSpotSwatch(name: string, lab_ref: [number, number, number]): ColourSwatch {
    return {
        id: { id: `pantone-${Date.now()}-${nextId++}` },
        name,
        colour: {
            type: 'Spot',
            name,
            tint: 1.0,
            lab_ref,
            cmyk_fallback: {
                type: 'Cmyk',
                c: 0, m: 0, y: 0, k: 0, alpha: 1,
            },
        },
        is_spot: true,
    };
}

/** Approximate Lab → sRGB conversion for preview swatches (D65 illuminant). */
function labToRgb(l: number, a: number, b: number): [number, number, number] {
    // Lab → XYZ
    const fy = (l + 16) / 116;
    const fx = a / 500 + fy;
    const fz = fy - b / 200;
    const delta = 6 / 29;
    const f = (t: number) => t > delta ? t ** 3 : 3 * delta ** 2 * (t - 4 / 29);
    const x = f(fx) * 0.95047;
    const y = f(fy);
    const z = f(fz) * 1.08883;

    // XYZ → linear sRGB
    const lr =  3.2404542 * x - 1.5371385 * y - 0.4985314 * z;
    const lg = -0.9692660 * x + 1.8760108 * y + 0.0415560 * z;
    const lb =  0.0556434 * x - 0.2040259 * y + 1.0572252 * z;

    // Linear → gamma
    const gc = (c: number) => c <= 0.0031308 ? 12.92 * c : 1.055 * c ** (1 / 2.4) - 0.055;
    return [
        Math.round(Math.max(0, Math.min(1, gc(lr))) * 255),
        Math.round(Math.max(0, Math.min(1, gc(lg))) * 255),
        Math.round(Math.max(0, Math.min(1, gc(lb))) * 255),
    ];
}

export function PantoneSearchDialog({ open, onOpenChange, onSelect }: Props) {
    const [query, setQuery] = useState('');
    const [results, setResults] = useState<SearchResult[]>([]);
    const [loading, setLoading] = useState(false);
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => {
        if (!open) { setQuery(''); setResults([]); return; }
    }, [open]);

    useEffect(() => {
        if (timerRef.current) clearTimeout(timerRef.current);
        if (!query.trim()) { setResults([]); return; }
        timerRef.current = setTimeout(async () => {
            setLoading(true);
            try {
                const r = await searchPantone(query);
                setResults(r);
            } catch (e) {
                console.error('[PantoneSearchDialog] search failed', e);
            } finally {
                setLoading(false);
            }
        }, 250);
        return () => { if (timerRef.current) clearTimeout(timerRef.current); };
    }, [query]);

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle>Search Pantone Colours</DialogTitle>
                </DialogHeader>

                <input
                    type="search"
                    placeholder="e.g. 186 C"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    className="w-full h-9 rounded border border-input bg-background px-3 text-sm"
                    autoFocus
                />

                <div className="max-h-72 overflow-y-auto space-y-1 mt-1">
                    {loading && (
                        <p className="text-xs text-muted-foreground text-center py-4">Searching…</p>
                    )}
                    {!loading && results.length === 0 && query.trim() && (
                        <p className="text-xs text-muted-foreground text-center py-4">No results.</p>
                    )}
                    {results.map((r) => {
                        const [rv, gv, bv] = labToRgb(r.lab_ref[0], r.lab_ref[1], r.lab_ref[2]);
                        const css = `rgb(${rv},${gv},${bv})`;
                        return (
                            <button
                                key={r.name}
                                className="w-full flex items-center gap-3 px-3 py-2 rounded hover:bg-muted transition-colors text-left min-h-[44px]"
                                onClick={() => onSelect(makeSpotSwatch(r.name, r.lab_ref))}
                            >
                                <div
                                    className="w-7 h-7 rounded border border-input shrink-0"
                                    style={{ background: css }}
                                />
                                <span className="text-xs flex-1 truncate">{r.name}</span>
                                <span className="text-[10px] text-muted-foreground shrink-0">
                                    L*{r.lab_ref[0].toFixed(0)}
                                </span>
                            </button>
                        );
                    })}
                </div>
            </DialogContent>
        </Dialog>
    );
}
