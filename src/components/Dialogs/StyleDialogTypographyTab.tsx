import { Check, ChevronsUpDown } from 'lucide-react';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
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
import { Button } from '@/components/ui/button';
import { InheritedProps } from '@/lib/hooks/useStyleDialog';

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

interface StyleDialogTypographyTabProps {
    fontFamily: string; setFontFamily: (v: string) => void;
    fontSize: string; setFontSize: (v: string) => void;
    fontWeight: string; setFontWeight: (v: string) => void;
    fontStyle: string; setFontStyle: (v: string) => void;
    color: string; setColor: (v: string) => void;
    textTransform: string; setTextTransform: (v: string) => void;
    isFontPopoverOpen: boolean; setIsFontPopoverOpen: (v: boolean) => void;
    inheritedProps?: InheritedProps;
}

function InheritedBadge({ sourceDisplayName }: { sourceDisplayName: string }) {
    return (
        <span className="text-[10px] text-muted-foreground/60 italic font-normal">
            from {sourceDisplayName}
        </span>
    );
}

export function StyleDialogTypographyTab({
    fontFamily, setFontFamily,
    fontSize, setFontSize,
    fontWeight, setFontWeight,
    fontStyle, setFontStyle,
    color, setColor,
    textTransform, setTextTransform,
    isFontPopoverOpen, setIsFontPopoverOpen,
    inheritedProps = {},
}: StyleDialogTypographyTabProps) {
    const inheritedFontFamily = inheritedProps['fo:font-family'] ?? inheritedProps['style:font-name'];
    const inheritedFontSize = inheritedProps['fo:font-size'];
    const inheritedFontWeight = inheritedProps['fo:font-weight'];
    const inheritedFontStyle = inheritedProps['fo:font-style'];
    const inheritedColor = inheritedProps['fo:color'];
    const inheritedTextTransform = inheritedProps['textTransform'];

    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4">
            <div className="space-y-1.5 flex flex-col">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Font Family</Label>
                    {!fontFamily && inheritedFontFamily && <InheritedBadge sourceDisplayName={inheritedFontFamily.sourceDisplayName} />}
                </div>
                <Popover open={isFontPopoverOpen} onOpenChange={setIsFontPopoverOpen}>
                    <PopoverTrigger asChild>
                        <Button
                            variant="outline"
                            role="combobox"
                            className={cn(
                                "w-full justify-between font-normal h-8 text-xs",
                                !fontFamily && inheritedFontFamily && "text-muted-foreground"
                            )}
                        >
                            {fontFamily || inheritedFontFamily?.value || "Select font..."}
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
                                            onSelect={() => { setFontFamily(font); setIsFontPopoverOpen(false); }}
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
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Font Size</Label>
                    {!fontSize && inheritedFontSize && <InheritedBadge sourceDisplayName={inheritedFontSize.sourceDisplayName} />}
                </div>
                <Input
                    value={fontSize}
                    onChange={(e) => setFontSize(e.target.value)}
                    placeholder={inheritedFontSize?.value || "e.g., 12pt"}
                    className="h-8 text-xs"
                />
            </div>
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Weight</Label>
                    {!fontWeight && inheritedFontWeight && <InheritedBadge sourceDisplayName={inheritedFontWeight.sourceDisplayName} />}
                </div>
                <Select value={fontWeight} onValueChange={setFontWeight}>
                    <SelectTrigger className="h-8 text-xs">
                        <SelectValue placeholder={inheritedFontWeight?.value || "Normal"} />
                    </SelectTrigger>
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
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Posture</Label>
                    {!fontStyle && inheritedFontStyle && <InheritedBadge sourceDisplayName={inheritedFontStyle.sourceDisplayName} />}
                </div>
                <Select value={fontStyle} onValueChange={setFontStyle}>
                    <SelectTrigger className="h-8 text-xs">
                        <SelectValue placeholder={inheritedFontStyle?.value || "Upright"} />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="none">Upright</SelectItem>
                        <SelectItem value="italic">Italic</SelectItem>
                        <SelectItem value="oblique">Oblique</SelectItem>
                    </SelectContent>
                </Select>
            </div>
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Color</Label>
                    {!color && inheritedColor && <InheritedBadge sourceDisplayName={inheritedColor.sourceDisplayName} />}
                </div>
                <div className="flex gap-2">
                    <Input
                        type="color"
                        value={(color || inheritedColor?.value || '#000000').startsWith('#') ? (color || inheritedColor?.value || '#000000') : '#000000'}
                        onChange={(e) => setColor(e.target.value)}
                        className="w-8 h-8 p-0 border-none bg-transparent"
                    />
                    <Input
                        value={color}
                        onChange={(e) => setColor(e.target.value)}
                        placeholder={inheritedColor?.value || "#000000"}
                        className="h-8 text-xs"
                    />
                </div>
            </div>
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Transform</Label>
                    {!textTransform && inheritedTextTransform && <InheritedBadge sourceDisplayName={inheritedTextTransform.sourceDisplayName} />}
                </div>
                <Select value={textTransform} onValueChange={setTextTransform}>
                    <SelectTrigger className="h-8 text-xs">
                        <SelectValue placeholder={inheritedTextTransform?.value || "None"} />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="none">None</SelectItem>
                        <SelectItem value="uppercase">Uppercase</SelectItem>
                        <SelectItem value="lowercase">Lowercase</SelectItem>
                        <SelectItem value="capitalize">Capitalize</SelectItem>
                    </SelectContent>
                </Select>
            </div>
        </div>
    );
}
