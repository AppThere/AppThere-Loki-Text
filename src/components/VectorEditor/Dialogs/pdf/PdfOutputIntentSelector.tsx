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

// ProfileInfo verified from types.ts: { id: string; name: string; description: string }
// PdfExportSettings verified from pdfTypes.ts: output_condition_identifier, output_condition, registry_name

import type { ProfileInfo, ColourSpace } from '@/lib/vector/types';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';

/** Maps BuiltInProfile id → static metadata for PDF/X OutputIntent. */
const PROFILE_INFO: Record<
    string,
    { identifier: string; registry: string; isCmyk: boolean }
> = {
    IsoCoatedV2: {
        identifier: 'ISO Coated v2',
        registry: 'http://www.color.org',
        isCmyk: true,
    },
    SwopV2: {
        identifier: 'SWOP v2',
        registry: 'http://www.color.org',
        isCmyk: true,
    },
    GraCol2006: {
        identifier: 'GRACoL 2006 Coated1v2',
        registry: 'http://www.color.org',
        isCmyk: true,
    },
    SrgbIec61966: {
        identifier: 'sRGB IEC61966-2.1',
        registry: 'http://www.color.org',
        isCmyk: false,
    },
};

export interface OutputIntentSelection {
    profileId: string;
    output_condition_identifier: string;
    output_condition: string;
    registry_name: string;
}

interface PdfOutputIntentSelectorProps {
    selectedProfileId: string;
    profiles: ProfileInfo[];
    docColourSpace: ColourSpace;
    onChange: (selection: OutputIntentSelection) => void;
}

export function PdfOutputIntentSelector({
    selectedProfileId,
    profiles,
    docColourSpace,
    onChange,
}: PdfOutputIntentSelectorProps) {
    const selectedProfile = profiles.find((p) => p.id === selectedProfileId);
    const meta = PROFILE_INFO[selectedProfileId];

    const handleSelect = (id: string) => {
        const profile = profiles.find((p) => p.id === id);
        const m = PROFILE_INFO[id];
        onChange({
            profileId: id,
            output_condition_identifier: m?.identifier ?? id,
            output_condition: profile?.description ?? '',
            registry_name: m?.registry ?? 'http://www.color.org',
        });
    };

    const docIsCmyk = docColourSpace.type === 'Cmyk';
    const profileIsCmyk = meta?.isCmyk ?? false;
    const showMismatch = profileIsCmyk !== docIsCmyk;

    return (
        <div className="space-y-2">
            <Select value={selectedProfileId} onValueChange={handleSelect}>
                <SelectTrigger>
                    <SelectValue placeholder="Select output profile…" />
                </SelectTrigger>
                <SelectContent>
                    {profiles.map((p) => {
                        const isCmyk = PROFILE_INFO[p.id]?.isCmyk ?? false;
                        return (
                            <SelectItem key={p.id} value={p.id}>
                                <span className="flex items-center gap-2">
                                    <span
                                        className={
                                            isCmyk
                                                ? 'text-[10px] px-1 rounded bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200'
                                                : 'text-[10px] px-1 rounded bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-200'
                                        }
                                    >
                                        {isCmyk ? 'CMYK' : 'RGB'}
                                    </span>
                                    {p.name}
                                </span>
                            </SelectItem>
                        );
                    })}
                </SelectContent>
            </Select>

            {selectedProfile && (
                <p className="text-xs text-muted-foreground">{selectedProfile.description}</p>
            )}

            {showMismatch && (
                <p className="text-xs text-muted-foreground flex items-start gap-1">
                    <span>ℹ</span>
                    <span>
                        Your document uses {docIsCmyk ? 'CMYK' : 'RGB'}. Colours will be converted to{' '}
                        {profileIsCmyk ? 'CMYK' : 'RGB'} during export using the selected output intent
                        profile.
                    </span>
                </p>
            )}
        </div>
    );
}
