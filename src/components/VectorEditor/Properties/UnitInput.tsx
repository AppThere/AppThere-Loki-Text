import { useCallback, useEffect, useRef, useState } from 'react';
import type { LengthUnit } from '@/lib/vector/types';
import { fromPx, toPx, UNIT_SUFFIXES } from '@/lib/vector/unitConverter';
import { parseUnitInput } from '@/lib/vector/parseUnitInput';
import { cn } from '@/lib/utils';

interface UnitInputProps {
    value: number;           // always in px internally
    unit: LengthUnit;
    dpi?: number;
    min?: number;
    max?: number;
    step?: number;           // in display unit
    label?: string;
    onChange: (valuePx: number) => void;
    onBlur?: () => void;
    disabled?: boolean;
    className?: string;
}

function formatDisplay(px: number, unit: LengthUnit, dpi: number): string {
    const v = fromPx(px, unit, dpi);
    return parseFloat(v.toPrecision(5)).toString();
}

export function UnitInput({
    value,
    unit,
    dpi = 96,
    min,
    max,
    step = 1,
    label,
    onChange,
    onBlur,
    disabled,
    className,
}: UnitInputProps) {
    const [displayStr, setDisplayStr] = useState(() => formatDisplay(value, unit, dpi));
    const committedRef = useRef(value);
    const isFocusedRef = useRef(false);

    // Sync display when external value or unit changes — but never while the
    // user is actively editing, to avoid overwriting mid-edit text.
    useEffect(() => {
        if (isFocusedRef.current) return;
        setDisplayStr(formatDisplay(value, unit, dpi));
        committedRef.current = value;
    }, [value, unit, dpi]);

    const commit = useCallback(() => {
        const parsed = parseUnitInput(displayStr, unit, dpi, committedRef.current);
        let clamped = parsed;
        if (min !== undefined) clamped = Math.max(min, clamped);
        if (max !== undefined) clamped = Math.min(max, clamped);
        committedRef.current = clamped;
        setDisplayStr(formatDisplay(clamped, unit, dpi));
        onChange(clamped);
    }, [displayStr, unit, dpi, min, max, onChange]);

    const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter') {
            commit();
            (e.target as HTMLInputElement).blur();
        } else if (e.key === 'Escape') {
            setDisplayStr(formatDisplay(committedRef.current, unit, dpi));
        } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
            e.preventDefault();
            const multiplier = e.shiftKey ? 10 : e.altKey ? 0.1 : 1;
            const delta = (e.key === 'ArrowUp' ? 1 : -1) * step * multiplier;
            const current = fromPx(committedRef.current, unit, dpi);
            const next = current + delta;
            const nextPx = toPx(next, unit, dpi);
            let clamped = nextPx;
            if (min !== undefined) clamped = Math.max(min, clamped);
            if (max !== undefined) clamped = Math.min(max, clamped);
            committedRef.current = clamped;
            setDisplayStr(formatDisplay(clamped, unit, dpi));
            onChange(clamped);
        }
    }, [commit, unit, dpi, step, min, max, onChange]);

    return (
        <div className={cn('flex flex-col gap-0.5', className)}>
            {label && (
                <label className="text-[10px] font-medium text-muted-foreground uppercase tracking-wide">
                    {label}
                </label>
            )}
            <div className="relative flex items-center">
                <input
                    type="text"
                    inputMode="decimal"
                    value={displayStr}
                    disabled={disabled}
                    onChange={(e) => setDisplayStr(e.target.value)}
                    onFocus={() => { isFocusedRef.current = true; }}
                    onBlur={() => { isFocusedRef.current = false; commit(); onBlur?.(); }}
                    onKeyDown={handleKeyDown}
                    className={cn(
                        'w-full h-8 rounded-md border border-input bg-background px-2 pr-8 text-sm',
                        'focus:outline-none focus:ring-2 focus:ring-ring',
                        'disabled:opacity-50 disabled:cursor-not-allowed',
                    )}
                />
                <span className="pointer-events-none absolute right-2 text-xs text-muted-foreground select-none">
                    {UNIT_SUFFIXES[unit]}
                </span>
            </div>
        </div>
    );
}
