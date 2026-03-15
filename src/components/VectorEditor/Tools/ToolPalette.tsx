import { useEffect, useRef } from 'react';
import { MousePointer2, RectangleHorizontal, Circle, Minus, Hand, ZoomIn } from 'lucide-react';
import { useVectorStore, type ToolMode } from '@/lib/vector/store';
import { ToolPaletteItem } from './ToolPaletteItem';
import { cn } from '@/lib/utils';

interface ToolDef {
    tool: ToolMode;
    icon: typeof MousePointer2;
    label: string;
    shortcut: string;
}

const TOOLS: ToolDef[] = [
    { tool: 'select', icon: MousePointer2, label: 'Select', shortcut: 'V' },
    { tool: 'rect', icon: RectangleHorizontal, label: 'Rectangle', shortcut: 'R' },
    { tool: 'ellipse', icon: Circle, label: 'Ellipse', shortcut: 'E' },
    { tool: 'line', icon: Minus, label: 'Line', shortcut: 'L' },
    { tool: 'pan', icon: Hand, label: 'Pan', shortcut: 'H' },
    { tool: 'zoom', icon: ZoomIn, label: 'Zoom', shortcut: 'Z' },
];

interface ToolPaletteProps {
    variant: 'sidebar' | 'bottombar';
}

export function ToolPalette({ variant }: ToolPaletteProps) {
    const { toolMode, setTool } = useVectorStore();
    const prevToolRef = useRef<ToolMode>(toolMode);

    // Global keyboard shortcuts
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
            switch (e.key.toLowerCase()) {
                case 'v': case 's': setTool('select'); break;
                case 'r': setTool('rect'); break;
                case 'e': setTool('ellipse'); break;
                case 'l': setTool('line'); break;
                case 'h': prevToolRef.current = toolMode; setTool('pan'); break;
                case 'z': setTool('zoom'); break;
                case ' ':
                    if (!e.repeat) {
                        prevToolRef.current = toolMode;
                        setTool('pan');
                    }
                    e.preventDefault();
                    break;
            }
        };
        const handleKeyUp = (e: KeyboardEvent) => {
            if (e.key === ' ' || e.key.toLowerCase() === 'h') {
                setTool(prevToolRef.current);
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        window.addEventListener('keyup', handleKeyUp);
        return () => {
            window.removeEventListener('keydown', handleKeyDown);
            window.removeEventListener('keyup', handleKeyUp);
        };
    }, [toolMode, setTool]);

    if (variant === 'sidebar') {
        return (
            <div className="flex flex-col gap-1 p-1 w-12 bg-background border-r border-border">
                {TOOLS.map((t) => (
                    <ToolPaletteItem
                        key={t.tool}
                        {...t}
                        active={toolMode === t.tool}
                        variant="sidebar"
                        onSelect={setTool}
                    />
                ))}
            </div>
        );
    }

    return (
        <div className={cn('flex h-14 bg-background border-t border-border safe-pb')}>
            {TOOLS.map((t) => (
                <ToolPaletteItem
                    key={t.tool}
                    {...t}
                    active={toolMode === t.tool}
                    variant="bottombar"
                    onSelect={setTool}
                />
            ))}
        </div>
    );
}
