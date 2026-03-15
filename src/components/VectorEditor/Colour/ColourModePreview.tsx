import type { ColourPreviewPair } from '@/lib/vector/types';
import { displayRgbToCss, colourLabel } from '@/lib/vector/colourUtils';

interface Props {
    pairs: ColourPreviewPair[];
}

function deltaEColour(deltaE: number): string {
    if (deltaE < 5) return 'text-green-600 dark:text-green-400';
    if (deltaE < 15) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-red-600 dark:text-red-400';
}

/**
 * Renders a grid of before/after colour swatches with ΔE values.
 */
export function ColourModePreview({ pairs }: Props) {
    if (pairs.length === 0) {
        return (
            <p className="text-xs text-muted-foreground text-center py-6">
                No colours found in the document.
            </p>
        );
    }

    return (
        <div className="space-y-2">
            <div className="grid grid-cols-[1fr_auto_1fr_auto] gap-x-2 gap-y-1 text-[10px] text-muted-foreground font-medium pb-1 border-b border-border">
                <span>Before</span>
                <span />
                <span>After</span>
                <span className="text-right">ΔE</span>
            </div>

            <div className="max-h-60 overflow-y-auto space-y-1">
                {pairs.map((pair, i) => {
                    const origCss = displayRgbToCss(pair.original_display);
                    const convCss = displayRgbToCss(pair.converted_display);
                    const deClass = deltaEColour(pair.delta_e);
                    return (
                        <div
                            key={i}
                            className="grid grid-cols-[1fr_auto_1fr_auto] gap-x-2 items-center"
                        >
                            {/* Before */}
                            <div className="flex items-center gap-1.5 min-w-0">
                                <div
                                    className="w-6 h-6 rounded border border-input shrink-0"
                                    style={{ background: origCss }}
                                />
                                <span className="text-[10px] truncate text-muted-foreground">
                                    {colourLabel(pair.original)}
                                </span>
                            </div>

                            <span className="text-muted-foreground">→</span>

                            {/* After */}
                            <div className="flex items-center gap-1.5 min-w-0">
                                <div
                                    className="w-6 h-6 rounded border border-input shrink-0"
                                    style={{ background: convCss }}
                                />
                            </div>

                            {/* ΔE */}
                            <span className={`text-right text-[10px] tabular-nums ${deClass}`}>
                                {pair.delta_e.toFixed(1)}
                            </span>
                        </div>
                    );
                })}
            </div>

            <p className="text-[10px] text-muted-foreground">
                ΔE &lt; 5 = imperceptible · 5–15 = noticeable · &gt;15 = significant
            </p>
        </div>
    );
}
