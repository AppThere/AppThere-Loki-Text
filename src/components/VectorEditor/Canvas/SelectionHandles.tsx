import { useCallback } from 'react';
import { Rect, Circle, Line, Group } from 'react-konva';
import type Konva from 'konva';
import { useVectorStore } from '@/lib/vector/store';
import type { VectorObject } from '@/lib/vector/types';

interface BBox { x: number; y: number; w: number; h: number }

function getBBox(obj: VectorObject): BBox | null {
    if (obj.type === 'Rect') {
        return { x: obj.x + obj.transform.e, y: obj.y + obj.transform.f, w: obj.width, h: obj.height };
    }
    if (obj.type === 'Ellipse') {
        return {
            x: obj.cx - obj.rx + obj.transform.e,
            y: obj.cy - obj.ry + obj.transform.f,
            w: obj.rx * 2, h: obj.ry * 2,
        };
    }
    if (obj.type === 'Line') {
        const minX = Math.min(obj.x1, obj.x2) + obj.transform.e;
        const minY = Math.min(obj.y1, obj.y2) + obj.transform.f;
        return { x: minX, y: minY, w: Math.abs(obj.x2 - obj.x1), h: Math.abs(obj.y2 - obj.y1) };
    }
    return null;
}

const HANDLE_RADIUS = 5;
const ROTATION_OFFSET = 24;
const HANDLE_FILL = '#ffffff';
const HANDLE_STROKE = '#1a73e8';
const BOX_STROKE = '#1a73e8';

interface Props {
    zoom: number;
}

export function SelectionHandles({ zoom }: Props) {
    const { document, selectedIds, updateObject } = useVectorStore();
    if (!document || selectedIds.size === 0) return null;

    // Find selected objects
    const selected: VectorObject[] = [];
    for (const layer of document.layers) {
        for (const obj of layer.objects) {
            if (selectedIds.has(obj.id)) selected.push(obj);
        }
    }
    if (selected.length === 0) return null;

    // Compute union bounding box
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const obj of selected) {
        const bb = getBBox(obj);
        if (!bb) continue;
        minX = Math.min(minX, bb.x);
        minY = Math.min(minY, bb.y);
        maxX = Math.max(maxX, bb.x + bb.w);
        maxY = Math.max(maxY, bb.y + bb.h);
    }
    if (!isFinite(minX)) return null;

    const bw = maxX - minX;
    const bh = maxY - minY;
    const cx = minX + bw / 2;
    const hr = HANDLE_RADIUS / zoom;
    const strokeW = 1 / zoom;

    const handles = [
        { key: 'tl', hx: minX, hy: minY },
        { key: 'tm', hx: cx, hy: minY },
        { key: 'tr', hx: maxX, hy: minY },
        { key: 'ml', hx: minX, hy: minY + bh / 2 },
        { key: 'mr', hx: maxX, hy: minY + bh / 2 },
        { key: 'bl', hx: minX, hy: maxY },
        { key: 'bm', hx: cx, hy: maxY },
        { key: 'br', hx: maxX, hy: maxY },
    ];

    // For MVP: single-object resize via drag on corner handles
    const isSingle = selected.length === 1;
    const singleObj = isSingle ? selected[0] : null;

    const handleCornerDrag = useCallback((handleKey: string, e: Konva.KonvaEventObject<DragEvent>) => {
        if (!singleObj || singleObj.type !== 'Rect') return;
        const pos = e.target.position();
        if (handleKey === 'br') {
            const newW = Math.max(1, pos.x - singleObj.x - singleObj.transform.e);
            const newH = Math.max(1, pos.y - singleObj.y - singleObj.transform.f);
            updateObject(singleObj.id, { width: newW, height: newH } as Partial<VectorObject>);
        }
    }, [singleObj, updateObject]);

    return (
        <Group>
            {/* Bounding box */}
            <Rect
                x={minX}
                y={minY}
                width={bw}
                height={bh}
                stroke={BOX_STROKE}
                strokeWidth={strokeW}
                dash={[4 / zoom, 4 / zoom]}
                fill="transparent"
                listening={false}
            />
            {/* Rotation handle */}
            <Line
                points={[cx, minY, cx, minY - ROTATION_OFFSET / zoom]}
                stroke={BOX_STROKE}
                strokeWidth={strokeW}
                listening={false}
            />
            <Circle
                x={cx}
                y={minY - ROTATION_OFFSET / zoom}
                radius={hr}
                fill={HANDLE_FILL}
                stroke={HANDLE_STROKE}
                strokeWidth={strokeW}
                draggable={false}
            />
            {/* Corner/edge handles */}
            {handles.map(({ key, hx, hy }) => (
                <Circle
                    key={key}
                    x={hx}
                    y={hy}
                    radius={hr}
                    fill={HANDLE_FILL}
                    stroke={HANDLE_STROKE}
                    strokeWidth={strokeW}
                    draggable={isSingle}
                    onDragMove={(e) => handleCornerDrag(key, e)}
                />
            ))}
        </Group>
    );
}
