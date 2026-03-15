import type { Colour } from '@/lib/vector/types';
import { displayRgbToCss } from '@/lib/vector/colourUtils';
import { ColourPickerSlider } from './ColourPickerSlider';

interface Props {
    colour: Extract<Colour, { type: 'Spot' }>;
    /** Display sRGB for the spot colour (from IPC cache or Lab→display conversion) */
    displayRgba: [number, number, number, number];
    onChange: (colour: Colour) => void;
    /** Called when user clicks "Convert to process colour" */
    onConvertToProcess: () => void;
}

/**
 * Read-only spot colour display with a tint slider and a "Convert to process"
 * button. The name is not editable (spot colours are defined externally).
 */
export function ColourPickerSpot({ colour, displayRgba, onChange, onConvertToProcess }: Props) {
    const swatchCss = displayRgbToCss(displayRgba);

    const handleTintChange = (tint: number) => {
        onChange({ ...colour, tint });
    };

    const tintGradient = `linear-gradient(to right, white, ${swatchCss})`;
    const tintDisplay = String(Math.round(colour.tint * 100));

    const handleTintText = (raw: string) => {
        const num = parseFloat(raw);
        if (!isNaN(num)) {
            handleTintChange(Math.max(0, Math.min(100, num)) / 100);
        }
    };

    // Lab reference display
    const [l, a, b] = colour.lab_ref;
    const labStr = `L* ${l.toFixed(1)}  a* ${a.toFixed(1)}  b* ${b.toFixed(1)}`;

    return (
        <div className="space-y-3">
            {/* Swatch + name */}
            <div className="flex items-center gap-3">
                <div
                    className="w-10 h-10 rounded border border-input shrink-0"
                    style={{ background: swatchCss }}
                    aria-label="Spot colour swatch"
                />
                <div className="flex-1 min-w-0">
                    <p className="text-xs font-medium truncate">{colour.name}</p>
                    <p className="text-[10px] text-muted-foreground mt-0.5">{labStr}</p>
                </div>
            </div>

            {/* Tint slider */}
            <ColourPickerSlider
                label="Tint"
                value={colour.tint}
                onChange={handleTintChange}
                gradient={tintGradient}
                displayValue={tintDisplay}
                onTextChange={handleTintText}
            />

            {/* Alpha display (spot colours carry tint not alpha, but show 100% for info) */}
            <p className="text-[10px] text-muted-foreground text-center">
                Spot colour — tint {tintDisplay}%
            </p>

            {/* Convert button */}
            <button
                onClick={onConvertToProcess}
                className="w-full py-2 text-xs rounded border border-input hover:border-primary hover:text-primary transition-colors"
            >
                Convert to process colour…
            </button>
        </div>
    );
}

