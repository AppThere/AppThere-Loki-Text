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

type Preset = 'a4-portrait' | 'a4-landscape' | 'letter-portrait' | 'custom';

const PRESETS: { id: Preset; label: string }[] = [
    { id: 'a4-portrait', label: 'A4 Portrait' },
    { id: 'a4-landscape', label: 'A4 Landscape' },
    { id: 'letter-portrait', label: 'US Letter' },
    { id: 'custom', label: 'Custom' },
];

interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function NewDocumentDialog({ open, onOpenChange }: Props) {
    const [preset, setPreset] = useState<Preset>('a4-portrait');
    const [customW, setCustomW] = useState(800);
    const [customH, setCustomH] = useState(600);
    const [loading, setLoading] = useState(false);
    const { setDocument, reset } = useVectorStore();

    const handleCreate = async () => {
        setLoading(true);
        try {
            const doc = await newVectorDocument(
                preset,
                preset === 'custom' ? customW : undefined,
                preset === 'custom' ? customH : undefined,
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

                <div className="space-y-4 py-2">
                    <div className="grid grid-cols-2 gap-2">
                        {PRESETS.map((p) => (
                            <button
                                key={p.id}
                                onClick={() => setPreset(p.id)}
                                className={cn(
                                    'py-3 px-4 rounded-lg border text-sm font-medium transition-colors',
                                    preset === p.id
                                        ? 'border-primary bg-primary/10 text-primary'
                                        : 'border-border hover:border-muted-foreground',
                                )}
                            >
                                {p.label}
                            </button>
                        ))}
                    </div>

                    {preset === 'custom' && (
                        <div className="grid grid-cols-2 gap-3">
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
