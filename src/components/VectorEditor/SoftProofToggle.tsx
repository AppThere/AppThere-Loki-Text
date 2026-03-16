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

// IccProfileRef verified from types.ts: { type: 'BuiltIn'; profile: BuiltInProfile } | { type: 'FilePath'; path: string }
// BuiltInProfile values verified from types.ts

import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { useVectorStore } from '@/lib/vector/store';
import type { BuiltInProfile } from '@/lib/vector/types';
import { cn } from '@/lib/utils';

// Built-in CMYK profiles available for soft proofing.
const SOFT_PROOF_PROFILES: { id: BuiltInProfile; label: string }[] = [
    { id: 'IsoCoatedV2', label: 'ISO Coated v2' },
    { id: 'SwopV2', label: 'SWOP v2' },
    { id: 'GraCol2006', label: 'GRACoL 2006' },
];

interface SoftProofToggleProps {
    className?: string;
}

export function SoftProofToggle({ className }: SoftProofToggleProps) {
    const { softProofActive, softProofProfile, setSoftProof } = useVectorStore();

    const activeProfileId: BuiltInProfile =
        softProofProfile?.type === 'BuiltIn'
            ? softProofProfile.profile
            : 'IsoCoatedV2';

    const handleToggle = (active: boolean) => {
        setSoftProof(active, {
            type: 'BuiltIn',
            profile: activeProfileId,
        });
    };

    const handleProfileChange = (id: string) => {
        setSoftProof(softProofActive, {
            type: 'BuiltIn',
            profile: id as BuiltInProfile,
        });
    };

    return (
        <div className={cn('flex items-center gap-2', className)}>
            <Switch
                id="soft-proof-toggle"
                checked={softProofActive}
                onCheckedChange={handleToggle}
                aria-label="Toggle soft proof"
            />
            <Label
                htmlFor="soft-proof-toggle"
                className="text-xs font-medium cursor-pointer whitespace-nowrap"
            >
                Soft proof
            </Label>

            <Select
                value={activeProfileId}
                onValueChange={handleProfileChange}
                disabled={!softProofActive}
            >
                <SelectTrigger className="h-7 text-xs w-36">
                    <SelectValue />
                </SelectTrigger>
                <SelectContent>
                    {SOFT_PROOF_PROFILES.map((p) => (
                        <SelectItem key={p.id} value={p.id} className="text-xs">
                            {p.label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>
        </div>
    );
}
