import { useState, useEffect, useMemo } from 'react';
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
import { Checkbox } from '@/components/ui/checkbox';
import { cn } from '@/lib/utils';
import { Check, ChevronsUpDown, Eye } from 'lucide-react';
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

interface StyleDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    existingStyles: Record<string, StyleDefinition>;
    onSave: (style: StyleDefinition) => void;
    initialStyleName?: string;
}

const COMMON_FONTS = [
    "Courier Prime",
    "Atkinson Hyperlegible Next",
    "Newsreader",
    "Bitter",
    "Cormorant Garamond",
    "Public Sans",
    "Geist",
    "Bodoni Moda",
    "Lexend",
    "Caveat",
    "Roboto Flex",
    "System UI",
    "Times New Roman",
    "Arial",
    "Georgia"
];

const slugify = (text: string) => {
    return text
        .toString()
        .normalize('NFD')                   // split accented characters into their base characters and diacritical marks
        .replace(/[\u0300-\u036f]/g, '')   // remove all the accents, which happen to be all in the \u03xx range 
        .trim()
        .replace(/\s+/g, '_')              // replace spaces with underscores
        .replace(/[^\w-]+/g, '')            // remove all non-word chars
        .replace(/--+/g, '_');             // replace multiple underscores with single underscore
};

export function StyleDialog({
    open,
    onOpenChange,
    existingStyles,
    onSave,
    initialStyleName,
}: StyleDialogProps) {
    const [selectedStyleId, setSelectedStyleId] = useState<string>('__new__');
    const [name, setName] = useState('');
    const [displayName, setDisplayName] = useState('');
    // Always Paragraph for this dialog
    const family = StyleFamily.Paragraph;
    const [parent, setParent] = useState('');
    const [next, setNext] = useState('');

    // Typography & Display states
    const [fontSize, setFontSize] = useState('');
    const [fontFamily, setFontFamily] = useState('');
    const [fontWeight, setFontWeight] = useState('');
    const [fontStyle, setFontStyle] = useState('');
    const [color, setColor] = useState('');
    const [textTransform, setTextTransform] = useState('');

    // Alignment & Spacing states
    const [textAlign, setTextAlign] = useState('');
    const [lineHeight, setLineHeight] = useState('');
    const [marginTop, setMarginTop] = useState('');
    const [marginBottom, setMarginBottom] = useState('');
    const [marginLeft, setMarginLeft] = useState('');
    const [marginRight, setMarginRight] = useState('');
    const [textIndent, setTextIndent] = useState('');

    // Phase 14 Expansion states
    const [breakBefore, setBreakBefore] = useState(false);
    const [breakAfter, setBreakAfter] = useState(false);
    const [orphans, setOrphans] = useState('');
    const [widows, setWidows] = useState('');
    const [keepWithNext, setKeepWithNext] = useState(false);
    const [hyphenation, setHyphenation] = useState(false);
    const [contextualSpacing, setContextualSpacing] = useState(false);
    const [outlineLevel, setOutlineLevel] = useState('none');

    // Popover open states
    const [isStylePopoverOpen, setIsStylePopoverOpen] = useState(false);
    const [isFontPopoverOpen, setIsFontPopoverOpen] = useState(false);
    const [isParentPopoverOpen, setIsParentPopoverOpen] = useState(false);
    const [isNextPopoverOpen, setIsNextPopoverOpen] = useState(false);

    const handleStyleSelect = (styleId: string) => {
        setSelectedStyleId(styleId);
        if (styleId === '__new__') {
            setName('');
            setDisplayName('');
            setParent('');
            setNext('');

            setFontSize(''); setFontFamily(''); setFontWeight(''); setFontStyle(''); setColor(''); setTextTransform('');
            setTextAlign(''); setLineHeight(''); setMarginTop(''); setMarginBottom(''); setMarginLeft(''); setMarginRight(''); setTextIndent('');

            setBreakBefore(false); setBreakAfter(false); setOrphans(''); setWidows(''); setKeepWithNext(false); setHyphenation(false); setContextualSpacing(false); setOutlineLevel('none');
        } else {
            const style = existingStyles[styleId];
            if (style) {
                setName(style.name);
                setDisplayName(style.displayName || '');
                setParent(style.parent || '');
                setNext(style.next || '');

                const attrs = style.attributes || {};
                setFontSize(attrs['fo:font-size'] || '');
                setFontFamily(attrs['fo:font-family'] || attrs['style:font-name'] || '');
                setFontWeight(attrs['fo:font-weight'] || '');
                setFontStyle(attrs['fo:font-style'] || '');
                setColor(attrs['fo:color'] || '');
                setTextTransform(style.textTransform || '');

                setTextAlign(attrs['fo:text-align'] || '');
                setLineHeight(attrs['fo:line-height'] || '');
                setMarginTop(attrs['fo:margin-top'] || '');
                setMarginBottom(attrs['fo:margin-bottom'] || '');
                setMarginLeft(attrs['fo:margin-left'] || '');
                setMarginRight(attrs['fo:margin-right'] || '');
                setTextIndent(attrs['fo:text-indent'] || '');

                setBreakBefore(attrs['style:break-before'] === 'page');
                setBreakAfter(attrs['style:break-after'] === 'page');
                setOrphans(attrs['fo:orphans'] || '');
                setWidows(attrs['fo:widows'] || '');
                setKeepWithNext(attrs['fo:keep-with-next'] === 'always');
                setHyphenation(attrs['fo:hyphenate'] === 'true');
                setContextualSpacing(attrs['style:contextual-spacing'] === 'true');
                setOutlineLevel(style.outlineLevel?.toString() || 'none');
            }
        }
    };

    useEffect(() => {
        if (open) {
            handleStyleSelect(initialStyleName || '__new__');
        }
    }, [open, initialStyleName]);

    // Automate name (identifier) generation from displayName if creating NEW
    useEffect(() => {
        if (selectedStyleId === '__new__' && displayName) {
            const slug = slugify(displayName);
            setName(slug);
        }
    }, [displayName, selectedStyleId]);

    const handleSave = () => {
        // Ensure name is unique if NEW
        let finalName = name;
        if (selectedStyleId === '__new__') {
            let counter = 1;
            while (existingStyles[finalName]) {
                finalName = `${name}_${counter}`;
                counter++;
            }
        }

        const baseAttributes = (selectedStyleId !== '__new__' && existingStyles[selectedStyleId]) ? existingStyles[selectedStyleId].attributes : {};

        const attributes = {
            ...baseAttributes,
        };

        if (fontSize) attributes['fo:font-size'] = fontSize; else delete attributes['fo:font-size'];
        if (fontFamily) {
            attributes['fo:font-family'] = fontFamily;
            delete attributes['style:font-name'];
        } else {
            delete attributes['fo:font-family'];
        }
        if (fontWeight) attributes['fo:font-weight'] = fontWeight; else delete attributes['fo:font-weight'];
        if (fontStyle) attributes['fo:font-style'] = fontStyle; else delete attributes['fo:font-style'];
        if (color) attributes['fo:color'] = color; else delete attributes['fo:color'];

        if (textAlign) attributes['fo:text-align'] = textAlign; else delete attributes['fo:text-align'];
        if (lineHeight) attributes['fo:line-height'] = lineHeight; else delete attributes['fo:line-height'];
        if (marginTop) attributes['fo:margin-top'] = marginTop; else delete attributes['fo:margin-top'];
        if (marginBottom) attributes['fo:margin-bottom'] = marginBottom; else delete attributes['fo:margin-bottom'];
        if (marginLeft) attributes['fo:margin-left'] = marginLeft; else delete attributes['fo:margin-left'];
        if (marginRight) attributes['fo:margin-right'] = marginRight; else delete attributes['fo:margin-right'];
        if (textIndent) attributes['fo:text-indent'] = textIndent; else delete attributes['fo:text-indent'];

        if (breakBefore) attributes['style:break-before'] = 'page'; else delete attributes['style:break-before'];
        if (breakAfter) attributes['style:break-after'] = 'page'; else delete attributes['style:break-after'];
        if (orphans) attributes['fo:orphans'] = orphans; else delete attributes['fo:orphans'];
        if (widows) attributes['fo:widows'] = widows; else delete attributes['fo:widows'];
        if (keepWithNext) attributes['fo:keep-with-next'] = 'always'; else delete attributes['fo:keep-with-next'];
        if (hyphenation) attributes['fo:hyphenate'] = 'true'; else delete attributes['fo:hyphenate'];
        if (contextualSpacing) attributes['style:contextual-spacing'] = 'true'; else delete attributes['style:contextual-spacing'];

        const style: StyleDefinition = {
            name: finalName,
            displayName: displayName || null,
            family,
            parent: parent || null,
            next: next || null,
            attributes,
            textTransform: textTransform || null,
            outlineLevel: outlineLevel === 'none' ? null : parseInt(outlineLevel, 10),
            autocomplete: null,
        };
        onSave(style);
        onOpenChange(false);
    };

    const previewStyle = useMemo(() => {
        const s: React.CSSProperties = {};
        if (fontFamily) s.fontFamily = fontFamily;
        if (fontSize) s.fontSize = fontSize;
        if (fontWeight === 'bold') s.fontWeight = 'bold';
        else if (fontWeight && fontWeight !== 'none') s.fontWeight = fontWeight;

        if (fontStyle === 'italic') s.fontStyle = 'italic';
        else if (fontStyle === 'oblique') s.fontStyle = 'oblique';

        if (color) s.color = color;
        if (textTransform && textTransform !== 'none') s.textTransform = textTransform as any;
        if (textAlign && textAlign !== 'none') s.textAlign = (textAlign === 'start' ? 'left' : textAlign === 'end' ? 'right' : textAlign) as any;
        if (lineHeight) s.lineHeight = lineHeight;

        // Simulating indents/margins faintly
        if (marginTop) s.marginTop = '4px';
        if (marginBottom) s.marginBottom = '4px';
        if (textIndent) s.textIndent = '12px';

        return s;
    }, [fontFamily, fontSize, fontWeight, fontStyle, color, textTransform, textAlign, lineHeight, marginTop, marginBottom, textIndent]);

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-2xl h-[90vh] flex flex-col p-0 overflow-hidden sm:rounded-xl">
                <DialogHeader className="px-6 py-4 border-b shrink-0 bg-muted/30">
                    <div className="flex justify-between items-center mr-8">
                        <div>
                            <DialogTitle className="text-xl">
                                {selectedStyleId === '__new__' ? 'New Style' : 'Edit Paragraph Style'}
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
                            <Popover open={isStylePopoverOpen} onOpenChange={setIsStylePopoverOpen}>
                                <PopoverTrigger asChild>
                                    <Button
                                        variant="outline"
                                        role="combobox"
                                        className="w-full justify-between font-normal h-9"
                                    >
                                        {selectedStyleId === '__new__'
                                            ? "-- Create New Style --"
                                            : (existingStyles[selectedStyleId]?.displayName || selectedStyleId)}
                                        <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                                    </Button>
                                </PopoverTrigger>
                                <PopoverContent className="w-[300px] p-0">
                                    <Command>
                                        <CommandInput placeholder="Search style..." />
                                        <CommandList>
                                            <CommandEmpty>No style found.</CommandEmpty>
                                            <CommandGroup>
                                                <CommandItem
                                                    value="__new__"
                                                    onSelect={() => {
                                                        handleStyleSelect('__new__');
                                                        setIsStylePopoverOpen(false);
                                                    }}
                                                >
                                                    <Check
                                                        className={cn(
                                                            "mr-2 h-4 w-4",
                                                            selectedStyleId === '__new__' ? "opacity-100" : "opacity-0"
                                                        )}
                                                    />
                                                    -- Create New Style --
                                                </CommandItem>
                                                {Object.entries(existingStyles)
                                                    .filter(([name, s]) => s.displayName || !/^[PT]\d+$/.test(name))
                                                    .sort((a, b) => {
                                                        const nameA = (a[1].displayName || a[0]).toLowerCase();
                                                        const nameB = (b[1].displayName || b[0]).toLowerCase();
                                                        return nameA.localeCompare(nameB);
                                                    })
                                                    .map(([n, s]) => (
                                                        <CommandItem
                                                            key={n}
                                                            value={n}
                                                            onSelect={(value: string) => {
                                                                handleStyleSelect(value);
                                                                setIsStylePopoverOpen(false);
                                                            }}
                                                        >
                                                            <Check
                                                                className={cn(
                                                                    "mr-2 h-4 w-4",
                                                                    selectedStyleId === n ? "opacity-100" : "opacity-0"
                                                                )}
                                                            />
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
                            <Input
                                value={displayName}
                                onChange={(e) => setDisplayName(e.target.value)}
                                placeholder="e.g., Body Text"
                                className="h-9"
                            />
                        </div>
                    </div>

                    <Tabs defaultValue="typography" className="w-full">
                        <TabsList className="grid w-full grid-cols-3 h-9">
                            <TabsTrigger value="typography" className="text-xs py-1.5">Typography</TabsTrigger>
                            <TabsTrigger value="spacing" className="text-xs py-1.5">Alignment</TabsTrigger>
                            <TabsTrigger value="structure" className="text-xs py-1.5">Structure</TabsTrigger>
                        </TabsList>

                        <TabsContent value="typography" className="space-y-4 pt-4">
                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4">
                                <div className="space-y-1.5 flex flex-col">
                                    <Label className="text-xs">Font Family</Label>
                                    <Popover open={isFontPopoverOpen} onOpenChange={setIsFontPopoverOpen}>
                                        <PopoverTrigger asChild>
                                            <Button variant="outline" role="combobox" className="w-full justify-between font-normal h-8 text-xs">
                                                {fontFamily || "Select font..."}
                                                <ChevronsUpDown className="ml-2 h-3 w-3 opacity-50" />
                                            </Button>
                                        </PopoverTrigger>
                                        <PopoverContent className="w-[200px] p-0">
                                            <Command>
                                                <CommandInput placeholder="Search font..." className="h-8 text-xs" />
                                                <CommandList>
                                                    <CommandEmpty className="text-xs p-2">No font found.</CommandEmpty>
                                                    <CommandGroup>
                                                        {COMMON_FONTS.map((font) => (
                                                            <CommandItem
                                                                key={font}
                                                                value={font}
                                                                onSelect={() => {
                                                                    setFontFamily(font);
                                                                    setIsFontPopoverOpen(false);
                                                                }}
                                                                className="text-xs py-1"
                                                            >
                                                                <Check className={cn("mr-2 h-3 w-3", fontFamily === font ? "opacity-100" : "opacity-0")} />
                                                                {font}
                                                            </CommandItem>
                                                        ))}
                                                    </CommandGroup>
                                                </CommandList>
                                            </Command>
                                        </PopoverContent>
                                    </Popover>
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Font Size</Label>
                                    <Input value={fontSize} onChange={(e) => setFontSize(e.target.value)} placeholder="e.g., 12pt" className="h-8 text-xs" />
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Weight</Label>
                                    <Select value={fontWeight} onValueChange={setFontWeight}>
                                        <SelectTrigger className="h-8 text-xs"><SelectValue placeholder="Normal" /></SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="none">Normal</SelectItem>
                                            <SelectItem value="bold">Bold</SelectItem>
                                            <SelectItem value="100">Thin (100)</SelectItem>
                                            <SelectItem value="300">Light (300)</SelectItem>
                                            <SelectItem value="500">Medium (500)</SelectItem>
                                            <SelectItem value="700">Bold (700)</SelectItem>
                                            <SelectItem value="900">Black (900)</SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Posture</Label>
                                    <Select value={fontStyle} onValueChange={setFontStyle}>
                                        <SelectTrigger className="h-8 text-xs"><SelectValue placeholder="Upright" /></SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="none">Upright</SelectItem>
                                            <SelectItem value="italic">Italic</SelectItem>
                                            <SelectItem value="oblique">Oblique</SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Color</Label>
                                    <div className="flex gap-2">
                                        <Input type="color" value={color.startsWith('#') ? color : '#000000'} onChange={(e) => setColor(e.target.value)} className="w-8 h-8 p-0 border-none bg-transparent" />
                                        <Input value={color} onChange={(e) => setColor(e.target.value)} placeholder="#000000" className="h-8 text-xs" />
                                    </div>
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Transform</Label>
                                    <Select value={textTransform} onValueChange={setTextTransform}>
                                        <SelectTrigger className="h-8 text-xs"><SelectValue placeholder="None" /></SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="none">None</SelectItem>
                                            <SelectItem value="uppercase">Uppercase</SelectItem>
                                            <SelectItem value="lowercase">Lowercase</SelectItem>
                                            <SelectItem value="capitalize">Capitalize</SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                            </div>
                        </TabsContent>

                        <TabsContent value="spacing" className="space-y-4 pt-4">
                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4">
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Alignment</Label>
                                    <Select value={textAlign} onValueChange={setTextAlign}>
                                        <SelectTrigger className="h-8 text-xs"><SelectValue placeholder="Left" /></SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="none">Default (Left)</SelectItem>
                                            <SelectItem value="start">Start</SelectItem>
                                            <SelectItem value="center">Center</SelectItem>
                                            <SelectItem value="end">End</SelectItem>
                                            <SelectItem value="justify">Justify</SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Line Spacing</Label>
                                    <Input value={lineHeight} onChange={(e) => setLineHeight(e.target.value)} placeholder="e.g., 1.5 or 120%" className="h-8 text-xs" />
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Above Paragraph</Label>
                                    <Input value={marginTop} onChange={(e) => setMarginTop(e.target.value)} placeholder="0in" className="h-8 text-xs" />
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Below Paragraph</Label>
                                    <Input value={marginBottom} onChange={(e) => setMarginBottom(e.target.value)} placeholder="0in" className="h-8 text-xs" />
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Left Indent</Label>
                                    <Input value={marginLeft} onChange={(e) => setMarginLeft(e.target.value)} placeholder="0in" className="h-8 text-xs" />
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">Right Indent</Label>
                                    <Input value={marginRight} onChange={(e) => setMarginRight(e.target.value)} placeholder="0in" className="h-8 text-xs" />
                                </div>
                                <div className="space-y-1.5">
                                    <Label className="text-xs">First Line / Hanging</Label>
                                    <Input value={textIndent} onChange={(e) => setTextIndent(e.target.value)} placeholder="0.5in" className="h-8 text-xs" />
                                </div>
                                <div className="flex items-center space-x-2 pt-4">
                                    <Checkbox id="ctxSpacing" checked={contextualSpacing} onCheckedChange={(v) => setContextualSpacing(!!v)} />
                                    <Label htmlFor="ctxSpacing" className="text-xs font-normal">Don't add spacing between same styles</Label>
                                </div>
                            </div>
                        </TabsContent>

                        <TabsContent value="structure" className="space-y-4 pt-4">
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
                                                        <CommandItem value="none" onSelect={() => {
                                                            setParent('none');
                                                            setIsParentPopoverOpen(false);
                                                        }} className="text-xs py-1">
                                                            <Check className={cn("mr-2 h-3 w-3", parent === 'none' || !parent ? "opacity-100" : "opacity-0")} />
                                                            None
                                                        </CommandItem>
                                                        {Object.entries(existingStyles)
                                                            .filter(([name, s]) => s.family === family && name !== selectedStyleId && (s.displayName || !/^[PT]\d+$/.test(name)))
                                                            .sort((a, b) => (a[1].displayName || a[0]).localeCompare(b[1].displayName || b[0]))
                                                            .map(([n, s]) => (
                                                                <CommandItem key={n} value={n} onSelect={(value: string) => {
                                                                    setParent(value);
                                                                    setIsParentPopoverOpen(false);
                                                                }} className="text-xs py-1">
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
                                                        <CommandItem value="none" onSelect={() => {
                                                            setNext('none');
                                                            setIsNextPopoverOpen(false);
                                                        }} className="text-xs py-1">
                                                            <Check className={cn("mr-2 h-3 w-3", next === 'none' || !next ? "opacity-100" : "opacity-0")} />
                                                            Same style
                                                        </CommandItem>
                                                        {Object.entries(existingStyles)
                                                            .filter(([name, s]) => s.family === StyleFamily.Paragraph && (s.displayName || !/^[PT]\d+$/.test(name)))
                                                            .sort((a, b) => (a[1].displayName || a[0]).localeCompare(b[1].displayName || b[0]))
                                                            .map(([n, s]) => (
                                                                <CommandItem key={n} value={n} onSelect={(value: string) => {
                                                                    setNext(value);
                                                                    setIsNextPopoverOpen(false);
                                                                }} className="text-xs py-1">
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
                        </TabsContent>
                    </Tabs>

                    {/* Preview Section */}
                    <div className="mt-8 pt-6 border-t">
                        <div className="flex items-center gap-2 mb-3">
                            <Eye className="w-4 h-4 text-primary" />
                            <Label className="text-xs font-bold uppercase tracking-[0.1em] text-primary">Live Preview</Label>
                        </div>
                        <div className="border rounded-lg p-6 bg-slate-50 dark:bg-stone-900 overflow-hidden min-h-[120px] flex items-center justify-center">
                            <div className="w-full transition-all duration-200" style={previewStyle}>
                                The quick brown fox jumps over the lazy dog. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
                            </div>
                        </div>
                    </div>
                </div>

                <DialogFooter className="px-6 py-4 border-t bg-muted/30 shrink-0">
                    <Button variant="ghost" onClick={() => onOpenChange(false)} className="h-9">Cancel</Button>
                    <Button onClick={handleSave} className="px-8 h-9 shadow-lg">Save Style</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
