// Copyright 2024 AppThere
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// ColourPreviewPair fields verified from types.ts:
//   { original: Colour; original_display: [r,g,b,a]; converted_display: [r,g,b,a]; delta_e: number }
// deltaELabel / deltaEColourClass verified from pdfTypes.ts

import type { ColourPreviewPair } from '@/lib/vector/types';
import { deltaELabel, deltaEColourClass } from '@/lib/vector/pdfTypes';
import { displayRgbToCss, colourLabel } from '@/lib/vector/colourUtils';
import { cn } from '@/lib/utils';

interface SwatchProps {
    rgba: [number, number, number, number];
    label: string;
}

function Swatch({ rgba, label }: SwatchProps) {
    return (
        <div className="flex flex-col items-center gap-1">
            <div
                className="h-8 w-8 rounded border border-border shadow-sm"
                style={{ backgroundColor: displayRgbToCss(rgba) }}
                title={label}
            />
            <span className="text-[10px] text-muted-foreground text-center leading-tight max-w-[4rem] truncate">
                {label}
            </span>
        </div>
    );
}

interface PairRowProps {
    pair: ColourPreviewPair;
}

function PairRow({ pair }: PairRowProps) {
    const deClass = deltaEColourClass(pair.delta_e);
    const deLabel = deltaELabel(pair.delta_e);

    return (
        <div className="flex items-center gap-3 p-2 rounded-md border border-border">
            <Swatch rgba={pair.original_display} label={colourLabel(pair.original)} />
            <span className="text-muted-foreground text-sm">→</span>
            <Swatch rgba={pair.converted_display} label="Converted" />
            <div className="ml-auto text-right shrink-0">
                <p className={cn('text-xs font-medium', deClass)}>
                    ΔE {pair.delta_e.toFixed(1)}
                </p>
                <p className={cn('text-[10px]', deClass)}>{deLabel}</p>
            </div>
        </div>
    );
}

interface ColourPreviewGridProps {
    pairs: ColourPreviewPair[];
    loading?: boolean;
}

export function ColourPreviewGrid({ pairs, loading = false }: ColourPreviewGridProps) {
    if (loading) {
        return (
            <div className="space-y-2">
                {[1, 2, 3].map((i) => (
                    <div
                        key={i}
                        className="flex items-center gap-3 p-2 rounded-md border border-border animate-pulse"
                    >
                        <div className="h-8 w-8 rounded bg-muted" />
                        <span className="text-muted-foreground">→</span>
                        <div className="h-8 w-8 rounded bg-muted" />
                        <div className="ml-auto space-y-1">
                            <div className="h-3 w-12 bg-muted rounded" />
                            <div className="h-2 w-10 bg-muted rounded" />
                        </div>
                    </div>
                ))}
            </div>
        );
    }

    if (pairs.length === 0) {
        return (
            <p className="text-sm text-muted-foreground text-center py-4">
                No colour conversion preview available.
            </p>
        );
    }

    return (
        <div className="space-y-2">
            <div className="flex items-center justify-between text-xs text-muted-foreground mb-1">
                <span>Original → Converted</span>
                <span>Colour difference (ΔE)</span>
            </div>
            {pairs.map((pair, i) => (
                <PairRow key={i} pair={pair} />
            ))}
        </div>
    );
}
