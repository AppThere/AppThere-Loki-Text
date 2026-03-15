import { useState } from 'react';
import type { Colour, DocumentColourSettings } from '@/lib/vector/types';
import {
    colourLabel,
    colourSpaceBadge,
    isCmykDocument,
    getDisplayRgba,
    displayRgbToCss,
    parseHexColour,
} from '@/lib/vector/colourUtils';
import {
    parseRgbChannel,
    parseCmykChannel,
    formatRgbChannel,
    formatCmykChannel,
} from '@/lib/vector/parseUnitInput';
import { ColourPickerSlider } from './ColourPickerSlider';
import { ColourPickerSpot } from './ColourPickerSpot';

interface Props {
    colour: Colour;
    onChange: (colour: Colour) => void;
    colourSettings: DocumentColourSettings;
    displayCache: Map<string, string>;
}

function RgbSliders({ colour, onChange }: { colour: Extract<Colour, { type: 'Rgb' }>; onChange: (c: Colour) => void }) {
    const set = (channel: 'r' | 'g' | 'b' | 'a', v: number) =>
        onChange({ ...colour, [channel]: v });

    const mkGrad = (ch: 'r' | 'g' | 'b') => {
        const base = { r: colour.r, g: colour.g, b: colour.b };
        const lo = { ...base, [ch]: 0 };
        const hi = { ...base, [ch]: 1 };
        return `linear-gradient(to right, rgb(${lo.r * 255},${lo.g * 255},${lo.b * 255}), rgb(${hi.r * 255},${hi.g * 255},${hi.b * 255}))`;
    };

    const alphaGrad = `linear-gradient(to right, rgba(${Math.round(colour.r * 255)},${Math.round(colour.g * 255)},${Math.round(colour.b * 255)},0), rgba(${Math.round(colour.r * 255)},${Math.round(colour.g * 255)},${Math.round(colour.b * 255)},1))`;

    const makeTextHandler = (ch: 'r' | 'g' | 'b') => (raw: string) => {
        const v = parseRgbChannel(raw);
        if (v !== null) set(ch, v);
    };

    return (
        <>
            <ColourPickerSlider label="R" value={colour.r} onChange={(v) => set('r', v)} gradient={mkGrad('r')} displayValue={formatRgbChannel(colour.r)} onTextChange={makeTextHandler('r')} />
            <ColourPickerSlider label="G" value={colour.g} onChange={(v) => set('g', v)} gradient={mkGrad('g')} displayValue={formatRgbChannel(colour.g)} onTextChange={makeTextHandler('g')} />
            <ColourPickerSlider label="B" value={colour.b} onChange={(v) => set('b', v)} gradient={mkGrad('b')} displayValue={formatRgbChannel(colour.b)} onTextChange={makeTextHandler('b')} />
            <ColourPickerSlider label="A" value={colour.a} onChange={(v) => set('a', v)} gradient={alphaGrad} displayValue={formatRgbChannel(colour.a)} onTextChange={(raw) => { const v = parseRgbChannel(raw); if (v !== null) set('a', v); }} />
        </>
    );
}

function CmykSliders({ colour, onChange }: { colour: Extract<Colour, { type: 'Cmyk' }>; onChange: (c: Colour) => void }) {
    const set = (ch: 'c' | 'm' | 'y' | 'k' | 'alpha', v: number) =>
        onChange({ ...colour, [ch]: v });

    const mkGrad = (ch: 'c' | 'm' | 'y' | 'k') => {
        const toRgb = (c: number, m: number, y: number, k: number) => {
            const r = (1 - c) * (1 - k);
            const g = (1 - m) * (1 - k);
            const b = (1 - y) * (1 - k);
            return `rgb(${Math.round(r * 255)},${Math.round(g * 255)},${Math.round(b * 255)})`;
        };
        const { c, m, y, k } = colour;
        const overrides0 = { c, m, y, k, [ch]: 0 };
        const overrides1 = { c, m, y, k, [ch]: 1 };
        return `linear-gradient(to right, ${toRgb(overrides0.c, overrides0.m, overrides0.y, overrides0.k)}, ${toRgb(overrides1.c, overrides1.m, overrides1.y, overrides1.k)})`;
    };

    const makeTextHandler = (ch: 'c' | 'm' | 'y' | 'k') => (raw: string) => {
        const v = parseCmykChannel(raw);
        if (v !== null) set(ch, v);
    };

    return (
        <>
            <ColourPickerSlider label="C" value={colour.c} onChange={(v) => set('c', v)} gradient={mkGrad('c')} displayValue={formatCmykChannel(colour.c)} onTextChange={makeTextHandler('c')} />
            <ColourPickerSlider label="M" value={colour.m} onChange={(v) => set('m', v)} gradient={mkGrad('m')} displayValue={formatCmykChannel(colour.m)} onTextChange={makeTextHandler('m')} />
            <ColourPickerSlider label="Y" value={colour.y} onChange={(v) => set('y', v)} gradient={mkGrad('y')} displayValue={formatCmykChannel(colour.y)} onTextChange={makeTextHandler('y')} />
            <ColourPickerSlider label="K" value={colour.k} onChange={(v) => set('k', v)} gradient={mkGrad('k')} displayValue={formatCmykChannel(colour.k)} onTextChange={makeTextHandler('k')} />
        </>
    );
}

