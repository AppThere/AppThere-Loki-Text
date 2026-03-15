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

// Types verified from their source files before use.
// PdfExportSettings fields: standard, bleed_pt, output_condition_identifier,
//   output_condition, registry_name.
// ProfileInfo fields: id, name, description.

import { useState, useEffect } from 'react';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { useVectorStore } from '@/lib/vector/store';
import type { ProfileInfo } from '@/lib/vector/types';
import { getOutputIntentProfiles } from '@/lib/vector/commands';
import type { PdfExportSettings } from '@/lib/vector/pdfTypes';
import { DEFAULT_X4_SETTINGS, DEFAULT_X1A_SETTINGS } from '@/lib/vector/pdfTypes';
import { usePdfExport } from '@/lib/vector/usePdfExport';
import { PdfConformanceSelector } from './pdf/PdfConformanceSelector';
import { PdfOutputIntentSelector } from './pdf/PdfOutputIntentSelector';
import type { OutputIntentSelection } from './pdf/PdfOutputIntentSelector';
import { PdfBleedSettings } from './pdf/PdfBleedSettings';
import type { BleedValue } from './pdf/PdfBleedSettings';
import { ViolationList } from './pdf/ViolationList';
import { ColourPreviewGrid } from './pdf/ColourPreviewGrid';

interface ExportPdfDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

