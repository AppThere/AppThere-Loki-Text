import { useState, useEffect } from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { useVectorStore } from '@/lib/vector/store';
import type { DocumentColourSettings, ColourPreviewPair, ProfileInfo } from '@/lib/vector/types';
import {
    convertDocumentColourMode,
    getOutputIntentProfiles,
    previewColourConversion,
} from '@/lib/vector/commands';
import { defaultColourSettings, cmykSettings } from '@/lib/vector/colourUtils';
import { ColourModePreview } from '../Colour/ColourModePreview';

type Step = 'pick' | 'preview' | 'confirm';
type ModeChoice = 'rgb' | 'cmyk-iso' | 'cmyk-swop';

interface Props {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

function targetSettingsFor(mode: ModeChoice): DocumentColourSettings {
    if (mode === 'rgb') return defaultColourSettings();
    if (mode === 'cmyk-iso') return cmykSettings('IsoCoatedV2');
    return cmykSettings('SwopV2');
}

export function ColourModeDialog({ open, onOpenChange }: Props) {
    const { document: doc, setDocument } = useVectorStore();
    const [step, setStep] = useState<Step>('pick');
    const [mode, setMode] = useState<ModeChoice>('rgb');
    const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
    const [pairs, setPairs] = useState<ColourPreviewPair[]>([]);
    const [loading, setLoading] = useState(false);
    const [warnings, setWarnings] = useState<string[]>([]);

    // Load profiles once on open
    useEffect(() => {
        if (!open) { setStep('pick'); setPairs([]); setWarnings([]); return; }
        getOutputIntentProfiles().then(setProfiles).catch(console.error);
    }, [open]);

    const handlePreview = async () => {
        if (!doc) return;
        setLoading(true);
        try {
            const target = targetSettingsFor(mode);
            const result = await previewColourConversion(doc, target);
            setPairs(result);
            setStep('preview');
        } catch (e) {
            console.error('[ColourModeDialog] preview failed', e);
        } finally {
            setLoading(false);
        }
    };

    const handleConvert = async () => {
        if (!doc) return;
        setLoading(true);
        try {
            const target = targetSettingsFor(mode);
            const { document: newDoc, warnings: ws } = await convertDocumentColourMode(doc, target);
            setDocument(newDoc);
            if (ws.length > 0) {
                setWarnings(ws.map((w) => `${w.object_id} ${w.property}: ${w.message}`));
                setStep('confirm');
            } else {
                onOpenChange(false);
            }
        } catch (e) {
            console.error('[ColourModeDialog] convert failed', e);
        } finally {
            setLoading(false);
        }
    };

    const modeOpts: { id: ModeChoice; label: string; desc: string }[] = [
        { id: 'rgb', label: 'RGB (sRGB)', desc: 'Standard screen colour space.' },
        { id: 'cmyk-iso', label: 'CMYK — ISO Coated v2', desc: 'European offset printing standard.' },
        { id: 'cmyk-swop', label: 'CMYK — SWOP v2', desc: 'US web offset printing standard.' },
    ];

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-lg">
                <DialogHeader>
                    <DialogTitle>Document Colour Mode</DialogTitle>
                </DialogHeader>

                {step === 'pick' && (
                    <div className="space-y-3 py-2">
                        <p className="text-xs text-muted-foreground">
                            Choose the colour mode for this document. All object colours will be
                            converted to the new working space.
                        </p>
                        <div className="space-y-2">
                            {modeOpts.map((opt) => (
                                <label
                                    key={opt.id}
                                    className="flex items-start gap-3 p-3 rounded border border-border cursor-pointer hover:border-primary transition-colors"
                                >
                                    <input
                                        type="radio"
                                        name="colour-mode"
                                        value={opt.id}
                                        checked={mode === opt.id}
                                        onChange={() => setMode(opt.id)}
                                        className="mt-0.5"
                                    />
                                    <div>
                                        <p className="text-sm font-medium">{opt.label}</p>
                                        <p className="text-xs text-muted-foreground">{opt.desc}</p>
                                    </div>
                                </label>
                            ))}
                        </div>
                        {profiles.length > 0 && (
                            <p className="text-[10px] text-muted-foreground">
                                {profiles.length} built-in ICC profile(s) available.
                            </p>
                        )}
                    </div>
                )}

                {step === 'preview' && (
                    <div className="py-2 space-y-3">
                        <p className="text-xs text-muted-foreground">
                            Preview of colour changes. ΔE shows the perceptual difference after conversion.
                        </p>
                        <ColourModePreview pairs={pairs} />
                    </div>
                )}

                {step === 'confirm' && (
                    <div className="py-2 space-y-3">
                        <p className="text-xs text-green-700 dark:text-green-400 font-medium">
                            Conversion complete.
                        </p>
                        {warnings.length > 0 && (
                            <div className="space-y-1">
                                <p className="text-xs text-muted-foreground font-medium">Warnings:</p>
                                <ul className="max-h-40 overflow-y-auto space-y-0.5">
                                    {warnings.map((w, i) => (
                                        <li key={i} className="text-[10px] text-yellow-700 dark:text-yellow-400">
                                            {w}
                                        </li>
                                    ))}
                                </ul>
                            </div>
                        )}
                    </div>
                )}

                <DialogFooter className="flex gap-2 justify-end">
                    {step === 'pick' && (
                        <>
                            <Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
                            <Button onClick={handlePreview} disabled={loading || !doc}>
                                {loading ? 'Loading…' : 'Preview →'}
                            </Button>
                        </>
                    )}
                    {step === 'preview' && (
                        <>
                            <Button variant="outline" onClick={() => setStep('pick')}>Back</Button>
                            <Button onClick={handleConvert} disabled={loading}>
                                {loading ? 'Converting…' : 'Convert'}
                            </Button>
                        </>
                    )}
                    {step === 'confirm' && (
                        <Button onClick={() => onOpenChange(false)}>Done</Button>
                    )}
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
