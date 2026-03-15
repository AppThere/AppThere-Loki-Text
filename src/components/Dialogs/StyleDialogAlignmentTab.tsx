import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Checkbox } from '@/components/ui/checkbox';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';

interface StyleDialogAlignmentTabProps {
    textAlign: string; setTextAlign: (v: string) => void;
    lineHeight: string; setLineHeight: (v: string) => void;
    marginTop: string; setMarginTop: (v: string) => void;
    marginBottom: string; setMarginBottom: (v: string) => void;
    marginLeft: string; setMarginLeft: (v: string) => void;
    marginRight: string; setMarginRight: (v: string) => void;
    textIndent: string; setTextIndent: (v: string) => void;
    contextualSpacing: boolean; setContextualSpacing: (v: boolean) => void;
}

export function StyleDialogAlignmentTab({
    textAlign, setTextAlign,
    lineHeight, setLineHeight,
    marginTop, setMarginTop,
    marginBottom, setMarginBottom,
    marginLeft, setMarginLeft,
    marginRight, setMarginRight,
    textIndent, setTextIndent,
    contextualSpacing, setContextualSpacing,
}: StyleDialogAlignmentTabProps) {
    return (
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
    );
}
