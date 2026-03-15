import { useState } from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { UnitInput } from '../Properties/UnitInput';
import { newVectorDocument } from '@/lib/vector/commands';
import { useVectorStore } from '@/lib/vector/store';
import { cn } from '@/lib/utils';

type SizePreset = 'a4-portrait' | 'a4-landscape' | 'letter-portrait' | 'custom';
type ColourMode = 'rgb' | 'cmyk-iso' | 'cmyk-swop';

const SIZE_PRESETS: { id: SizePreset; label: string }[] = [
    { id: 'a4-portrait', label: 'A4 Portrait' },
    { id: 'a4-landscape', label: 'A4 Landscape' },
    { id: 'letter-portrait', label: 'US Letter' },
    { id: 'custom', label: 'Custom' },
];

const COLOUR_MODES: { id: ColourMode; label: string; desc: string }[] = [
    { id: 'rgb', label: 'RGB', desc: 'sRGB — for screen and web' },
    { id: 'cmyk-iso', label: 'CMYK (ISO)', desc: 'ISO Coated v2 — European print' },
    { id: 'cmyk-swop', label: 'CMYK (SWOP)', desc: 'SWOP v2 — US print' },
];

function resolvePreset(size: SizePreset, colour: ColourMode): string {
    if (colour === 'rgb') return size;
    // Map CMYK presets: a4 and letter get CMYK variants; others fall back to a4-cmyk
    if (colour === 'cmyk-iso') {
        if (size === 'a4-portrait' || size === 'custom') return 'a4-portrait-cmyk';
        if (size === 'letter-portrait') return 'letter-portrait-cmyk';
        // a4-landscape: use a4-portrait-cmyk then note landscape is not a CMYK preset
        return 'a4-portrait-cmyk';
    }
    // cmyk-swop
    if (size === 'letter-portrait' || size === 'custom') return 'letter-portrait-cmyk';
    return 'a4-portrait-cmyk';
}

interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function NewDocumentDialog({ open, onOpenChange }: Props) {
    const [sizePreset, setSizePreset] = useState<SizePreset>('a4-portrait');
    const [colourMode, setColourMode] = useState<ColourMode>('rgb');
    const [customW, setCustomW] = useState(800);
    const [customH, setCustomH] = useState(600);
    const [loading, setLoading] = useState(false);
    const { setDocument, reset } = useVectorStore();

    const handleCreate = async () => {
        setLoading(true);
        try {
            const preset = resolvePreset(sizePreset, colourMode);
            const doc = await newVectorDocument(
                preset,
                sizePreset === 'custom' ? customW : undefined,
                sizePreset === 'custom' ? customH : undefined,
            );
            reset();
            setDocument(doc);
            onOpenChange(false);
        } catch (e) {
            console.error('Failed to create document', e);
        } finally {
            setLoading(false);
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle>New Vector Document</DialogTitle>
                </DialogHeader>

                <div className="space-y-5 py-2">
                    {/* Size presets */}
                    <div className="space-y-2">
                        <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">Size</p>
                        <div className="grid grid-cols-2 gap-2">
                            {SIZE_PRESETS.map((p) => (
                                <button
                                    key={p.id}
                                    onClick={() => setSizePreset(p.id)}
                                    className={cn(
                                        'py-3 px-4 rounded-lg border text-sm font-medium transition-colors',
                                        sizePreset === p.id
                                            ? 'border-primary bg-primary/10 text-primary'
                                            : 'border-border hover:border-muted-foreground',
                                    )}
                                >
                                    {p.label}
                                </button>
                            ))}
                        </div>

                        {sizePreset === 'custom' && (
                            <div className="grid grid-cols-2 gap-3 pt-1">
                                <UnitInput
                                    label="Width"
                                    value={customW}
                                    unit="Px"
                                    min={1}
                                    onChange={setCustomW}
                                />
                                <UnitInput
                                    label="Height"
                                    value={customH}
                                    unit="Px"
                                    min={1}
                                    onChange={setCustomH}
                                />
                            </div>
                        )}
                    </div>

                    {/* Colour mode */}
                    <div className="space-y-2">
                        <p className="text-xs font-medium text-muted-foreground uppercase tracking-wide">Colour Mode</p>
                        <div className="space-y-1.5">
                            {COLOUR_MODES.map((m) => (
                                <label
                                    key={m.id}
                                    className="flex items-start gap-3 p-2.5 rounded border border-border cursor-pointer hover:border-primary transition-colors"
                                >
                                    <input
                                        type="radio"
                                        name="colour-mode-new"
                                        value={m.id}
                                        checked={colourMode === m.id}
                                        onChange={() => setColourMode(m.id)}
                                        className="mt-0.5"
                                    />
                                    <div>
                                        <p className="text-sm font-medium">{m.label}</p>
                                        <p className="text-xs text-muted-foreground">{m.desc}</p>
                                    </div>
                                </label>
                            ))}
                        </div>
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
                    <Button onClick={handleCreate} disabled={loading}>
                        {loading ? 'Creating…' : 'Create'}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
