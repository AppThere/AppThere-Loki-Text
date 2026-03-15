/**
 * Lightweight toast state management following the shadcn/ui pattern.
 *
 * `toast()` is a standalone function callable from anywhere — inside or
 * outside React components. `useToast()` subscribes to the shared state
 * and exposes the active toast list along with a dismiss helper.
 */
import { useState, useEffect } from 'react';

export type ToastVariant = 'default' | 'destructive';

export interface ToastOptions {
    title: string;
    description?: string;
    variant?: ToastVariant;
}

export interface ToastItem extends ToastOptions {
    id: string;
    open: boolean;
}

type Listener = (toasts: ToastItem[]) => void;

// ── Module-level singleton state ───────────────────────────────────────────

let toastCount = 0;
let activeToasts: ToastItem[] = [];
const listeners: Listener[] = [];

function emit(): void {
    for (const l of listeners) l([...activeToasts]);
}

/** Show a toast notification. Safe to call from outside React components. */
export function toast(options: ToastOptions): void {
    const id = String(++toastCount);
    activeToasts = [{ ...options, id, open: true }, ...activeToasts].slice(0, 3);
    emit();
}

// ── Hook ──────────────────────────────────────────────────────────────────

export function useToast() {
    const [toasts, setToasts] = useState<ToastItem[]>(activeToasts);

    useEffect(() => {
        listeners.push(setToasts);
        return () => {
            const i = listeners.indexOf(setToasts);
            if (i !== -1) listeners.splice(i, 1);
        };
    }, []);

    function dismiss(id: string): void {
        activeToasts = activeToasts.map((t) =>
            t.id === id ? { ...t, open: false } : t,
        );
        emit();
        // Remove from list after exit animation completes
        setTimeout(() => {
            activeToasts = activeToasts.filter((t) => t.id !== id);
            emit();
        }, 300);
    }

    return { toasts, dismiss };
}
