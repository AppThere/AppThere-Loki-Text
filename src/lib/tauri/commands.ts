import { invoke } from '@tauri-apps/api/core';
import type { StyleDefinition, Metadata, LexicalDocumentData } from '../types/odt';

/**
 * Android only: persist a content:// URI permission across app restarts.
 *
 * Routes through a regular Rust command (`take_persistable_uri_permission`)
 * which calls `PluginHandle::run_mobile_plugin_async` via JNI — bypassing the
 * Tauri ACL system that blocks direct `plugin:name|command` IPC from JS.
 * No-ops on desktop.
 */
export async function takePersistableUriPermission(uri: string): Promise<void> {
    await invoke('take_persistable_uri_permission', { uri });
}

/**
 * Android only: open a file using ACTION_OPEN_DOCUMENT, which grants a
 * persistable content:// URI permission so the file can be reopened from
 * the Recents list after the app process is killed.
 *
 * Routes through a regular Rust command (`pick_file_to_open`) which calls
 * `PluginHandle::run_mobile_plugin_async` via JNI — bypassing the Tauri ACL
 * system that blocks direct `plugin:name|command` IPC from JS.
 * Returns the selected content:// URI string, or rejects with "cancelled".
 */
export async function openFilePicker(): Promise<string> {
    return invoke<string>('pick_file_to_open');
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
