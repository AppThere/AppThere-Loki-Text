// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import type { MarginSpec } from '@/editor/page/pageGeometry';

interface PagePreviewSvgProps {
    widthMm: number;
    heightMm: number;
    margins: MarginSpec;
}

const MAX_HEIGHT_PX = 120;

export function PagePreviewSvg({ widthMm, heightMm, margins }: PagePreviewSvgProps) {
    const scale = MAX_HEIGHT_PX / heightMm;
    const svgW = widthMm * scale;
    const svgH = heightMm * scale;

    const mt = margins.top * scale;
    const mb = margins.bottom * scale;
    const ml = margins.inner * scale;
    const mr = margins.outer * scale;

    const bodyX = ml;
    const bodyY = mt;
    const bodyW = svgW - ml - mr;
    const bodyH = svgH - mt - mb;

    return (
        <svg
            width={Math.round(svgW)}
            height={Math.round(svgH)}
            viewBox={`0 0 ${svgW} ${svgH}`}
            aria-label="Page preview"
            style={{ display: 'block', border: '1px solid #ccc', background: '#fff' }}
        >
            {/* Page background */}
            <rect x={0} y={0} width={svgW} height={svgH} fill="#fff" />
            {/* Content area */}
            <rect
                x={bodyX}
                y={bodyY}
                width={bodyW > 0 ? bodyW : 0}
                height={bodyH > 0 ? bodyH : 0}
                fill="none"
                stroke="#aab"
                strokeWidth={0.8}
                strokeDasharray="2,2"
            />
        </svg>
    );
}