export function ColourPicker({ colour, onChange, colourSettings, displayCache }: Props) {
    const [hexDraft, setHexDraft] = useState<string | null>(null);

    const displayRgba = getDisplayRgba(colour, displayCache);
    const swatchCss = displayRgbToCss(displayRgba);
    const badge = colourSpaceBadge(colour);
    const isCmyk = isCmykDocument(colourSettings);

    const handleConvertSpotToProcess = () => {
        if (colour.type !== 'Spot') return;
        const fallback = colour.cmyk_fallback;
        onChange(fallback);
    };

    const handleHexCommit = (raw: string) => {
        setHexDraft(null);
        const parsed = parseHexColour(raw);
        if (parsed) onChange(parsed);
    };

    const hexDisplayValue = hexDraft ?? (() => {
        if (colour.type === 'Rgb') {
            const r = Math.round(colour.r * 255).toString(16).padStart(2, '0');
            const g = Math.round(colour.g * 255).toString(16).padStart(2, '0');
            const b = Math.round(colour.b * 255).toString(16).padStart(2, '0');
            return `#${r}${g}${b}`;
        }
        return '#------';
    })();

    return (
        <div className="space-y-3 p-2" data-testid="colour-picker">
            {/* Preview swatch + badge */}
            <div className="flex items-center gap-2">
                <div
                    className="w-10 h-10 rounded border border-input shrink-0"
                    style={{ background: swatchCss }}
                    aria-label="Colour preview"
                />
                <div className="flex-1">
                    <p className="text-xs font-medium">{colourLabel(colour)}</p>
                    <span className="text-[10px] bg-muted text-muted-foreground px-1.5 py-0.5 rounded">
                        {badge}
                    </span>
                </div>
            </div>

            {/* Channel sliders */}
            {colour.type === 'Rgb' && (
                <RgbSliders colour={colour} onChange={onChange} />
            )}
            {colour.type === 'Cmyk' && (
                <CmykSliders colour={colour} onChange={onChange} />
            )}
            {colour.type === 'Spot' && (
                <ColourPickerSpot
                    colour={colour}
                    displayRgba={displayRgba}
                    onChange={onChange}
                    onConvertToProcess={handleConvertSpotToProcess}
                />
            )}
            {colour.type === 'Linked' && (
                <p className="text-xs text-muted-foreground text-center py-2">
                    Linked swatch — edit via the Swatches panel.
                </p>
            )}
            {colour.type === 'Lab' && (
                <p className="text-xs text-muted-foreground text-center py-2">
                    L* {colour.l.toFixed(1)} a* {colour.a.toFixed(1)} b* {colour.b.toFixed(1)}
                </p>
            )}

            {/* Hex input — only for RGB */}
            {(colour.type === 'Rgb') && (
                <div className="flex items-center gap-2 pt-1 border-t border-border">
                    <span className="text-[10px] text-muted-foreground w-8 text-right shrink-0">Hex</span>
                    <input
                        type="text"
                        value={hexDisplayValue}
                        onChange={(e) => setHexDraft(e.target.value)}
                        onBlur={(e) => handleHexCommit(e.target.value)}
                        onKeyDown={(e) => { if (e.key === 'Enter') handleHexCommit((e.target as HTMLInputElement).value); }}
                        className="flex-1 h-6 text-xs rounded border border-input bg-background px-2"
                        aria-label="Hex colour value"
                        spellCheck={false}
                    />
                </div>
            )}

            {/* Mode switch hint for CMYK docs with RGB colour */}
            {isCmyk && colour.type === 'Rgb' && (
                <p className="text-[10px] text-muted-foreground text-center">
                    Document is CMYK — consider using CMYK values.
                </p>
            )}
        </div>
    );
}