export function ExportPdfDialog({ open, onOpenChange }: ExportPdfDialogProps) {
    const { document: doc } = useVectorStore();
    const [exportState, exportActions] = usePdfExport();
    const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
    const [settings, setSettings] = useState<PdfExportSettings>(DEFAULT_X4_SETTINGS);
    const [bleed, setBleed] = useState<BleedValue>({ enabled: false, bleed_pt: 0 });

    // Determine document colour space for mismatch detection
    const docColourSpace = doc?.colour_settings?.working_space ?? { type: 'Srgb' as const };

    useEffect(() => {
        if (!open) return;
        exportActions.reset();
        const isCmyk = docColourSpace.type === 'Cmyk';
        setSettings(isCmyk ? DEFAULT_X1A_SETTINGS : DEFAULT_X4_SETTINGS);
        setBleed({ enabled: false, bleed_pt: 0 });
        getOutputIntentProfiles().then(setProfiles).catch(console.error);
    }, [open]); // eslint-disable-line react-hooks/exhaustive-deps

    const currentSettings: PdfExportSettings = {
        ...settings,
        bleed_pt: bleed.enabled ? bleed.bleed_pt : 0,
    };

    const handleOutputIntentChange = (sel: OutputIntentSelection) => {
        setSettings((s) => ({
            ...s,
            output_condition_identifier: sel.output_condition_identifier,
            output_condition: sel.output_condition,
            registry_name: sel.registry_name,
        }));
    };

    const handleValidate = () => {
        if (!doc) return;
        exportActions.runValidation(doc, currentSettings);
    };

    const handlePreview = () => {
        if (!doc) return;
        exportActions.runPreview(doc, currentSettings);
    };

    const handleExport = () => {
        if (!doc) return;
        exportActions.runExport(doc, currentSettings);
    };

    const { step, violations, previewPairs, previewLoading, exportError, exportedPath } =
        exportState;

    const errorsExist = violations.some(
        (v) => v.rule !== 'X/empty-document',
    );

    const renderBody = () => {
        if (step === 'done') {
            return (
                <div className="py-6 text-center space-y-2">
                    <p className="text-lg font-medium text-foreground">Export complete</p>
                    {exportedPath && (
                        <p className="text-sm text-muted-foreground break-all">{exportedPath}</p>
                    )}
                </div>
            );
        }

        if (step === 'error') {
            return (
                <div className="py-4 space-y-2">
                    <p className="text-sm font-medium text-destructive">Export failed</p>
                    <p className="text-sm text-muted-foreground break-all">{exportError}</p>
                </div>
            );
        }

        if (step === 'exporting') {
            return (
                <div className="py-6 text-center text-sm text-muted-foreground">
                    Exporting PDF…
                </div>
            );
        }

        if (step === 'confirm' || step === 'preview') {
            return (
                <div className="space-y-4">
                    <p className="text-sm text-muted-foreground">
                        Preview shows how colours will appear after conversion to the selected output profile.
                    </p>
                    <ColourPreviewGrid pairs={previewPairs} loading={previewLoading} />
                </div>
            );
        }

        if (step === 'validate') {
            const isValidating = violations.length === 0 && !errorsExist;
            return (
                <div className="space-y-4">
                    <ViolationList violations={violations} loading={isValidating} />
                </div>
            );
        }

        // step === 'settings'
        return (
            <div className="space-y-6">
                <div className="space-y-2">
                    <p className="text-sm font-medium">Conformance standard</p>
                    <PdfConformanceSelector
                        value={settings.standard}
                        onChange={(standard) => setSettings((s) => ({ ...s, standard }))}
                    />
                </div>

                {profiles.length > 0 && (
                    <div className="space-y-2">
                        <p className="text-sm font-medium">Output intent profile</p>
                        <PdfOutputIntentSelector
                            selectedProfileId={profiles[0].id}
                            profiles={profiles}
                            docColourSpace={docColourSpace}
                            onChange={handleOutputIntentChange}
                        />
                    </div>
                )}

                <div className="space-y-2">
                    <p className="text-sm font-medium">Bleed</p>
                    <PdfBleedSettings value={bleed} onChange={setBleed} />
                </div>
            </div>
        );
    };

    const renderFooter = () => {
        if (step === 'done') {
            return (
                <Button onClick={() => onOpenChange(false)}>Close</Button>
            );
        }

        if (step === 'error') {
            return (
                <>
                    <Button variant="outline" onClick={() => exportActions.goToStep('confirm')}>
                        Back
                    </Button>
                    <Button onClick={() => exportActions.goToStep('settings')}>
                        Start over
                    </Button>
                </>
            );
        }

        if (step === 'exporting') {
            return <Button disabled>Exporting…</Button>;
        }

        if (step === 'confirm' || step === 'preview') {
            return (
                <>
                    <Button variant="outline" onClick={() => exportActions.goToStep('validate')}>
                        Back
                    </Button>
                    <Button onClick={handleExport} disabled={previewLoading}>
                        Export PDF/X…
                    </Button>
                </>
            );
        }

        if (step === 'validate') {
            if (errorsExist) {
                return (
                    <>
                        <Button variant="outline" onClick={() => exportActions.goToStep('settings')}>
                            Back to settings
                        </Button>
                    </>
                );
            }
            const isValidating = violations.length === 0;
            return (
                <>
                    <Button variant="outline" onClick={() => exportActions.goToStep('settings')}>
                        Back
                    </Button>
                    <Button onClick={handlePreview} disabled={isValidating}>
                        Preview colours
                    </Button>
                </>
            );
        }

        // settings
        return (
            <>
                <Button variant="outline" onClick={() => onOpenChange(false)}>
                    Cancel
                </Button>
                <Button onClick={handleValidate} disabled={!doc}>
                    Validate
                </Button>
            </>
        );
    };

    const stepTitle: Record<typeof step, string> = {
        settings: 'Export PDF/X',
        validate: 'Checking conformance…',
        preview: 'Preparing colour preview…',
        confirm: 'Colour conversion preview',
        exporting: 'Exporting…',
        done: 'Export complete',
        error: 'Export failed',
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg max-h-[85vh] overflow-y-auto">
                <DialogHeader>
                    <DialogTitle>{stepTitle[step]}</DialogTitle>
                </DialogHeader>

                <div className="py-2">{renderBody()}</div>

                <DialogFooter className="flex gap-2 justify-end">{renderFooter()}</DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
