import { Check, ChevronsUpDown } from 'lucide-react';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Checkbox } from '@/components/ui/checkbox';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import {
    Command,
    CommandEmpty,
    CommandGroup,
    CommandInput,
    CommandItem,
    CommandList,
} from "@/components/ui/command";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/components/ui/popover";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { StyleDefinition, StyleFamily } from '@/lib/types/odt';

interface StyleDialogStructureTabProps {
    parent: string; setParent: (v: string) => void;
    next: string; setNext: (v: string) => void;
    outlineLevel: string; setOutlineLevel: (v: string) => void;
    breakBefore: boolean; setBreakBefore: (v: boolean) => void;
    breakAfter: boolean; setBreakAfter: (v: boolean) => void;
    orphans: string; setOrphans: (v: string) => void;
    widows: string; setWidows: (v: string) => void;
    keepWithNext: boolean; setKeepWithNext: (v: boolean) => void;
    hyphenation: boolean; setHyphenation: (v: boolean) => void;
    isParentPopoverOpen: boolean; setIsParentPopoverOpen: (v: boolean) => void;
    isNextPopoverOpen: boolean; setIsNextPopoverOpen: (v: boolean) => void;
    existingStyles: Record<string, StyleDefinition>;
    family: StyleFamily;
    selectedStyleId: string;
}

