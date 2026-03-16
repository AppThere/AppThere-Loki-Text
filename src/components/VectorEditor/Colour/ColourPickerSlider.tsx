import { useRef, useCallback } from 'react';

interface Props {
    /** 0.0–1.0 current value */
    value: number;
    onChange: (value: number) => void;
    /** CSS gradient string for the track background */
    gradient: string;
    label: string;
    /** Formatted display value (e.g. "128" or "50") */
    displayValue: string;
    onTextChange: (raw: string) => void;
}

/**
 * A draggable colour channel slider with a gradient track.
 * The track height is 20px so it satisfies touch targets when combined
 * with surrounding padding.
 */
export function ColourPickerSlider({
    value,
    onChange,
    gradient,
    label,
    displayValue,
    onTextChange,
}: Props) {
    const trackRef = useRef<HTMLDivElement>(null);
    const dragging = useRef(false);

    const clamp = (v: number) => Math.max(0, Math.min(1, v));

    const posFromEvent = useCallback((clientX: number): number => {
        const track = trackRef.current;
        if (!track) return value;
        const rect = track.getBoundingClientRect();
        return clamp((clientX - rect.left) / rect.width);
    }, [value]);

    const handlePointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
        dragging.current = true;
        (e.currentTarget as HTMLDivElement).setPointerCapture(e.pointerId);
        onChange(posFromEvent(e.clientX));
    };

    const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
        if (!dragging.current) return;
        onChange(posFromEvent(e.clientX));
    };

    const handlePointerUp = () => {
        dragging.current = false;
    };

    const thumbLeft = `${Math.round(value * 100)}%`;

    return (
        <div className="flex items-center gap-2">
            <span className="text-[10px] text-muted-foreground w-8 shrink-0 text-right select-none">
                {label}
            </span>

            {/* Track */}
            <div
                ref={trackRef}
                className="relative flex-1 h-5 rounded cursor-pointer select-none touch-none"
                style={{ background: gradient }}
                onPointerDown={handlePointerDown}
                onPointerMove={handlePointerMove}
                onPointerUp={handlePointerUp}
            >
                {/* Thumb */}
                <div
                    className="absolute top-0 bottom-0 w-3 -translate-x-1/2 rounded border-2 border-white shadow pointer-events-none"
                    style={{ left: thumbLeft, background: 'transparent' }}
                />
            </div>

            {/* Text input */}
            <input
                type="text"
                inputMode="numeric"
                value={displayValue}
                onChange={(e) => onTextChange(e.target.value)}
                className="w-10 h-6 text-center text-xs rounded border border-input bg-background px-0"
                aria-label={label}
            />
        </div>
    );
}
