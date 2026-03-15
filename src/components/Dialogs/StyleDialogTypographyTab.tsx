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
}

export function StyleDialogTypographyTab({
    fontFamily, setFontFamily,
    fontSize, setFontSize,
    fontWeight, setFontWeight,
    fontStyle, setFontStyle,
    color, setColor,
    textTransform, setTextTransform,
    isFontPopoverOpen, setIsFontPopoverOpen,
}: StyleDialogTypographyTabProps) {
    return (
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
    );
}
