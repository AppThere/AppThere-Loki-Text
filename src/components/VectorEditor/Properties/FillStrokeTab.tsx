import { useVectorStore } from '@/lib/vector/store';
import { UnitInput } from './UnitInput';
import type { VectorObject, Paint, Colour, DocumentColourSettings } from '@/lib/vector/types';
import { cn } from '@/lib/utils';
import { ColourPicker } from '../Colour/ColourPicker';

interface Props {
    obj: VectorObject;
    colourSettings: DocumentColourSettings;
    displayCache: Map<string, string>;
}

function defaultSolidColour(): Colour {
    return { type: 'Rgb', r: 0, g: 0, b: 0, a: 1 };
}

export function FillStrokeTab({ obj, colourSettings, displayCache }: Props) {
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

    const fillColour =
        obj.style.fill.type === 'Solid' ? obj.style.fill.colour : defaultSolidColour();
    const strokeColour =
        obj.style.stroke.paint.type === 'Solid'
            ? obj.style.stroke.paint.colour
            : defaultSolidColour();

    const toggleBtn = (active: boolean, onToggle: () => void) => (
        <button
            className={cn(
                'text-[10px] px-2 py-0.5 rounded border transition-colors min-h-[44px] min-w-[44px]',
                active
                    ? 'border-primary text-primary'
                    : 'border-muted text-muted-foreground',
            )}
            onClick={onToggle}
        >
            {active ? 'On' : 'None'}
        </button>
    );

    return (
        <div className="space-y-4 p-3">
            {/* Fill */}
            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-foreground">Fill</span>
                    {toggleBtn(fillActive, () =>
                        updateFill(
                            fillActive
                                ? { type: 'None' }
                                : { type: 'Solid', colour: defaultSolidColour() },
                        ),
                    )}
                </div>
                {fillActive && (
                    <ColourPicker
                        colour={fillColour}
                        onChange={(c) => updateFill({ type: 'Solid', colour: c })}
                        colourSettings={colourSettings}
                        displayCache={displayCache}
                    />
                )}
            </div>

            {/* Stroke */}
            <div className="space-y-2">
                <div className="flex items-center justify-between">
                    <span className="text-xs font-semibold text-foreground">Stroke</span>
                    {toggleBtn(strokeActive, () =>
                        updateStroke(
                            strokeActive
                                ? { type: 'None' }
                                : { type: 'Solid', colour: defaultSolidColour() },
                        ),
                    )}
                </div>
                {strokeActive && (
                    <>
                        <ColourPicker
                            colour={strokeColour}
                            onChange={(c) => updateStroke({ type: 'Solid', colour: c })}
                            colourSettings={colourSettings}
                            displayCache={displayCache}
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
