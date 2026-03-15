import { toast } from '@/lib/hooks/useToast';

/**
 * Formats a caught error into a destructive toast payload.
 * Exported for unit testing; prefer `notifyError` at call sites.
 */
export function formatError(
    context: string,
    err: unknown,
): { title: string; description: string; variant: 'destructive' } {
    return {
        title: context,
        description: err instanceof Error ? err.message : String(err),
        variant: 'destructive',
    };
}

/**
 * Shows a destructive toast for a caught error.
 *
 * Call this alongside (not instead of) `console.error` in catch blocks so
 * debugging context is preserved in the console and the user gets feedback.
 */
export function notifyError(context: string, err: unknown): void {
    toast(formatError(context, err));
}
