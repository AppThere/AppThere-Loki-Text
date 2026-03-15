// Typed wrappers for Tauri invoke calls for vector editor commands.

import { invoke } from '@tauri-apps/api/core';
import type { VectorDocument } from './types';

export async function openVectorDocument(
    path: string,
    fileContent?: Uint8Array,
): Promise<VectorDocument> {
    try {
        return await invoke<VectorDocument>('open_vector_document', {
            path,
            fileContent: fileContent ? Array.from(fileContent) : null,
        });
    } catch (e) {
        throw new Error(String(e));
    }
}

export async function saveVectorDocument(
    path: string,
    document: VectorDocument,
): Promise<void> {
    try {
        await invoke('save_vector_document', { path, document });
    } catch (e) {
        throw new Error(String(e));
    }
}

export async function newVectorDocument(
    preset: 'a4-portrait' | 'a4-landscape' | 'letter-portrait' | 'custom',
    widthPx?: number,
    heightPx?: number,
): Promise<VectorDocument> {
    try {
        return await invoke<VectorDocument>('new_vector_document', {
            preset,
            widthPx: widthPx ?? null,
            heightPx: heightPx ?? null,
        });
    } catch (e) {
        throw new Error(String(e));
    }
}

export async function serializeVectorDocument(
    document: VectorDocument,
): Promise<Uint8Array> {
    try {
        const bytes = await invoke<number[]>('serialize_vector_document', { document });
        return new Uint8Array(bytes);
    } catch (e) {
        throw new Error(String(e));
    }
}

export async function deserializeVectorDocument(
    fileContent: Uint8Array,
): Promise<VectorDocument> {
    try {
        return await invoke<VectorDocument>('deserialize_vector_document', {
            fileContent: Array.from(fileContent),
        });
    } catch (e) {
        throw new Error(String(e));
    }
}
