// Pure helper functions for the vector editor Zustand store.

import type { Layer, VectorObject } from './types';

/** Add an object to the active layer. */
export function addObjectToLayer(
    layers: Layer[],
    activeLayerIndex: number,
    obj: VectorObject,
): Layer[] {
    return layers.map((layer, i) =>
        i === activeLayerIndex
            ? { ...layer, objects: [...layer.objects, obj] }
            : layer,
    );
}

/** Update an object by id across all layers (shallow merge). */
export function updateObjectInLayers(
    layers: Layer[],
    id: string,
    patch: Partial<VectorObject>,
): Layer[] {
    return layers.map((layer) => ({
        ...layer,
        objects: layer.objects.map((obj) =>
            obj.id === id ? ({ ...obj, ...patch } as VectorObject) : obj,
        ),
    }));
}

/** Remove objects whose ids are in the given set, across all layers. */
export function deleteObjectsFromLayers(
    layers: Layer[],
    ids: Set<string>,
): Layer[] {
    return layers.map((layer) => ({
        ...layer,
        objects: layer.objects.filter((obj) => !ids.has(obj.id)),
    }));
}

/** Generate a simple UUID v4 string. */
export function generateId(): string {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
        const r = (Math.random() * 16) | 0;
        const v = c === 'x' ? r : (r & 0x3) | 0x8;
        return v.toString(16);
    });
}
