// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useRef, useLayoutEffect, useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { Printer, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { usePageStyleStore } from '@/lib/stores/pageStyleStore';
import { mmToPx, effectivePageDimensions } from '@/editor/page/pageGeometry';
import { splitIntoPages } from './previewSerializer';
import type { PreviewPage } from './previewSerializer';
import './PrintPreviewView.css';

interface PrintPreviewViewProps {
    /** Serialised HTML snapshot of the document body. */
    snapshotHtml: string;
}

/**
 * Read-only print preview. Renders the document as individual page cards
 * sized to the active PageStyle, with a toolbar (Print button) and an
 * edit-disabled banner.
 */
export function PrintPreviewView({ snapshotHtml }: PrintPreviewViewProps) {
    const { t } = useTranslation('common');
    const { pageStyle } = usePageStyleStore();
    const [pages, setPages] = useState<PreviewPage[]>([]);
    const measureRef = useRef<HTMLDivElement>(null);

    const { width: pageWidthMm, height: pageHeightMm } = effectivePageDimensions(pageStyle);
    const { margins } = pageStyle;

    const pageWidthPx = mmToPx(pageWidthMm);
    const pageHeightPx = mmToPx(pageHeightMm);
    const paddingTopPx = mmToPx(margins.top);
    const paddingBottomPx = mmToPx(margins.bottom);
    const paddingLeftPx = mmToPx(margins.inner);
    const paddingRightPx = mmToPx(margins.outer);

    // Measure block heights in a hidden off-screen container, then split.
    useLayoutEffect(() => {
        const el = measureRef.current;
        if (!el) return;
        el.innerHTML = snapshotHtml;
        const split = splitIntoPages(el, pageStyle);
        setPages(split);
    }, [snapshotHtml, pageStyle]);

    const handlePrint = useCallback(() => {
        window.print();
    }, []);

    return (
        <div className="print-preview-outer flex-1 flex flex-col min-h-0">
            {/* Edit-disabled banner */}
            <div className="print-preview-banner" role="status">
                <AlertCircle className="h-4 w-4 shrink-0" />
                <span>{t('preview.editingDisabledBanner')}</span>
            </div>

            {/* Toolbar */}
            <div className="print-preview-toolbar">
                <Button
                    variant="outline"
                    size="sm"
                    onClick={handlePrint}
                    className="gap-2"
                >
                    <Printer className="h-4 w-4" />
                    {t('preview.printButton')}
                </Button>
            </div>

            {/* Hidden measurement container */}
            <div
                ref={measureRef}
                aria-hidden="true"
                style={{
                    position: 'absolute',
                    visibility: 'hidden',
                    pointerEvents: 'none',
                    width: pageWidthPx - paddingLeftPx - paddingRightPx,
                    top: -9999,
                    left: -9999,
                }}
            />

            {/* Page cards */}
            <div className="print-preview-view">
                {pages.map((page, idx) => (
                    <PageCard
                        key={idx}
                        page={page}
                        pageNumber={idx + 1}
                        totalPages={pages.length}
                        pageWidthPx={pageWidthPx}
                        pageHeightPx={pageHeightPx}
                        paddingTopPx={paddingTopPx}
                        paddingBottomPx={paddingBottomPx}
                        paddingLeftPx={paddingLeftPx}
                        paddingRightPx={paddingRightPx}
                        label={t('preview.pageLabel', {
                            current: idx + 1,
                            total: pages.length,
                        })}
                    />
                ))}
                {pages.length === 0 && (
                    <div className="text-sm text-muted-foreground p-4">
                        {snapshotHtml.trim() === '' ? t('preview.emptyDocument') : null}
                    </div>
                )}
            </div>
        </div>
    );
}

// ---------------------------------------------------------------------------
// Sub-component
// ---------------------------------------------------------------------------

interface PageCardProps {
    page: PreviewPage;
    pageNumber: number;
    totalPages: number;
    pageWidthPx: number;
    pageHeightPx: number;
    paddingTopPx: number;
    paddingBottomPx: number;
    paddingLeftPx: number;
    paddingRightPx: number;
    label: string;
}

function PageCard({
    page,
    pageWidthPx,
    pageHeightPx,
    paddingTopPx,
    paddingBottomPx,
    paddingLeftPx,
    paddingRightPx,
    label,
}: PageCardProps) {
    return (
        <div
            className="print-preview-page"
            style={{
                width: pageWidthPx,
                minHeight: pageHeightPx,
                paddingTop: paddingTopPx,
                paddingBottom: paddingBottomPx,
                paddingLeft: paddingLeftPx,
                paddingRight: paddingRightPx,
            }}
        >
            {/* eslint-disable-next-line react/no-danger */}
            <div dangerouslySetInnerHTML={{ __html: page.html }} />
            <span className="print-preview-page-label" aria-hidden="true">
                {label}
            </span>
        </div>
    );
}
