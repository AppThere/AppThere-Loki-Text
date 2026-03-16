// Typed wrappers for Tauri invoke calls for vector editor commands.

import { invoke } from '@tauri-apps/api/core';
import type {
    Colour,
    ColourPreviewPair,
    ConvertColourModeResponse,
    DocumentColourSettings,
    ProfileInfo,
    VectorDocument,
} from './types';
import { colourCacheKey } from './colourUtils';
import type { ConformanceViolation, PdfExportSettings } from './pdfTypes';

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
    preset: string,
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

export async function batchConvertColours(
    colours: Colour[],
    settings: DocumentColourSettings,
): Promise<Array<[number, number, number, number]>> {
    const result = await invoke<number[][]>('batch_convert_colours', {
        colours,
        settings,
    });
    return result as Array<[number, number, number, number]>;
}

/**
 * Convert all colours in a document to a new colour space.
 * Returns the converted document and any gamut-clipping warnings.
 *
 * Rust command: convert_document_colour_mode(document, target_settings)
 * Returns a tuple that serialises as [VectorDocument, ConversionWarning[]].
 */
export async function convertDocumentColourMode(
    document: VectorDocument,
    targetSettings: DocumentColourSettings,
): Promise<ConvertColourModeResponse> {
    try {
        const result = await invoke<[VectorDocument, Array<{ object_id: string; property: string; message: string }>]>(
            'convert_document_colour_mode',
            { document, targetSettings },
        );
        return { document: result[0], warnings: result[1] };
    } catch (e) {
        throw new Error(String(e));
    }
}

/**
 * Returns metadata for all built-in ICC output intent profiles.
 */
export async function getOutputIntentProfiles(): Promise<ProfileInfo[]> {
    try {
        return await invoke<ProfileInfo[]>('get_output_intent_profiles');
    } catch (e) {
        throw new Error(String(e));
    }
}

/**
 * Preview how colours will look after conversion to a different colour space.
 *
 * Collects unique colours from the document, converts them with both the
 * source and target settings, then returns before/after pairs with ΔE values.
 */
export async function previewColourConversion(
    document: VectorDocument,
    targetSettings: DocumentColourSettings,
    maxColours = 64,
): Promise<ColourPreviewPair[]> {
    const seen = new Set<string>();
    const colours: Colour[] = [];
    for (const layer of document.layers) {
        for (const obj of layer.objects) {
            if (obj.style.fill.type === 'Solid') {
                const key = colourCacheKey(obj.style.fill.colour);
                if (!seen.has(key) && colours.length < maxColours) {
                    seen.add(key);
                    colours.push(obj.style.fill.colour);
                }
            }
            if (obj.style.stroke.paint.type === 'Solid') {
                const key = colourCacheKey(obj.style.stroke.paint.colour);
                if (!seen.has(key) && colours.length < maxColours) {
                    seen.add(key);
                    colours.push(obj.style.stroke.paint.colour);
                }
            }
        }
    }
    if (colours.length === 0) return [];

    const [originalDisplays, convertedRaw] = await Promise.all([
        batchConvertColours(colours, document.colour_settings),
        invoke<number[][]>('preview_colour_conversion', {
            document,
            targetSettings,
            maxColours,
        }),
    ]);
    const convertedDisplays = convertedRaw as Array<[number, number, number, number]>;

    return colours.map((colour, i) => {
        const orig = (originalDisplays[i] ?? [0, 0, 0, 1]) as [number, number, number, number];
        const conv = (convertedDisplays[i] ?? [0, 0, 0, 1]) as [number, number, number, number];
        const dr = (orig[0] - conv[0]) * 255;
        const dg = (orig[1] - conv[1]) * 255;
        const db = (orig[2] - conv[2]) * 255;
        const delta_e = Math.sqrt(dr * dr + dg * dg + db * db) / Math.sqrt(3);
        return { original: colour, original_display: orig, converted_display: conv, delta_e };
    });
}

/**
 * Search the Pantone colour library by name.
 * Returns up to 50 matching entries with Lab reference values.
 */
export async function searchPantone(
    query: string,
): Promise<Array<{ name: string; lab_ref: [number, number, number] }>> {
    try {
        return await invoke<Array<{ name: string; lab_ref: [number, number, number] }>>(
            'search_pantone',
            { query },
        );
    } catch (e) {
        throw new Error(String(e));
    }
}

/**
 * Validate a vector document against a PDF/X standard.
 *
 * Returns a list of conformance violations. An empty list means the document
 * is conformant and ready for export.
 */
export async function validatePdfXConformance(
    document: VectorDocument,
    settings: PdfExportSettings,
): Promise<ConformanceViolation[]> {
    try {
        return await invoke<ConformanceViolation[]>('validate_pdf_x_conformance', {
            document,
            settings,
        });
    } catch (e) {
        throw new Error(String(e));
    }
}

/**
 * Export a vector document to a PDF/X file at the given path.
 *
 * Throws if the document is not conformant or if the file cannot be written.
 */
export async function exportPdfX(
    document: VectorDocument,
    settings: PdfExportSettings,
    path: string,
): Promise<void> {
    try {
        await invoke<void>('export_pdf_x', { document, settings, path });
    } catch (e) {
        throw new Error(String(e));
    }
}
