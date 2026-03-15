import type { LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { ToolMode } from '@/lib/vector/store';

interface ToolPaletteItemProps {
    tool: ToolMode;
    icon: LucideIcon;
    label: string;
    shortcut: string;
    active: boolean;
    variant: 'sidebar' | 'bottombar';
    onSelect: (tool: ToolMode) => void;
}

export function ToolPaletteItem({
    tool, icon: Icon, label, shortcut, active, variant, onSelect,
}: ToolPaletteItemProps) {
    if (variant === 'sidebar') {
        return (
            <button
                title={`${label} (${shortcut})`}
                onClick={() => onSelect(tool)}
                className={cn(
                    'w-full flex items-center justify-center h-10 rounded-md transition-colors',
                    active
                        ? 'bg-accent text-accent-foreground'
                        : 'text-muted-foreground hover:bg-muted hover:text-foreground',
                )}
            >
                <Icon className="h-5 w-5" />
            </button>
        );
    }

    return (
        <button
            onClick={() => onSelect(tool)}
            className={cn(
                'flex flex-col items-center justify-center flex-1 h-full gap-0.5 transition-colors',
                active
                    ? 'text-primary bg-primary/10'
                    : 'text-muted-foreground hover:text-foreground',
            )}
        >
            <Icon className="h-5 w-5" />
            <span className="text-[9px] font-medium">{label}</span>
        </button>
    );
}
