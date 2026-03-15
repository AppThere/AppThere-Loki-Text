import { useCallback, useRef, useState } from 'react';
import { Stage, Layer, Rect } from 'react-konva';
import type Konva from 'konva';
import { useVectorStore } from '@/lib/vector/store';
import { ObjectRenderer } from './ObjectRenderer';
import { CanvasGrid } from './CanvasGrid';
import { SelectionHandles } from './SelectionHandles';
import { handleWheelZoom, buildPreviewObject, buildFinalObject, screenToDoc } from '@/lib/vector/canvasEventHandlers';
import type { VectorObject } from '@/lib/vector/types';
import { useDisplayColours } from '@/lib/vector/useDisplayColours';
import { defaultColourSettings } from '@/lib/vector/colourUtils';

interface VectorCanvasProps {
    width: number;
    height: number;
}

export function VectorCanvas({ width, height }: VectorCanvasProps) {
    const {
        document, toolMode, zoom, panX, panY,
        selectedIds, showGrid, gridSpacingPx,
        setZoom, setPan, setSelectedIds, addObject, setTool,
    } = useVectorStore();

    const [dragState, setDragState] = useState<{
        active: boolean; startX: number; startY: number;
        curX: number; curY: number; shift: boolean;
    } | null>(null);

    const isPanning = useRef(false);
    const panStart = useRef({ x: 0, y: 0, panX: 0, panY: 0 });

    const getDocPos = useCallback((e: Konva.KonvaEventObject<MouseEvent | TouchEvent>) => {
        const stage = e.target.getStage();
        const pos = stage?.getPointerPosition() ?? { x: 0, y: 0 };
        return screenToDoc(pos.x, pos.y, panX, panY, zoom);
    }, [panX, panY, zoom]);

    const handleWheel = useCallback((e: Konva.KonvaEventObject<WheelEvent>) => {
        const [nz, nx, ny] = handleWheelZoom(e, zoom, panX, panY);
        setZoom(nz);
        setPan(nx, ny);
    }, [zoom, panX, panY, setZoom, setPan]);

    const handleMouseDown = useCallback((e: Konva.KonvaEventObject<MouseEvent>) => {
        if (e.evt.button === 1 || toolMode === 'pan') {
            isPanning.current = true;
            panStart.current = { x: e.evt.clientX, y: e.evt.clientY, panX, panY };
            return;
        }
        if (toolMode === 'select') return;
        if (toolMode === 'rect' || toolMode === 'ellipse' || toolMode === 'line') {
            const [dx, dy] = getDocPos(e);
            setDragState({ active: true, startX: dx, startY: dy, curX: dx, curY: dy, shift: e.evt.shiftKey });
        }
    }, [toolMode, panX, panY, getDocPos]);

    const handleMouseMove = useCallback((e: Konva.KonvaEventObject<MouseEvent>) => {
        if (isPanning.current) {
            const dx = e.evt.clientX - panStart.current.x;
            const dy = e.evt.clientY - panStart.current.y;
            setPan(panStart.current.panX + dx, panStart.current.panY + dy);
            return;
        }
        if (dragState?.active) {
            const [dx, dy] = getDocPos(e);
            setDragState((s) => s ? { ...s, curX: dx, curY: dy, shift: e.evt.shiftKey } : s);
        }
    }, [dragState, getDocPos, setPan]);

    const handleMouseUp = useCallback((_e: Konva.KonvaEventObject<MouseEvent>) => {
        isPanning.current = false;
        if (!dragState?.active) return;
        const tool = toolMode as 'rect' | 'ellipse' | 'line';
        const obj = buildFinalObject(tool, dragState.startX, dragState.startY, dragState.curX, dragState.curY, dragState.shift);
        if (obj && (Math.abs(dragState.curX - dragState.startX) > 2 || Math.abs(dragState.curY - dragState.startY) > 2)) {
            addObject(obj);
            setSelectedIds(new Set([obj.id]));
            setTool('select');
        }
        setDragState(null);
    }, [dragState, toolMode, addObject, setSelectedIds, setTool]);

    const handleStageClick = useCallback((e: Konva.KonvaEventObject<MouseEvent>) => {
        if (toolMode !== 'select') return;
        if (e.target === e.target.getStage()) {
            setSelectedIds(new Set());
        }
    }, [toolMode, setSelectedIds]);

    const handleObjectSelect = useCallback((id: string) => {
        if (toolMode !== 'select') return;
        setSelectedIds(new Set([id]));
    }, [toolMode, setSelectedIds]);

    // Build preview shape while drawing
    let previewObj: VectorObject | null = null;
    if (dragState?.active && (toolMode === 'rect' || toolMode === 'ellipse' || toolMode === 'line')) {
        previewObj = buildPreviewObject(toolMode, dragState.startX, dragState.startY, dragState.curX, dragState.curY, dragState.shift);
    }

    const cursor = toolMode === 'pan' || isPanning.current ? 'grab'
        : (toolMode === 'rect' || toolMode === 'ellipse' || toolMode === 'line') ? 'crosshair'
        : 'default';

    const colourSettings = document?.colour_settings ?? defaultColourSettings();
    const allObjects = document?.layers.flatMap((l) => l.objects) ?? [];
    const displayCache = useDisplayColours(allObjects, colourSettings);

    const canvasW = document?.canvas.width ?? 0;
    const canvasH = document?.canvas.height ?? 0;

    return (
        // Pasteboard: fixed neutral gray, never adapts to dark mode
        <div style={{ cursor, width, height, overflow: 'hidden', background: '#c8c8c8' }}>
            <Stage
                width={width}
                height={height}
                x={panX}
                y={panY}
                scaleX={zoom}
                scaleY={zoom}
                onWheel={handleWheel}
                onMouseDown={handleMouseDown}
                onMouseMove={handleMouseMove}
                onMouseUp={handleMouseUp}
                onClick={handleStageClick}
            >
                {/* Page background: always white regardless of app theme */}
                <Layer listening={false}>
                    <Rect
                        x={0}
                        y={0}
                        width={canvasW}
                        height={canvasH}
                        fill="#ffffff"
                        shadowColor="rgba(0,0,0,0.18)"
                        shadowBlur={12}
                        shadowOffsetX={2}
                        shadowOffsetY={2}
                    />
                </Layer>

                {/* Grid layer */}
                {showGrid && (
                    <Layer listening={false}>
                        <CanvasGrid
                            width={width / zoom}
                            height={height / zoom}
                            zoom={zoom}
                            panX={0}
                            panY={0}
                            spacingPx={gridSpacingPx}
                        />
                    </Layer>
                )}

                {/* Document layers */}
                {document?.layers.map((layer) => (
                    <Layer key={layer.id} visible={layer.visible}>
                        {layer.objects.map((obj) => (
                            <ObjectRenderer
                                key={obj.id}
                                object={obj}
                                onSelect={handleObjectSelect}
                                isSelected={selectedIds.has(obj.id)}
                                displayCache={displayCache}
                            />
                        ))}
                    </Layer>
                ))}

                {/* Preview shape while drawing */}
                {previewObj && (
                    <Layer listening={false}>
                        <ObjectRenderer object={previewObj} />
                    </Layer>
                )}

                {/* Selection handles layer */}
                <Layer>
                    <SelectionHandles zoom={zoom} />
                </Layer>
            </Stage>
        </div>
    );
}
