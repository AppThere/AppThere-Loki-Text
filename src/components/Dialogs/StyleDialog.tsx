import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Check, ChevronsUpDown, Eye } from 'lucide-react';
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
import { StyleDefinition } from '@/lib/types/odt';
import { useStyleDialog } from '@/lib/hooks/useStyleDialog';
import { StyleDialogTypographyTab } from './StyleDialogTypographyTab';
import { StyleDialogAlignmentTab } from './StyleDialogAlignmentTab';
import { StyleDialogStructureTab } from './StyleDialogStructureTab';

interface StyleDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    existingStyles: Record<string, StyleDefinition>;
    onSave: (style: StyleDefinition) => void;
    initialStyleName?: string;
}

export function StyleDialog({
    open,
    onOpenChange,
    existingStyles,
    onSave,
    initialStyleName,
}: StyleDialogProps) {
    const d = useStyleDialog({ open, existingStyles, onSave, onOpenChange, initialStyleName });

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-2xl h-[90vh] flex flex-col p-0 overflow-hidden sm:rounded-xl">
                <DialogHeader className="px-6 py-4 border-b shrink-0 bg-muted/30">
                    <div className="flex justify-between items-center mr-8">
                        <div>
                            <DialogTitle className="text-xl">
                                {d.selectedStyleId === '__new__' ? 'New Style' : 'Edit Paragraph Style'}
                            </DialogTitle>
                            <DialogDescription className="text-xs">
                                Configure typography and layout for your document.
                            </DialogDescription>
                        </div>
                    </div>
                </DialogHeader>

                <div className="flex-1 overflow-y-auto px-6 py-4 space-y-6">
                    {/* Basic Info Section */}
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <div className="space-y-1.5 flex flex-col">
                            <Label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Select Style to Edit</Label>
                            <Popover open={d.isStylePopoverOpen} onOpenChange={d.setIsStylePopoverOpen}>
                                <PopoverTrigger asChild>
                                    <Button variant="outline" role="combobox" className="w-full justify-between font-normal h-9">
                                        {d.selectedStyleId === '__new__'
                                            ? "-- Create New Style --"
                                            : (existingStyles[d.selectedStyleId]?.displayName || d.selectedStyleId)}
                                        <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                                    </Button>
                                </PopoverTrigger>
                                <PopoverContent className="w-[300px] p-0">
                                    <Command>
                                        <CommandInput placeholder="Search style..." />
                                        <CommandList>
                                            <CommandEmpty>No style found.</CommandEmpty>
                                            <CommandGroup>
                                                <CommandItem value="__new__" onSelect={() => { d.handleStyleSelect('__new__'); d.setIsStylePopoverOpen(false); }}>
                                                    <Check className={cn("mr-2 h-4 w-4", d.selectedStyleId === '__new__' ? "opacity-100" : "opacity-0")} />
                                                    -- Create New Style --
                                                </CommandItem>
                                                {Object.entries(existingStyles)
                                                    .filter(([name, s]) => s.displayName || !/^[PT]\d+$/.test(name))
                                                    .sort((a, b) => (a[1].displayName || a[0]).toLowerCase().localeCompare((b[1].displayName || b[0]).toLowerCase()))
                                                    .map(([n, s]) => (
                                                        <CommandItem key={n} value={n} onSelect={(value: string) => { d.handleStyleSelect(value); d.setIsStylePopoverOpen(false); }}>
                                                            <Check className={cn("mr-2 h-4 w-4", d.selectedStyleId === n ? "opacity-100" : "opacity-0")} />
                                                            {s.displayName || n}
                                                        </CommandItem>
                                                    ))}
                                            </CommandGroup>
                                        </CommandList>
                                    </Command>
                                </PopoverContent>
                            </Popover>
                        </div>
                        <div className="space-y-1.5">
                            <Label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mr-2">Style Name</Label>
                            <Input value={d.displayName} onChange={(e) => d.setDisplayName(e.target.value)} placeholder="e.g., Body Text" className="h-9" />
                        </div>
                    </div>

                    <Tabs defaultValue="typography" className="w-full">
                        <TabsList className="grid w-full grid-cols-3 h-9">
                            <TabsTrigger value="typography" className="text-xs py-1.5">Typography</TabsTrigger>
                            <TabsTrigger value="spacing" className="text-xs py-1.5">Alignment</TabsTrigger>
                            <TabsTrigger value="structure" className="text-xs py-1.5">Structure</TabsTrigger>
                        </TabsList>

                        <TabsContent value="typography" className="space-y-4 pt-4">
                            <StyleDialogTypographyTab
                                fontFamily={d.fontFamily} setFontFamily={d.setFontFamily}
                                fontSize={d.fontSize} setFontSize={d.setFontSize}
                                fontWeight={d.fontWeight} setFontWeight={d.setFontWeight}
                                fontStyle={d.fontStyle} setFontStyle={d.setFontStyle}
                                color={d.color} setColor={d.setColor}
                                textTransform={d.textTransform} setTextTransform={d.setTextTransform}
                                isFontPopoverOpen={d.isFontPopoverOpen} setIsFontPopoverOpen={d.setIsFontPopoverOpen}
                                inheritedProps={d.inheritedProps}
                            />
                        </TabsContent>

                        <TabsContent value="spacing" className="space-y-4 pt-4">
                            <StyleDialogAlignmentTab
                                textAlign={d.textAlign} setTextAlign={d.setTextAlign}
                                lineHeight={d.lineHeight} setLineHeight={d.setLineHeight}
                                marginTop={d.marginTop} setMarginTop={d.setMarginTop}
                                marginBottom={d.marginBottom} setMarginBottom={d.setMarginBottom}
                                marginLeft={d.marginLeft} setMarginLeft={d.setMarginLeft}
                                marginRight={d.marginRight} setMarginRight={d.setMarginRight}
                                textIndent={d.textIndent} setTextIndent={d.setTextIndent}
                                contextualSpacing={d.contextualSpacing} setContextualSpacing={d.setContextualSpacing}
                                inheritedProps={d.inheritedProps}
                            />
                        </TabsContent>

                        <TabsContent value="structure" className="space-y-4 pt-4">
                            <StyleDialogStructureTab
                                parent={d.parent} setParent={d.setParent}
                                next={d.next} setNext={d.setNext}
                                outlineLevel={d.outlineLevel} setOutlineLevel={d.setOutlineLevel}
                                breakBefore={d.breakBefore} setBreakBefore={d.setBreakBefore}
                                breakAfter={d.breakAfter} setBreakAfter={d.setBreakAfter}
                                orphans={d.orphans} setOrphans={d.setOrphans}
                                widows={d.widows} setWidows={d.setWidows}
                                keepWithNext={d.keepWithNext} setKeepWithNext={d.setKeepWithNext}
                                hyphenation={d.hyphenation} setHyphenation={d.setHyphenation}
                                isParentPopoverOpen={d.isParentPopoverOpen} setIsParentPopoverOpen={d.setIsParentPopoverOpen}
                                isNextPopoverOpen={d.isNextPopoverOpen} setIsNextPopoverOpen={d.setIsNextPopoverOpen}
                                existingStyles={existingStyles}
                                family={d.family}
                                selectedStyleId={d.selectedStyleId}
                            />
                        </TabsContent>
                    </Tabs>

                    {/* Preview Section */}
                    <div className="mt-8 pt-6 border-t">
                        <div className="flex items-center gap-2 mb-3">
                            <Eye className="w-4 h-4 text-primary" />
                            <Label className="text-xs font-bold uppercase tracking-[0.1em] text-primary">Live Preview</Label>
                        </div>
                        <div className="border rounded-lg p-6 bg-slate-50 dark:bg-stone-900 overflow-hidden min-h-[120px] flex items-center justify-center">
                            <div className="w-full transition-all duration-200" style={d.previewStyle}>
                                The quick brown fox jumps over the lazy dog. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                            </div>
                        </div>
                    </div>
                </div>

                <DialogFooter className="px-6 py-4 border-t bg-muted/30 shrink-0">
                    <Button variant="ghost" onClick={() => onOpenChange(false)} className="h-9">Cancel</Button>
                    <Button onClick={d.handleSave} className="px-8 h-9 shadow-lg">Save Style</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
