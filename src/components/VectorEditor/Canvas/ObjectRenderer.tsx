import { Rect, Ellipse, Line, Path, Group } from 'react-konva';
import type { VectorObject, Paint, Transform } from '@/lib/vector/types';

function paintToKonva(paint: Paint): string {
    if (paint.type === 'None') return 'transparent';
    const { r, g, b, a } = paint.colour;
    return `rgba(${r},${g},${b},${(a / 255).toFixed(3)})`;
}

/** Decompose affine matrix into Konva-compatible position and rotation (MVP: translate + rotate). */
function decomposeTransform(t: Transform): { x: number; y: number; rotation: number } {
    const rotation = Math.atan2(t.b, t.a) * (180 / Math.PI);
    const hasSkewOrScale = Math.abs(Math.abs(t.a) - 1) > 1e-4 || Math.abs(Math.abs(t.d) - 1) > 1e-4 || Math.abs(t.b) > 1e-4;
    if (hasSkewOrScale) {
        console.warn('[ObjectRenderer] Non-trivial scale/skew transform detected; only translation applied.');
    }
    return { x: t.e, y: t.f, rotation };
}

interface Props {
    object: VectorObject;
    onSelect?: (id: string) => void;
    isSelected?: boolean;
}

export function ObjectRenderer({ object, onSelect, isSelected }: Props) {
    const { transform, style, visible, locked, id } = object;
    const { x, y, rotation } = decomposeTransform(transform);

    const fillColor = paintToKonva(style.fill);
    const strokeColor = paintToKonva(style.stroke.paint);
    const strokeWidth = style.stroke.paint.type === 'None' ? 0 : style.stroke.width;
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
        stroke: strokeColor,
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
        return (
            <Path
                {...commonProps}
                data={object.d}
            />
        );
    }

    if (object.type === 'Group') {
        return (
            <Group id={id} x={x} y={y} rotation={rotation} opacity={opacity} visible={visible} listening={!locked}>
                {object.children.map((child) => (
                    <ObjectRenderer
                        key={child.id}
                        object={child}
                        onSelect={onSelect}
                        isSelected={isSelected}
                    />
                ))}
            </Group>
        );
    }

    return null;
}
