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

import { useState, useCallback } from 'react';
import { save as dialogSave } from '@tauri-apps/plugin-dialog';
import { validatePdfXConformance, exportPdfX, previewColourConversion } from './commands';
import type { VectorDocument, ColourPreviewPair } from './types';
import type { PdfExportSettings, ConformanceViolation } from './pdfTypes';
import { groupViolationsByLevel } from './pdfTypes';

export type ExportStep =
    | 'settings'
    | 'validate'
    | 'preview'
    | 'confirm'
    | 'exporting'
    | 'done'
    | 'error';

export interface PdfExportState {
    step: ExportStep;
    violations: ConformanceViolation[];
    previewPairs: ColourPreviewPair[];
    previewLoading: boolean;
    exportLoading: boolean;
    exportError: string | null;
    exportedPath: string | null;
}

export interface PdfExportActions {
    runValidation: (doc: VectorDocument, settings: PdfExportSettings) => Promise<void>;
    runPreview: (doc: VectorDocument, settings: PdfExportSettings) => Promise<void>;
    runExport: (doc: VectorDocument, settings: PdfExportSettings) => Promise<void>;
    goToStep: (step: ExportStep) => void;
    reset: () => void;
}

const INITIAL_STATE: PdfExportState = {
    step: 'settings',
    violations: [],
    previewPairs: [],
    previewLoading: false,
    exportLoading: false,
    exportError: null,
    exportedPath: null,
};

/** Build a DocumentColourSettings for the preview from the export settings. */
function buildTargetSettings(settings: PdfExportSettings) {
    // Map the export profile to a DocumentColourSettings for preview conversion.
    // Use the output_condition_identifier to infer the profile.
    const id = settings.output_condition_identifier.toLowerCase();
    if (id.includes('coated') || id.includes('fogra') || id.includes('iso')) {
        return {
            working_space: {
                type: 'Cmyk' as const,
                profile: { type: 'BuiltIn' as const, profile: 'IsoCoatedV2' as const },
            },
            rendering_intent: 'RelativeColorimetric' as const,
            blackpoint_compensation: true,
        };
    }
    if (id.includes('swop')) {
        return {
            working_space: {
                type: 'Cmyk' as const,
                profile: { type: 'BuiltIn' as const, profile: 'SwopV2' as const },
            },
            rendering_intent: 'RelativeColorimetric' as const,
            blackpoint_compensation: true,
        };
    }
    if (id.includes('gracol') || id.includes('gra')) {
        return {
            working_space: {
                type: 'Cmyk' as const,
                profile: { type: 'BuiltIn' as const, profile: 'GraCol2006' as const },
            },
            rendering_intent: 'RelativeColorimetric' as const,
            blackpoint_compensation: true,
        };
    }
    // Default: sRGB
    return {
        working_space: { type: 'Srgb' as const },
        rendering_intent: 'RelativeColorimetric' as const,
        blackpoint_compensation: true,
    };
}

export function usePdfExport(): [PdfExportState, PdfExportActions] {
    const [state, setState] = useState<PdfExportState>(INITIAL_STATE);

    const runValidation = useCallback(async (doc: VectorDocument, settings: PdfExportSettings) => {
        setState((s) => ({ ...s, step: 'validate', violations: [] }));
        try {
            const violations = await validatePdfXConformance(doc, settings);
            const { errors } = groupViolationsByLevel(violations);
            setState((s) => ({
                ...s,
                violations,
                step: errors.length === 0 ? 'preview' : 'validate',
            }));
        } catch (e) {
            setState((s) => ({
                ...s,
                step: 'error',
                exportError: `Validation failed: ${String(e)}`,
            }));
        }
    }, []);

    const runPreview = useCallback(async (doc: VectorDocument, settings: PdfExportSettings) => {
        setState((s) => ({ ...s, previewLoading: true }));
        try {
            const targetSettings = buildTargetSettings(settings);
            const pairs = await previewColourConversion(doc, targetSettings);
            setState((s) => ({ ...s, previewPairs: pairs, step: 'confirm', previewLoading: false }));
        } catch (e) {
            console.error('[usePdfExport] preview failed', e);
            // Preview failure is not fatal — advance to confirm anyway.
            setState((s) => ({ ...s, previewPairs: [], step: 'confirm', previewLoading: false }));
        }
    }, []);

    const runExport = useCallback(async (doc: VectorDocument, settings: PdfExportSettings) => {
        const docTitle = doc.metadata.title ?? 'document';
        let path: string | null = null;
        try {
            const result = await dialogSave({
                title: 'Export PDF/X',
                defaultPath: `${docTitle}.pdf`,
                filters: [{ name: 'PDF', extensions: ['pdf'] }],
            });
            path = result ?? null;
        } catch {
            // Dialog cancelled or errored.
            path = null;
        }
        if (!path) {
            setState((s) => ({ ...s, step: 'confirm' }));
            return;
        }
        setState((s) => ({ ...s, step: 'exporting', exportLoading: true, exportError: null }));
        try {
            await exportPdfX(doc, settings, path);
            setState((s) => ({ ...s, step: 'done', exportedPath: path, exportLoading: false }));
        } catch (e) {
            setState((s) => ({
                ...s,
                step: 'error',
                exportError: String(e),
                exportLoading: false,
            }));
        }
    }, []);

    const goToStep = useCallback((step: ExportStep) => {
        setState((s) => ({ ...s, step }));
    }, []);

    const reset = useCallback(() => {
        setState(INITIAL_STATE);
    }, []);

    const actions: PdfExportActions = { runValidation, runPreview, runExport, goToStep, reset };
    return [state, actions];
}
