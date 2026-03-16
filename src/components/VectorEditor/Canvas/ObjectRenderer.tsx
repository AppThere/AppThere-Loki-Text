import { Rect, Ellipse, Line, Path, Group } from 'react-konva';
import type { VectorObject, Paint, Transform } from '@/lib/vector/types';
import { getDisplayColour } from '@/lib/vector/colourUtils';

function paintToKonvaFill(
    paint: Paint,
    displayCache: Map<string, string>,
    softProofOverrides: Map<string, string> | null,
): { fill: string; fillEnabled: boolean } {
    if (paint.type === 'None') {
        return { fill: 'transparent', fillEnabled: false };
    }
    return {
        fill: getDisplayColour(paint.colour, displayCache, softProofOverrides),
        fillEnabled: true,
    };
}

function paintToKonvaStroke(
    paint: Paint,
    displayCache: Map<string, string>,
    softProofOverrides: Map<string, string> | null,
): { stroke: string; strokeEnabled: boolean } {
    if (paint.type === 'None') {
        return { stroke: 'transparent', strokeEnabled: false };
    }
    return {
        stroke: getDisplayColour(paint.colour, displayCache, softProofOverrides),
        strokeEnabled: true,
    };
}

/** Decompose affine matrix into Konva-compatible position and rotation (MVP: translate + rotate). */
function decomposeTransform(t: Transform): { x: number; y: number; rotation: number } {
    const rotation = Math.atan2(t.b, t.a) * (180 / Math.PI);
    const hasSkewOrScale =
        Math.abs(Math.abs(t.a) - 1) > 1e-4 ||
        Math.abs(Math.abs(t.d) - 1) > 1e-4 ||
        Math.abs(t.b) > 1e-4;
    if (hasSkewOrScale) {
        console.warn(
            '[ObjectRenderer] Non-trivial scale/skew transform detected; only translation applied.',
        );
    }
    return { x: t.e, y: t.f, rotation };
}

interface Props {
    object: VectorObject;
    onSelect?: (id: string) => void;
    isSelected?: boolean;
    displayCache?: Map<string, string>;
    softProofOverrides?: Map<string, string> | null;
}

export function ObjectRenderer({ object, onSelect, isSelected, displayCache = new Map(), softProofOverrides = null }: Props) {
    const { transform, style, visible, locked, id } = object;
    const { x, y, rotation } = decomposeTransform(transform);

    const { fill: fillColor, fillEnabled } = paintToKonvaFill(style.fill, displayCache, softProofOverrides);
    const { stroke: strokeColor, strokeEnabled } = paintToKonvaStroke(style.stroke.paint, displayCache, softProofOverrides);
    const strokeWidth = strokeEnabled ? style.stroke.width : 0;
    const opacity = style.opacity;

    const commonProps = {
        id,
        x,
        y,
        rotation,
        opacity,
        visible,
        listening: !locked,
        fill: fillColor,
        fillEnabled,
        stroke: strokeColor,
        strokeEnabled,
        strokeWidth,
        hitStrokeWidth: 20,
        onClick: () => onSelect?.(id),
        onTap: () => onSelect?.(id),
    };

    if (object.type === 'Rect') {
        return (
            <Rect
                {...commonProps}
                x={x + object.x}
                y={y + object.y}
                width={object.width}
                height={object.height}
                cornerRadius={object.rx || 0}
            />
        );
    }

    if (object.type === 'Ellipse') {
        return (
            <Ellipse
                {...commonProps}
                x={x + object.cx}
                y={y + object.cy}
                radiusX={object.rx}
                radiusY={object.ry}
            />
        );
    }

    if (object.type === 'Line') {
        return (
            <Line
                {...commonProps}
                points={[object.x1, object.y1, object.x2, object.y2]}
                x={x}
                y={y}
            />
        );
    }

    if (object.type === 'Path') {
        return <Path {...commonProps} data={object.d} />;
    }

    if (object.type === 'Group') {
        return (
            <Group
                id={id}
                x={x}
                y={y}
                rotation={rotation}
                opacity={opacity}
                visible={visible}
                listening={!locked}
            >
                {object.children.map((child) => (
                    <ObjectRenderer
                        key={child.id}
                        object={child}
                        onSelect={onSelect}
                        isSelected={isSelected}
                        displayCache={displayCache}
                        softProofOverrides={softProofOverrides}
                    />
                ))}
            </Group>
        );
    }

    return null;
}
