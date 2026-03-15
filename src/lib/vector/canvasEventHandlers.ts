// Event handler logic for VectorCanvas, extracted to keep VectorCanvas ≤ 300 lines.

import type Konva from 'konva';
import type { VectorDocument, VectorObject } from './types';
import { defaultStyle, identityTransform } from './types';
import { generateId } from './storeHelpers';
import type { ToolMode } from './store';

export interface CanvasEventContext {
    document: VectorDocument | null;
    toolMode: ToolMode;
    zoom: number;
    panX: number;
    panY: number;
    setZoom: (z: number) => void;
    setPan: (x: number, y: number) => void;
    setSelectedIds: (ids: Set<string>) => void;
    addObject: (obj: VectorObject) => void;
    updateObject: (id: string, patch: Partial<VectorObject>) => void;
    setTool: (tool: ToolMode) => void;
    prevTool: ToolMode;
}

export interface DragState {
    active: boolean;
    startX: number;   // in document coords
    startY: number;
    currentX: number;
    currentY: number;
    shiftHeld: boolean;
}

/** Convert a stage pointer event position to document coordinates. */
export function screenToDoc(
    screenX: number,
    screenY: number,
    panX: number,
    panY: number,
    zoom: number,
): [number, number] {
    return [(screenX - panX) / zoom, (screenY - panY) / zoom];
}

/** Handle wheel zoom. Returns new [zoom, panX, panY]. */
export function handleWheelZoom(
    e: Konva.KonvaEventObject<WheelEvent>,
    zoom: number,
    panX: number,
    panY: number,
): [number, number, number] {
    e.evt.preventDefault();
    const stage = e.target.getStage();
    if (!stage) return [zoom, panX, panY];

    const pointer = stage.getPointerPosition() ?? { x: 0, y: 0 };
    const scaleBy = 1.05;
    const newZoom = e.evt.deltaY < 0
        ? Math.min(50, zoom * scaleBy)
        : Math.max(0.05, zoom / scaleBy);

    const newPanX = pointer.x - (pointer.x - panX) * (newZoom / zoom);
    const newPanY = pointer.y - (pointer.y - panY) * (newZoom / zoom);

    return [newZoom, newPanX, newPanY];
}

/** Build a preview object for the current draw drag. */
export function buildPreviewObject(
    tool: 'rect' | 'ellipse' | 'line',
    startX: number,
    startY: number,
    endX: number,
    endY: number,
    shiftHeld: boolean,
): VectorObject | null {
    let ex = endX;
    let ey = endY;

    if (shiftHeld) {
        const dx = ex - startX;
        const dy = ey - startY;
        const size = Math.min(Math.abs(dx), Math.abs(dy));
        ex = startX + Math.sign(dx) * size;
        ey = startY + Math.sign(dy) * size;
    }

    const id = '__preview__';
    const style = defaultStyle();
    const transform = identityTransform();

    if (tool === 'rect') {
        const x = Math.min(startX, ex);
        const y = Math.min(startY, ey);
        return {
            type: 'Rect', id, label: null, style, transform, visible: true, locked: false,
            x, y, width: Math.abs(ex - startX), height: Math.abs(ey - startY), rx: 0, ry: 0,
        };
    }
    if (tool === 'ellipse') {
        const cx = (startX + ex) / 2;
        const cy = (startY + ey) / 2;
        return {
            type: 'Ellipse', id, label: null, style, transform, visible: true, locked: false,
            cx, cy, rx: Math.abs(ex - startX) / 2, ry: Math.abs(ey - startY) / 2,
        };
    }
    if (tool === 'line') {
        return {
            type: 'Line', id, label: null, style, transform, visible: true, locked: false,
            x1: startX, y1: startY, x2: ex, y2: ey,
        };
    }
    return null;
}

/** Build the final object to add to the document. */
export function buildFinalObject(
    tool: 'rect' | 'ellipse' | 'line',
    startX: number,
    startY: number,
    endX: number,
    endY: number,
    shiftHeld: boolean,
): VectorObject | null {
    const preview = buildPreviewObject(tool, startX, startY, endX, endY, shiftHeld);
    if (!preview) return null;
    return { ...preview, id: generateId() };
}
