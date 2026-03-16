import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { DocumentColourSettings, VectorObject } from './types';
import { colourCacheKey, collectNonRgbColours } from './colourUtils';

/**
 * Manages the display colour cache for a vector document.
 *
 * For sRGB documents (the common case), the cache is never populated —
 * colourToKonva handles Rgb variants directly without IPC.
 *
 * For CMYK and other non-sRGB documents, this hook calls the Rust backend
 * once per unique non-RGB colour to obtain its display sRGB value.
 */
export function useDisplayColours(
    objects: VectorObject[],
    colourSettings: DocumentColourSettings,
): Map<string, string> {
    const [cache, setCache] = useState<Map<string, string>>(new Map());
    const pendingRef = useRef<Set<string>>(new Set());

    useEffect(() => {
        const nonRgb = collectNonRgbColours(objects);
        if (nonRgb.length === 0) return;

        // Filter to colours not already cached or pending
        const toFetch = nonRgb.filter((c) => {
            const key = colourCacheKey(c);
            return !cache.has(key) && !pendingRef.current.has(key);
        });
        if (toFetch.length === 0) return;

        // Mark as pending to prevent duplicate requests
        toFetch.forEach((c) => pendingRef.current.add(colourCacheKey(c)));

        invoke<number[][]>('batch_convert_colours', {
            colours: toFetch,
            settings: colourSettings,
        })
            .then((results) => {
                setCache((prev) => {
                    const next = new Map(prev);
                    toFetch.forEach((colour, i) => {
                        const [r, g, b, a] = results[i];
                        const css = `rgba(${Math.round(r * 255)},${Math.round(g * 255)},${Math.round(b * 255)},${a})`;
                        next.set(colourCacheKey(colour), css);
                        pendingRef.current.delete(colourCacheKey(colour));
                    });
                    return next;
                });
            })
            .catch((err: unknown) => {
                console.error(
                    '[useDisplayColours] batch_convert_colours failed:',
                    err,
                );
                toFetch.forEach((c) =>
                    pendingRef.current.delete(colourCacheKey(c)),
                );
            });
    }, [objects, colourSettings]); // eslint-disable-line react-hooks/exhaustive-deps

    return cache;
}
