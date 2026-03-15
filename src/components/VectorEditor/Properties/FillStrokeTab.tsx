import { useVectorStore } from '@/lib/vector/store';
import { UnitInput } from './UnitInput';
import type { VectorObject, Paint, Colour } from '@/lib/vector/types';
import { cn } from '@/lib/utils';

function colourToHex(c: Colour): string {
    return '#' + [c.r, c.g, c.b].map((v) => v.toString(16).padStart(2, '0')).join('');
}

function hexToColour(hex: string): Colour {
    const r = parseInt(hex.slice(1, 3), 16) || 0;
    const g = parseInt(hex.slice(3, 5), 16) || 0;
    const b = parseInt(hex.slice(5, 7), 16) || 0;
    return { r, g, b, a: 255 };
}

function getPaintColour(paint: Paint): string {
    return paint.type === 'Solid' ? colourToHex(paint.colour) : '#000000';
}

interface Props {
    obj: VectorObject;
}

export function FillStrokeTab({ obj }: Props) {
    const { updateObject, document } = useVectorStore();
    const unit = document?.canvas.display_unit ?? 'Px';
    const dpi = document?.canvas.dpi ?? 96;

    const fillActive = obj.style.fill.type === 'Solid';
    const strokeActive = obj.style.stroke.paint.type === 'Solid';

    const updateFill = (paint: Paint) => {
        updateObject(obj.id, {
            style: { ...obj.style, fill: paint },
        } as Partial<VectorObject>);
    };

    const updateStroke = (paint: Paint) => {
        updateObject(obj.id, {
            style: { ...obj.style, stroke: { ...obj.style.stroke, paint } },
        } as Partial<VectorObject>);
    };

    const updateStrokeWidth = (px: number) => {
        updateObject(obj.id, {
            style: { ...obj.style, stroke: { ...obj.style.stroke, width: px } },
        } as Partial<VectorObject>);
    };

    return (
        <div className="space-y-4 p-3">
            {/* Fill */}
            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-foreground">Fill</span>
                    <button
                        className={cn(
                            'text-[10px] px-2 py-0.5 rounded border transition-colors',
                            fillActive ? 'border-primary text-primary' : 'border-muted text-muted-foreground',
                        )}
                        onClick={() => updateFill(fillActive
                            ? { type: 'None' }
                            : { type: 'Solid', colour: { r: 0, g: 0, b: 0, a: 255 } }
                        )}
                    >
                        {fillActive ? 'On' : 'None'}
                    </button>
                </div>
                {fillActive && (
                    <input
                        type="color"
                        value={getPaintColour(obj.style.fill)}
                        onChange={(e) => updateFill({ type: 'Solid', colour: hexToColour(e.target.value) })}
                        className="w-full h-8 rounded cursor-pointer border border-input"
                    />
                )}
            </div>

            {/* Stroke */}
            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-foreground">Stroke</span>
                    <button
                        className={cn(
                            'text-[10px] px-2 py-0.5 rounded border transition-colors',
                            strokeActive ? 'border-primary text-primary' : 'border-muted text-muted-foreground',
                        )}
                        onClick={() => updateStroke(strokeActive
                            ? { type: 'None' }
                            : { type: 'Solid', colour: { r: 0, g: 0, b: 0, a: 255 } }
                        )}
                    >
                        {strokeActive ? 'On' : 'None'}
                    </button>
                </div>
                {strokeActive && (
                    <>
                        <input
                            type="color"
                            value={getPaintColour(obj.style.stroke.paint)}
                            onChange={(e) => updateStroke({ type: 'Solid', colour: hexToColour(e.target.value) })}
                            className="w-full h-8 rounded cursor-pointer border border-input"
                        />
                        <UnitInput
                            label="Width"
                            value={obj.style.stroke.width}
                            unit={unit}
                            dpi={dpi}
                            min={0}
                            onChange={updateStrokeWidth}
                        />
                    </>
                )}
            </div>
        </div>
    );
}
