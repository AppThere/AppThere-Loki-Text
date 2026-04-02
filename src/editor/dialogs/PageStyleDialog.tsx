// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { usePageStyleStore } from '@/lib/stores/pageStyleStore';
import {
    type PageStyle, type PaperSizeKey,
    effectivePageDimensions, defaultPageStyle,
} from '@/editor/page/pageGeometry';
import { PagePreviewSvg } from './PagePreviewSvg';

interface PageStyleDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

const PAPER_SIZES: Array<PaperSizeKey | 'custom'> = ['A4', 'A5', 'A3', 'Letter', 'Legal', 'Tabloid', 'custom'];

function NumInput({ id, value, onChange, min = 0, max = 1200, step = 0.5, label }: {
    id: string; value: number; onChange: (v: number) => void;
    min?: number; max?: number; step?: number; label: string;
}) {
    return (
        <div className="flex items-center gap-2">
            <Label htmlFor={id} className="w-16 text-right text-sm shrink-0">{label}</Label>
            <div className="flex items-center gap-1">
                <input
                    id={id} type="number" value={value} step={step} min={min} max={max}
                    onChange={(e) => onChange(parseFloat(e.target.value) || 0)}
                    className="w-20 h-8 border rounded px-2 text-sm"
                />
                <span className="text-sm text-muted-foreground">mm</span>
            </div>
        </div>
    );
}

export function PageStyleDialog({ open, onOpenChange }: PageStyleDialogProps) {
    const { t } = useTranslation('common');
    const { pageStyle, setPageStyle } = usePageStyleStore();
    const [local, setLocal] = useState<PageStyle>(pageStyle);

    useEffect(() => {
        if (open) setLocal(pageStyle);
    }, [open, pageStyle]);

    const update = (patch: Partial<PageStyle>) => setLocal((s) => ({ ...s, ...patch }));
    const updateMargin = (k: keyof PageStyle['margins'], v: number) =>
        setLocal((s) => ({ ...s, margins: { ...s.margins, [k]: v } }));

    const dims = effectivePageDimensions(local);

    const handleOk = () => {
        setPageStyle(local);
        onOpenChange(false);
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[500px]">
                <DialogHeader>
                    <DialogTitle>{t('pageStyle.title')}</DialogTitle>
                </DialogHeader>

                <Tabs defaultValue="page">
                    <TabsList className="w-full">
                        <TabsTrigger value="page" className="flex-1">{t('pageStyle.tabPage')}</TabsTrigger>
                        <TabsTrigger value="margins" className="flex-1">{t('pageStyle.tabMargins')}</TabsTrigger>
                    </TabsList>

                    {/* ── Page tab ── */}
                    <TabsContent value="page" className="space-y-4 pt-2">
                        <div className="flex gap-6 items-start">
                            <div className="space-y-3 flex-1">
                                <div className="space-y-1">
                                    <Label>{t('pageStyle.paperFormat')}</Label>
                                    <Select value={local.paperSize} onValueChange={(v) => update({ paperSize: v as PaperSizeKey | 'custom' })}>
                                        <SelectTrigger className="h-8"><SelectValue /></SelectTrigger>
                                        <SelectContent>
                                            {PAPER_SIZES.map((s) => (
                                                <SelectItem key={s} value={s}>{t(`pageStyle.size.${s}`)}</SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>

                                {local.paperSize === 'custom' && (
                                    <div className="space-y-2">
                                        <NumInput id="cw" label={t('pageStyle.width')} value={local.customWidth ?? 210}
                                            onChange={(v) => update({ customWidth: v })} min={50} max={1200} />
                                        <NumInput id="ch" label={t('pageStyle.height')} value={local.customHeight ?? 297}
                                            onChange={(v) => update({ customHeight: v })} min={50} max={1200} />
                                    </div>
                                )}

                                <div className="space-y-1">
                                    <Label>{t('pageStyle.orientation')}</Label>
                                    <div className="flex gap-2">
                                        {(['portrait', 'landscape'] as const).map((o) => (
                                            <Button key={o} size="sm" variant={local.orientation === o ? 'default' : 'outline'}
                                                onClick={() => update({ orientation: o })}>
                                                {t(`pageStyle.${o}`)}
                                            </Button>
                                        ))}
                                    </div>
                                </div>
                            </div>

                            {/* Live preview */}
                            <div className="shrink-0">
                                <PagePreviewSvg widthMm={dims.width} heightMm={dims.height} margins={local.margins} />
                            </div>
                        </div>
                    </TabsContent>

                    {/* ── Margins tab ── */}
                    <TabsContent value="margins" className="space-y-4 pt-2">
                        <div className="space-y-2">
                            <NumInput id="mt" label={t('pageStyle.marginTop')}    value={local.margins.top}    onChange={(v) => updateMargin('top', v)}    max={100} />
                            <NumInput id="mb" label={t('pageStyle.marginBottom')} value={local.margins.bottom} onChange={(v) => updateMargin('bottom', v)} max={100} />
                            <NumInput id="mi" label={local.duplex ? t('pageStyle.marginInner') : t('pageStyle.marginLeft')}
                                value={local.margins.inner} onChange={(v) => updateMargin('inner', v)} max={100} />
                            <NumInput id="mo" label={local.duplex ? t('pageStyle.marginOuter') : t('pageStyle.marginRight')}
                                value={local.margins.outer} onChange={(v) => updateMargin('outer', v)} max={100} />
                        </div>

                        <div className="flex items-center gap-2">
                            <Checkbox id="duplex" checked={local.duplex}
                                onCheckedChange={(c) => update({ duplex: c === true })} />
                            <Label htmlFor="duplex" className="cursor-pointer" title={t('pageStyle.duplexTooltip')}>
                                {t('pageStyle.duplexLabel')}
                            </Label>
                        </div>

                        <div className="flex gap-2 pt-1">
                            {([
                                { key: 'narrow', v: 12.7 },
                                { key: 'normal', v: 25.4 },
                                { key: 'wide',   v: 38.1 },
                            ] as const).map(({ key, v }) => (
                                <Button key={key} size="sm" variant="outline"
                                    onClick={() => update({ margins: { top: v, bottom: v, inner: v, outer: v } })}>
                                    {t(`pageStyle.preset.${key}`)}
                                </Button>
                            ))}
                        </div>
                    </TabsContent>
                </Tabs>

                <DialogFooter>
                    <Button variant="outline" onClick={() => { setLocal(defaultPageStyle); }}>{t('pageStyle.reset')}</Button>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>{t('dialog.cancel')}</Button>
                    <Button onClick={handleOk}>{t('dialog.ok')}</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
