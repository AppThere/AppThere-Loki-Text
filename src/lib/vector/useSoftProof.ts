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
// DocumentColourSettings verified from types.ts: { working_space: ColourSpace; rendering_intent; blackpoint_compensation }

import { useState, useEffect, useRef } from 'react';
import { useVectorStore } from './store';
import { batchConvertColours } from './commands';
import { collectAllUniqueColours, colourCacheKey, displayRgbToCss } from './colourUtils';
import type { VectorObject } from './types';

/**
 * Collects all VectorObjects from the document layers (including group children).
 */
function collectAllObjects(doc: { layers: Array<{ objects: VectorObject[] }> }): VectorObject[] {
    const result: VectorObject[] = [];
    function visit(obj: VectorObject) {
        result.push(obj);
        if (obj.type === 'Group') obj.children.forEach(visit);
    }
    for (const layer of doc.layers) {
        layer.objects.forEach(visit);
    }
    return result;
}

/**
 * Hook that returns a Map of colour-cache-key → CSS rgba string for
 * soft-proof rendering, or null when soft-proof is inactive.
 *
 * When `softProofActive` is true and `softProofProfile` is set, this hook
 * batch-converts all unique document colours to the soft-proof colour space
 * and exposes the results as a lookup map for the canvas renderers.
 */
export function useSoftProof(): Map<string, string> | null {
    const { document: doc, softProofActive, softProofProfile } = useVectorStore();
    const [overrides, setOverrides] = useState<Map<string, string> | null>(null);
    // Track in-flight request to avoid stale updates.
    const reqIdRef = useRef(0);

    useEffect(() => {
        if (!softProofActive || !softProofProfile || !doc) {
            setOverrides(null);
            return;
        }

        const reqId = ++reqIdRef.current;

        const objects = collectAllObjects(doc);
        const colours = collectAllUniqueColours(objects);
        if (colours.length === 0) {
            setOverrides(new Map());
            return;
        }

        // Build target settings using the soft-proof profile as the working space.
        // BuiltIn profiles are always CMYK ICC profiles in this context.
        const targetSettings =
            softProofProfile.type === 'BuiltIn'
                ? {
                      working_space: {
                          type: 'Cmyk' as const,
                          profile: softProofProfile,
                      },
                      rendering_intent: 'RelativeColorimetric' as const,
                      blackpoint_compensation: true,
                  }
                : {
                      // FilePath — treat as a custom CMYK profile.
                      working_space: {
                          type: 'Custom' as const,
                          profile: softProofProfile,
                      },
                      rendering_intent: 'RelativeColorimetric' as const,
                      blackpoint_compensation: true,
                  };

        batchConvertColours(colours, targetSettings)
            .then((rgbaList) => {
                if (reqIdRef.current !== reqId) return; // stale
                const map = new Map<string, string>();
                colours.forEach((colour, i) => {
                    const rgba = rgbaList[i] ?? [0, 0, 0, 1];
                    map.set(colourCacheKey(colour), displayRgbToCss(rgba as [number, number, number, number]));
                });
                setOverrides(map);
            })
            .catch((err) => {
                if (reqIdRef.current !== reqId) return;
                console.error('[useSoftProof] batch convert failed', err);
                setOverrides(null);
            });
    }, [doc, softProofActive, softProofProfile]);

    return overrides;
}
