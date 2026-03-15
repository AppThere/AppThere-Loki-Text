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

import { cn } from '@/lib/utils';
import type { PdfXStandard } from '@/lib/vector/pdfTypes';

// PdfXStandard values verified from pdfTypes.ts: 'X4_2008' and 'X1a2001'

interface PdfConformanceSelectorProps {
    value: PdfXStandard;
    onChange: (value: PdfXStandard) => void;
    disabled?: boolean;
}

const OPTIONS: {
    value: PdfXStandard;
    title: string;
    description: string;
}[] = [
    {
        value: 'X4_2008',
        title: 'PDF/X-4:2008',
        description:
            'Modern standard. Allows transparency and RGB with ICC profiles. Recommended for most print workflows.',
    },
    {
        value: 'X1a2001',
        title: 'PDF/X-1a:2001',
        description:
            'Legacy standard. Requires all-CMYK, no transparency. Use only when specified by your printer.',
    },
];

export function PdfConformanceSelector({
    value,
    onChange,
    disabled = false,
}: PdfConformanceSelectorProps) {
    return (
        <div className={cn('space-y-2', disabled && 'opacity-50 pointer-events-none')}>
            {OPTIONS.map((opt) => {
                const selected = value === opt.value;
                return (
                    <button
                        key={opt.value}
                        type="button"
                        onClick={() => onChange(opt.value)}
                        disabled={disabled}
                        className={cn(
                            'w-full text-left p-3 rounded-lg border transition-colors min-h-[44px]',
                            selected
                                ? 'border-primary bg-accent text-accent-foreground'
                                : 'border-border hover:border-muted-foreground',
                        )}
                        aria-pressed={selected}
                    >
                        <div className="flex items-center gap-2">
                            <span
                                className={cn(
                                    'inline-block h-3 w-3 rounded-full border-2 shrink-0',
                                    selected ? 'border-primary bg-primary' : 'border-muted-foreground',
                                )}
                            />
                            <span className="text-sm font-medium">{opt.title}</span>
                        </div>
                        <p className="mt-1 ml-5 text-xs text-muted-foreground leading-snug">
                            {opt.description}
                        </p>
                    </button>
                );
            })}
        </div>
    );
}
