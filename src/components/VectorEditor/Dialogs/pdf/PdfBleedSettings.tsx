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

import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

// Preset bleed sizes: mm → points (1 mm = 2.8346 pt)
const MM_TO_PT = 2.8346;

const PRESETS = [
    { label: 'None', pt: 0 },
    { label: '3 mm', pt: Math.round(3 * MM_TO_PT * 1000) / 1000 },
    { label: '5 mm', pt: Math.round(5 * MM_TO_PT * 1000) / 1000 },
];

export interface BleedValue {
    enabled: boolean;
    bleed_pt: number;
}

interface PdfBleedSettingsProps {
    value: BleedValue;
    onChange: (value: BleedValue) => void;
}

export function PdfBleedSettings({ value, onChange }: PdfBleedSettingsProps) {
    const handleToggle = (enabled: boolean) => {
        onChange({ ...value, enabled });
    };

    const handlePtChange = (raw: string) => {
        const pt = parseFloat(raw);
        if (!isNaN(pt) && pt >= 0) {
            onChange({ ...value, bleed_pt: pt });
        }
    };

    const handlePreset = (pt: number) => {
        onChange({ enabled: pt > 0, bleed_pt: pt });
    };

    return (
        <div className="space-y-3">
            <div className="flex items-center gap-3">
                <Switch
                    id="bleed-toggle"
                    checked={value.enabled}
                    onCheckedChange={handleToggle}
                />
                <Label htmlFor="bleed-toggle" className="text-sm font-medium cursor-pointer">
                    Add bleed
                </Label>
            </div>

            <div className={cn('space-y-2', !value.enabled && 'opacity-50 pointer-events-none')}>
                <div className="flex items-center gap-2">
                    <Input
                        type="number"
                        min={0}
                        step={0.001}
                        value={value.bleed_pt}
                        onChange={(e) => handlePtChange(e.target.value)}
                        disabled={!value.enabled}
                        className="w-28 h-8 text-sm"
                        aria-label="Bleed in points"
                    />
                    <span className="text-xs text-muted-foreground">pt</span>
                    <span className="text-xs text-muted-foreground">
                        ({(value.bleed_pt / MM_TO_PT).toFixed(2)} mm)
                    </span>
                </div>

                <div className="flex gap-2">
                    {PRESETS.map((p) => (
                        <Button
                            key={p.label}
                            type="button"
                            variant="outline"
                            size="sm"
                            disabled={!value.enabled && p.pt > 0}
                            onClick={() => handlePreset(p.pt)}
                            className={cn(
                                'h-7 text-xs',
                                value.bleed_pt === p.pt && value.enabled === (p.pt > 0)
                                    ? 'border-primary'
                                    : '',
                            )}
                        >
                            {p.label}
                        </Button>
                    ))}
                </div>

                <p className="text-xs text-muted-foreground">
                    Bleed is applied equally on all four sides.
                </p>
            </div>
        </div>
    );
}
