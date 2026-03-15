import { useVectorStore } from '@/lib/vector/store';
import { UnitInput } from './UnitInput';
import type { VectorObject, Paint, Colour } from '@/lib/vector/types';
import { cn } from '@/lib/utils';

function getDisplayColour(colour: Colour): { r: number; g: number; b: number; a: number } {
    if (colour.type === 'Rgb') {
        return { r: colour.r * 255, g: colour.g * 255, b: colour.b * 255, a: colour.a };
    }
    // For non-RGB colours in Phase 1, show a placeholder.
    // Phase 2 will add proper CMYK/Spot picker support.
    console.warn('[FillStrokeTab] Non-RGB colour in Phase 1 picker:', colour);
    return { r: 0, g: 0, b: 0, a: 1 };
}

function setDisplayColour(r: number, g: number, b: number, a: number): Colour {
    // Phase 1 picker always produces Rgb
    return { type: 'Rgb', r: r / 255, g: g / 255, b: b / 255, a };
}

function colourToHex(colour: Colour): string {
    const { r, g, b } = getDisplayColour(colour);
    return (
        '#' +
        [r, g, b]
            .map((v) => Math.round(v).toString(16).padStart(2, '0'))
            .join('')
    );
}

function getPaintColour(paint: Paint): string {
    return paint.type === 'Solid' ? colourToHex(paint.colour) : '#000000';
}

/** Returns a badge label for non-RGB colours to signal Phase 1 limitation. */
function nonRgbBadge(colour: Colour): string | null {
    if (colour.type === 'Rgb') return null;
    return colour.type;
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

    const fillBadge =
        fillActive && obj.style.fill.type === 'Solid'
            ? nonRgbBadge(obj.style.fill.colour)
            : null;
    const strokeBadge =
        strokeActive && obj.style.stroke.paint.type === 'Solid'
            ? nonRgbBadge(obj.style.stroke.paint.colour)
            : null;

    return (
        <div className="space-y-4 p-3">
            {/* Fill */}
            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-foreground">Fill</span>
                    <button
                        className={cn(
                            'text-[10px] px-2 py-0.5 rounded border transition-colors',
                            fillActive
                                ? 'border-primary text-primary'
                                : 'border-muted text-muted-foreground',
                        )}
                        onClick={() =>
                            updateFill(
                                fillActive
                                    ? { type: 'None' }
                                    : { type: 'Solid', colour: { type: 'Rgb', r: 0, g: 0, b: 0, a: 1 } },
                            )
                        }
                    >
                        {fillActive ? 'On' : 'None'}
                    </button>
                </div>
                {fillActive && (
                    <div className="relative">
                        <input
                            type="color"
                            value={getPaintColour(obj.style.fill)}
                            onChange={(e) => {
                                const hex = e.target.value;
                                const r = parseInt(hex.slice(1, 3), 16) || 0;
                                const g = parseInt(hex.slice(3, 5), 16) || 0;
                                const b = parseInt(hex.slice(5, 7), 16) || 0;
                                updateFill({
                                    type: 'Solid',
                                    colour: setDisplayColour(r, g, b, 1),
                                });
                            }}
                            className="w-full h-8 rounded cursor-pointer border border-input"
                        />
                        {fillBadge && (
                            <span
                                className="absolute right-1 top-1 text-[9px] bg-muted text-muted-foreground px-1 rounded"
                                title="CMYK editing requires CMYK document mode (coming in Phase 2)"
                            >
                                {fillBadge}
                            </span>
                        )}
                    </div>
                )}
            </div>

            {/* Stroke */}
            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-foreground">Stroke</span>
                    <button
                        className={cn(
                            'text-[10px] px-2 py-0.5 rounded border transition-colors',
                            strokeActive
                                ? 'border-primary text-primary'
                                : 'border-muted text-muted-foreground',
                        )}
                        onClick={() =>
                            updateStroke(
                                strokeActive
                                    ? { type: 'None' }
                                    : { type: 'Solid', colour: { type: 'Rgb', r: 0, g: 0, b: 0, a: 1 } },
                            )
                        }
                    >
                        {strokeActive ? 'On' : 'None'}
                    </button>
                </div>
                {strokeActive && (
                    <>
                        <div className="relative">
                            <input
                                type="color"
                                value={getPaintColour(obj.style.stroke.paint)}
                                onChange={(e) => {
                                    const hex = e.target.value;
                                    const r = parseInt(hex.slice(1, 3), 16) || 0;
                                    const g = parseInt(hex.slice(3, 5), 16) || 0;
                                    const b = parseInt(hex.slice(5, 7), 16) || 0;
                                    updateStroke({
                                        type: 'Solid',
                                        colour: setDisplayColour(r, g, b, 1),
                                    });
                                }}
                                className="w-full h-8 rounded cursor-pointer border border-input"
                            />
                            {strokeBadge && (
                                <span
                                    className="absolute right-1 top-1 text-[9px] bg-muted text-muted-foreground px-1 rounded"
                                    title="CMYK editing requires CMYK document mode (coming in Phase 2)"
                                >
                                    {strokeBadge}
                                </span>
                            )}
                        </div>
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
