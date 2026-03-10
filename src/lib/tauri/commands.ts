import { invoke } from '@tauri-apps/api/core';
import type { StyleDefinition, Metadata } from '../types/odt';
import type { TiptapResponse } from '../utils/lexicalAdapter';

export async function openDocument(
    path: string,
    fileContent?: Uint8Array
): Promise<TiptapResponse> {
    return await invoke('open_document', {
        path,
        fileContent: fileContent ? Array.from(fileContent) : null,
    });
}

export async function saveDocument(
    path: string,
    tiptapJson: string,
    styles: Record<string, StyleDefinition>,
    metadata: Metadata,
    originalPath?: string,
    originalContent?: Uint8Array
): Promise<Uint8Array | null> {
    const result: number[] | null = await invoke('save_document', {
        path,
        tiptapJson,
        styles,
        metadata,
        originalPath: originalPath ?? null,
        originalContent: originalContent ? Array.from(originalContent) : null,
    });
    return result ? new Uint8Array(result) : null;
}

export async function saveEpub(
    path: string,
    tiptapJson: string,
    styles: Record<string, StyleDefinition>,
    metadata: Metadata,
    fontPaths: string[]
): Promise<Uint8Array | null> {
    const result: number[] | null = await invoke('save_epub', {
        path,
        tiptapJson,
        styles,
        metadata,
        fontPaths,
    });
    return result ? new Uint8Array(result) : null;
}
