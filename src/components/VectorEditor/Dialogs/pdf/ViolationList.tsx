// Copyright 2024 AppThere
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import type { ConformanceViolation } from '@/lib/vector/pdfTypes';
import { groupViolationsByLevel } from '@/lib/vector/pdfTypes';
import { cn } from '@/lib/utils';

interface ViolationRowProps {
    violation: ConformanceViolation;
    variant: 'error' | 'warning' | 'auto-fix';
}

function ViolationRow({ violation, variant }: ViolationRowProps) {
    const containerClass = {
        error: 'border-destructive/50 bg-destructive/5',
        warning: 'border-yellow-400/50 bg-yellow-50/50 dark:bg-yellow-900/10',
        'auto-fix': 'border-green-400/50 bg-green-50/50 dark:bg-green-900/10',
    }[variant];

    const badgeClass = {
        error: 'bg-destructive text-destructive-foreground',
        warning: 'bg-yellow-400 text-yellow-900',
        'auto-fix': 'bg-green-600 text-white',
    }[variant];

    const badgeLabel = {
        error: 'Error',
        warning: 'Warning',
        'auto-fix': 'Auto-fix',
    }[variant];

    return (
        <li className={cn('rounded-md border p-3 text-sm', containerClass)}>
            <div className="flex items-start gap-2">
                <span
                    className={cn(
                        'mt-0.5 shrink-0 text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded',
                        badgeClass,
                    )}
                >
                    {badgeLabel}
                </span>
                <div className="min-w-0">
                    <p className="font-medium text-foreground">{violation.message}</p>
                    {variant === 'auto-fix' && (
                        <p className="mt-0.5 text-xs text-green-700 dark:text-green-400">
                            Will be handled automatically during export
                        </p>
                    )}
                    <p className="mt-0.5 text-xs text-muted-foreground font-mono">
                        {violation.rule}
                    </p>
                </div>
            </div>
        </li>
    );
}

interface ViolationListProps {
    violations: ConformanceViolation[];
    /** If true, shows a skeleton loading state. */
    loading?: boolean;
}

export function ViolationList({ violations, loading = false }: ViolationListProps) {
    if (loading) {
        return (
            <ul className="space-y-2">
                {[1, 2].map((i) => (
                    <li key={i} className="rounded-md border border-border p-3 animate-pulse">
                        <div className="h-4 bg-muted rounded w-3/4" />
                        <div className="mt-2 h-3 bg-muted rounded w-1/2" />
                    </li>
                ))}
            </ul>
        );
    }

    if (violations.length === 0) {
        return (
            <p className="text-sm text-muted-foreground text-center py-4">
                No violations found. Document is conformant.
            </p>
        );
    }

    const { errors, warnings, autoFixed } = groupViolationsByLevel(violations);

    return (
        <div className="space-y-3">
            {errors.length > 0 && (
                <div>
                    <p className="text-xs font-semibold text-destructive mb-1.5">
                        {errors.length} {errors.length === 1 ? 'Error' : 'Errors'} — must fix before export
                    </p>
                    <ul className="space-y-2">
                        {errors.map((v) => (
                            <ViolationRow key={v.rule} violation={v} variant="error" />
                        ))}
                    </ul>
                </div>
            )}
            {autoFixed.length > 0 && (
                <div>
                    <p className="text-xs font-semibold text-green-700 dark:text-green-400 mb-1.5">
                        {autoFixed.length} {autoFixed.length === 1 ? 'Issue' : 'Issues'} — handled automatically
                    </p>
                    <ul className="space-y-2">
                        {autoFixed.map((v) => (
                            <ViolationRow key={v.rule} violation={v} variant="auto-fix" />
                        ))}
                    </ul>
                </div>
            )}
            {warnings.length > 0 && (
                <div>
                    <p className="text-xs font-semibold text-yellow-700 dark:text-yellow-400 mb-1.5">
                        {warnings.length} {warnings.length === 1 ? 'Warning' : 'Warnings'}
                    </p>
                    <ul className="space-y-2">
                        {warnings.map((v) => (
                            <ViolationRow key={v.rule} violation={v} variant="warning" />
                        ))}
                    </ul>
                </div>
            )}
        </div>
    );
}
