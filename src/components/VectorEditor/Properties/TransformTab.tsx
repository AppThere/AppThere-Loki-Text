import { useVectorStore } from '@/lib/vector/store';
import { UnitInput } from './UnitInput';
import type { VectorObject } from '@/lib/vector/types';

interface Props {
    obj: VectorObject;
}

function getObjectXY(obj: VectorObject): [number, number] {
    if (obj.type === 'Rect') return [obj.x + obj.transform.e, obj.y + obj.transform.f];
    if (obj.type === 'Ellipse') return [obj.cx - obj.rx + obj.transform.e, obj.cy - obj.ry + obj.transform.f];
    if (obj.type === 'Line') return [Math.min(obj.x1, obj.x2) + obj.transform.e, Math.min(obj.y1, obj.y2) + obj.transform.f];
    return [obj.transform.e, obj.transform.f];
}

function getObjectWH(obj: VectorObject): [number, number] {
    if (obj.type === 'Rect') return [obj.width, obj.height];
    if (obj.type === 'Ellipse') return [obj.rx * 2, obj.ry * 2];
    if (obj.type === 'Line') return [Math.abs(obj.x2 - obj.x1), Math.abs(obj.y2 - obj.y1)];
    return [0, 0];
}

function getRotation(obj: VectorObject): number {
    return Math.atan2(obj.transform.b, obj.transform.a) * (180 / Math.PI);
}

export function TransformTab({ obj }: Props) {
    const { updateObject, document } = useVectorStore();
    const unit = document?.canvas.display_unit ?? 'Px';
    const dpi = document?.canvas.dpi ?? 96;

    const [x, y] = getObjectXY(obj);
    const [w, h] = getObjectWH(obj);
    const rotation = getRotation(obj);

    const updateX = (px: number) => {
        const patch: Partial<VectorObject> = { transform: { ...obj.transform, e: px } };
        if (obj.type === 'Rect') Object.assign(patch, { x: 0 });
        updateObject(obj.id, patch);
    };

    const updateY = (px: number) => {
        const patch: Partial<VectorObject> = { transform: { ...obj.transform, f: px } };
        if (obj.type === 'Rect') Object.assign(patch, { y: 0 });
        updateObject(obj.id, patch);
    };

    const updateW = (px: number) => {
        if (obj.type === 'Rect') updateObject(obj.id, { width: Math.max(1, px) } as Partial<VectorObject>);
        if (obj.type === 'Ellipse') updateObject(obj.id, { rx: Math.max(0.5, px / 2) } as Partial<VectorObject>);
    };

    const updateH = (px: number) => {
        if (obj.type === 'Rect') updateObject(obj.id, { height: Math.max(1, px) } as Partial<VectorObject>);
        if (obj.type === 'Ellipse') updateObject(obj.id, { ry: Math.max(0.5, px / 2) } as Partial<VectorObject>);
    };

    return (
        <div className="space-y-3 p-3">
            <div className="grid grid-cols-2 gap-2">
                <UnitInput label="X" value={x} unit={unit} dpi={dpi} onChange={updateX} />
                <UnitInput label="Y" value={y} unit={unit} dpi={dpi} onChange={updateY} />
                <UnitInput label="W" value={w} unit={unit} dpi={dpi} min={1} onChange={updateW} />
                <UnitInput label="H" value={h} unit={unit} dpi={dpi} min={1} onChange={updateH} />
            </div>
            <UnitInput
                label="Rotation (°)"
                value={rotation}
                unit="Px"
                dpi={dpi}
                min={0}
                max={360}
                step={1}
                onChange={(deg) => {
                    const rad = deg * (Math.PI / 180);
                    const cos = Math.cos(rad);
                    const sin = Math.sin(rad);
                    updateObject(obj.id, {
                        transform: { ...obj.transform, a: cos, b: sin, c: -sin, d: cos },
                    } as Partial<VectorObject>);
                }}
            />
        </div>
    );
}