export function StyleDialogStructureTab({
    parent, setParent,
    next, setNext,
    outlineLevel, setOutlineLevel,
    breakBefore, setBreakBefore,
    breakAfter, setBreakAfter,
    orphans, setOrphans,
    widows, setWidows,
    keepWithNext, setKeepWithNext,
    hyphenation, setHyphenation,
    isParentPopoverOpen, setIsParentPopoverOpen,
    isNextPopoverOpen, setIsNextPopoverOpen,
    existingStyles,
    family,
    selectedStyleId,
}: StyleDialogStructureTabProps) {
    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4">
            <div className="space-y-1.5 flex flex-col">
                <Label className="text-xs">Parent Style</Label>
                <Popover open={isParentPopoverOpen} onOpenChange={setIsParentPopoverOpen}>
                    <PopoverTrigger asChild>
                        <Button variant="outline" role="combobox" className="w-full justify-between font-normal h-8 text-xs">
                            {parent && parent !== 'none' ? (existingStyles[parent]?.displayName || parent) : "None"}
                            <ChevronsUpDown className="ml-2 h-3 w-3 opacity-50" />
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent className="w-[200px] p-0">
                        <Command>
                            <CommandInput placeholder="Search style..." className="h-8 text-xs" />
                            <CommandList>
                                <CommandEmpty className="text-xs p-2">No style found.</CommandEmpty>
                                <CommandGroup>
                                    <CommandItem value="none" onSelect={() => { setParent('none'); setIsParentPopoverOpen(false); }} className="text-xs py-1">
                                        <Check className={cn("mr-2 h-3 w-3", parent === 'none' || !parent ? "opacity-100" : "opacity-0")} />
                                        None
                                    </CommandItem>
                                    {Object.entries(existingStyles)
                                        .filter(([name, s]) => s.family === family && name !== selectedStyleId && (s.displayName || !/^[PT]\d+$/.test(name)))
                                        .sort((a, b) => (a[1].displayName || a[0]).localeCompare(b[1].displayName || b[0]))
                                        .map(([n, s]) => (
                                            <CommandItem key={n} value={n} onSelect={(value: string) => { setParent(value); setIsParentPopoverOpen(false); }} className="text-xs py-1">
                                                <Check className={cn("mr-2 h-3 w-3", parent === n ? "opacity-100" : "opacity-0")} />
                                                {s.displayName || n}
                                            </CommandItem>
                                        ))}
                                </CommandGroup>
                            </CommandList>
                        </Command>
                    </PopoverContent>
                </Popover>
            </div>
            <div className="space-y-1.5 flex flex-col">
                <Label className="text-xs">Next Style</Label>
                <Popover open={isNextPopoverOpen} onOpenChange={setIsNextPopoverOpen}>
                    <PopoverTrigger asChild>
                        <Button variant="outline" role="combobox" className="w-full justify-between font-normal h-8 text-xs">
                            {next && next !== 'none' ? (existingStyles[next]?.displayName || next) : "Same style"}
                            <ChevronsUpDown className="ml-2 h-3 w-3 opacity-50" />
                        </Button>
                    </PopoverTrigger>
                    <PopoverContent className="w-[200px] p-0">
                        <Command>
                            <CommandInput placeholder="Search style..." className="h-8 text-xs" />
                            <CommandList>
                                <CommandEmpty className="text-xs p-2">No style found.</CommandEmpty>
                                <CommandGroup>
                                    <CommandItem value="none" onSelect={() => { setNext('none'); setIsNextPopoverOpen(false); }} className="text-xs py-1">
                                        <Check className={cn("mr-2 h-3 w-3", next === 'none' || !next ? "opacity-100" : "opacity-0")} />
                                        Same style
                                    </CommandItem>
                                    {Object.entries(existingStyles)
                                        .filter(([name, s]) => s.family === StyleFamily.Paragraph && (s.displayName || !/^[PT]\d+$/.test(name)))
                                        .sort((a, b) => (a[1].displayName || a[0]).localeCompare(b[1].displayName || b[0]))
                                        .map(([n, s]) => (
                                            <CommandItem key={n} value={n} onSelect={(value: string) => { setNext(value); setIsNextPopoverOpen(false); }} className="text-xs py-1">
                                                <Check className={cn("mr-2 h-3 w-3", next === n ? "opacity-100" : "opacity-0")} />
                                                {s.displayName || n}
                                            </CommandItem>
                                        ))}
                                </CommandGroup>
                            </CommandList>
                        </Command>
                    </PopoverContent>
                </Popover>
            </div>
            <div className="space-y-4 sm:col-span-2 grid grid-cols-2 gap-4">
                <div className="space-y-1.5">
                    <Label className="text-xs">Heading Level</Label>
                    <Select value={outlineLevel} onValueChange={setOutlineLevel}>
                        <SelectTrigger className="h-8 text-xs"><SelectValue placeholder="None" /></SelectTrigger>
                        <SelectContent>
                            <SelectItem value="none">None (Body)</SelectItem>
                            <SelectItem value="1">Level 1</SelectItem>
                            <SelectItem value="2">Level 2</SelectItem>
                            <SelectItem value="3">Level 3</SelectItem>
                            <SelectItem value="4">Level 4</SelectItem>
                        </SelectContent>
                    </Select>
                </div>
                <div className="space-y-1.5">
                    <Label className="text-xs">Pagination</Label>
                    <div className="space-y-2 pt-1">
                        <div className="flex items-center space-x-2">
                            <Checkbox id="breakBefore" checked={breakBefore} onCheckedChange={(v) => setBreakBefore(!!v)} />
                            <Label htmlFor="breakBefore" className="text-xs font-normal">Page break before</Label>
                        </div>
                        <div className="flex items-center space-x-2">
                            <Checkbox id="breakAfter" checked={breakAfter} onCheckedChange={(v) => setBreakAfter(!!v)} />
                            <Label htmlFor="breakAfter" className="text-xs font-normal">Page break after</Label>
                        </div>
                        <div className="flex items-center space-x-2">
                            <Checkbox id="keepNext" checked={keepWithNext} onCheckedChange={(v) => setKeepWithNext(!!v)} />
                            <Label htmlFor="keepNext" className="text-xs font-normal">Keep with next</Label>
                        </div>
                    </div>
                </div>
                <div className="space-y-2">
                    <Label className="text-xs">Orphans / Widows</Label>
                    <div className="flex gap-2">
                        <Input value={orphans} onChange={(e) => setOrphans(e.target.value)} placeholder="Opt" title="Orphans" className="h-8 text-xs w-full" />
                        <Input value={widows} onChange={(e) => setWidows(e.target.value)} placeholder="Wid" title="Widows" className="h-8 text-xs w-full" />
                    </div>
                </div>
                <div className="flex items-center space-x-2 pt-4">
                    <Checkbox id="hyphen" checked={hyphenation} onCheckedChange={(v) => setHyphenation(!!v)} />
                    <Label htmlFor="hyphen" className="text-xs font-normal">Allow Hyphenation</Label>
                </div>
            </div>
        </div>
    );
}
