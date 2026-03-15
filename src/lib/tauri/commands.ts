import { invoke } from '@tauri-apps/api/core';
import type { StyleDefinition, Metadata, LexicalDocumentData } from '../types/odt';

/** Response from `open_document`: native Lexical editor state + styles + metadata. */
export interface LexicalResponse {
    content: LexicalDocumentData;
    styles: Record<string, StyleDefinition>;
    metadata: Metadata;
}

export async function openDocument(
    path: string,
    fileContent?: Uint8Array
): Promise<LexicalResponse> {
    return await invoke('open_document', {
        path,
        fileContent: fileContent ? Array.from(fileContent) : null,
    });
}

export async function saveDocument(
    path: string,
    lexicalJson: string,
    styles: Record<string, StyleDefinition>,
    metadata: Metadata,
    originalPath?: string,
    originalContent?: Uint8Array
): Promise<Uint8Array | null> {
    const result: number[] | null = await invoke('save_document', {
        path,
        lexicalJson,
        styles,
        metadata,
        originalPath: originalPath ?? null,
        originalContent: originalContent ? Array.from(originalContent) : null,
    });
    return result ? new Uint8Array(result) : null;
}

/**
 * Serialise a Lexical document to ODT bytes without writing to disk.
 * Used by `SessionManager.autoSave` and `saveToOriginal`.
 */
export async function serializeDocument(
    lexicalJson: string,
    styles: Record<string, StyleDefinition>,
    metadata: Metadata,
): Promise<Uint8Array> {
    const result: number[] = await invoke('serialize_document', {
        lexicalJson,
        styles,
        metadata,
    });
    return new Uint8Array(result);
}

export async function saveEpub(
    path: string,
    lexicalJson: string,
    styles: Record<string, StyleDefinition>,
    metadata: Metadata,
    fontPaths: string[]
): Promise<Uint8Array | null> {
    const result: number[] | null = await invoke('save_epub', {
        path,
        lexicalJson,
        styles,
        metadata,
        fontPaths,
    });
    return result ? new Uint8Array(result) : null;
}
