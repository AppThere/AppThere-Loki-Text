import { invoke } from '@tauri-apps/api/core';
import type { StyleDefinition, Metadata, LexicalDocumentData } from '../types/odt';

/**
 * Android only: persist a content:// URI permission across app restarts.
 *
 * After the user picks a file through the SAF file picker, Android grants a
 * temporary permission for that URI. Calling this command calls
 * `ContentResolver.takePersistableUriPermission()` so the app can still read
 * (and write) the file in future sessions — fixing the Recents open-after-
 * restart permission error.
 *
 * On non-Android platforms this will reject; callers must swallow the error.
 */
export async function takePersistableUriPermission(uri: string): Promise<void> {
    await invoke('plugin:uriPermission|takePersistablePermission', { uri });
}

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

/** A PDF/X conformance violation returned by `validateTextPdfXConformance`. */
export interface PdfConformanceViolation {
    rule: string;
    message: string;
    autoFixable: boolean;
}

/** PDF/X export settings — matches `loki_pdf::export_settings::PdfExportSettings`. */
export interface PdfExportSettings {
    standard: 'X1a2001' | 'X4_2008';
    bleedPt: number;
    outputConditionIdentifier: string;
    outputCondition: string;
    registryName: string;
    resolutionDpi: number;
}

export const DEFAULT_PDF_SETTINGS: PdfExportSettings = {
    standard: 'X4_2008',
    bleedPt: 0,
    outputConditionIdentifier: 'sRGB',
    outputCondition: 'sRGB IEC61966-2.1',
    registryName: 'http://www.color.org',
    resolutionDpi: 300,
};

/**
 * Validate a text document against PDF/X conformance rules.
 * Returns an empty array if the document passes all checks.
 */
export async function validateTextPdfXConformance(
    lexicalJson: string,
    styles: Record<string, StyleDefinition>,
    metadata: Metadata,
    settings: PdfExportSettings,
): Promise<PdfConformanceViolation[]> {
    return await invoke('validate_text_pdf_x_conformance', {
        lexicalJson,
        styles,
        metadata,
        settings,
    });
}

/**
 * Export a text document to a PDF/X file at the given path.
 * Throws a string error message on failure.
 */
export async function exportTextPdfX(
    lexicalJson: string,
    styles: Record<string, StyleDefinition>,
    metadata: Metadata,
    settings: PdfExportSettings,
    path: string,
): Promise<void> {
    await invoke('export_text_pdf_x', {
        lexicalJson,
        styles,
        metadata,
        settings,
        path,
    });
}
