import { useCallback } from 'react';
import { Check, ChevronsUpDown } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
} from "@/components/ui/command";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/components/ui/popover";
import type { StyleDefinition } from '@/lib/types/odt';

interface ToolbarStyleSelectorProps {
    styles: Record<string, StyleDefinition>;
    currentStyle?: string;
    onStyleChange?: (styleName: string) => void;
    isOpen: boolean;
    onOpenChange: (open: boolean) => void;
}

export function ToolbarStyleSelector({
    styles,
    currentStyle,
    onStyleChange,
    isOpen,
    onOpenChange,
}: ToolbarStyleSelectorProps) {
    const resolveBaseStyle = useCallback((styleName: string): string => {
        const isInternal = (name: string) => /^[PT]\d+$/.test(name);

        let current = styles[styleName];
        if (!current) return styleName;
        if (current.displayName || !isInternal(current.name)) return styleName;

        const visited = new Set<string>([styleName]);
        while (current && !current.displayName && isInternal(current.name) && current.parent) {
            if (visited.has(current.parent)) break;
            visited.add(current.parent);
            current = styles[current.parent];
        }
        return current?.name || styleName;
    }, [styles]);

    return (
        <Popover open={isOpen} onOpenChange={onOpenChange}>
            <PopoverTrigger asChild>
                <Button
                    variant="outline"
                    role="combobox"
                    className="w-48 h-8 ml-1 bg-white dark:bg-slate-900 justify-between font-normal text-slate-700 dark:text-slate-200"
                >
                    <span className="truncate">
                        {(() => {
                            if (!currentStyle) return "Normal Text";
                            const baseName = resolveBaseStyle(currentStyle);
                            const style = styles[baseName] || styles[currentStyle];
                            return style?.displayName || style?.name || currentStyle;
                        })()}
                    </span>
                    <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                </Button>
            </PopoverTrigger>
            <PopoverContent className="w-48 p-0">
                <Command>
                    <CommandInput placeholder="Search style..." className="h-8" />
                    <CommandEmpty>No style found.</CommandEmpty>
                    <CommandGroup className="max-h-60 overflow-y-auto">
                        {Object.entries(styles)
                            .filter(([name, style]) => {
                                if (style.family !== 'Paragraph') return false;
                                if (style.displayName) return true;
                                return !/^[PT]\d+$/.test(name);
                            })
                            .sort((a, b) => {
                                const nameA = (a[1].displayName || a[0]).toLowerCase();
                                const nameB = (b[1].displayName || b[0]).toLowerCase();
                                return nameA.localeCompare(nameB);
                            })
                            .map(([name, style]) => {
                                const baseStyleName = currentStyle ? resolveBaseStyle(currentStyle) : null;
                                const isSelected = currentStyle === name || baseStyleName === name;

                                return (
                                    <CommandItem
                                        key={name}
                                        value={name}
                                        onSelect={(currentValue: string) => {
                                            onStyleChange?.(currentValue);
                                            onOpenChange(false);
                                        }}
                                    >
                                        <Check
                                            className={cn(
                                                "mr-2 h-4 w-4",
                                                isSelected ? "opacity-100" : "opacity-0"
                                            )}
                                        />
                                        {style.displayName || name}
                                    </CommandItem>
                                );
                            })}
                    </CommandGroup>
                </Command>
            </PopoverContent>
        </Popover>
    );
}
