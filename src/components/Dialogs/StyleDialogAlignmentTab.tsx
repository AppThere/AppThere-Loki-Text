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
import { InheritedProps } from '@/lib/hooks/useStyleDialog';

interface StyleDialogAlignmentTabProps {
    textAlign: string; setTextAlign: (v: string) => void;
    lineHeight: string; setLineHeight: (v: string) => void;
    marginTop: string; setMarginTop: (v: string) => void;
    marginBottom: string; setMarginBottom: (v: string) => void;
    marginLeft: string; setMarginLeft: (v: string) => void;
    marginRight: string; setMarginRight: (v: string) => void;
    textIndent: string; setTextIndent: (v: string) => void;
    contextualSpacing: boolean; setContextualSpacing: (v: boolean) => void;
    inheritedProps?: InheritedProps;
}

function InheritedBadge({ sourceDisplayName }: { sourceDisplayName: string }) {
    return (
        <span className="text-[10px] text-muted-foreground/60 italic font-normal">
            from {sourceDisplayName}
        </span>
    );
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
    inheritedProps = {},
}: StyleDialogAlignmentTabProps) {
    const inheritedTextAlign = inheritedProps['fo:text-align'];
    const inheritedLineHeight = inheritedProps['fo:line-height'];
    const inheritedMarginTop = inheritedProps['fo:margin-top'];
    const inheritedMarginBottom = inheritedProps['fo:margin-bottom'];
    const inheritedMarginLeft = inheritedProps['fo:margin-left'];
    const inheritedMarginRight = inheritedProps['fo:margin-right'];
    const inheritedTextIndent = inheritedProps['fo:text-indent'];
    const inheritedContextualSpacing = inheritedProps['style:contextual-spacing'];

    return (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4">
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Alignment</Label>
                    {!textAlign && inheritedTextAlign && <InheritedBadge sourceDisplayName={inheritedTextAlign.sourceDisplayName} />}
                </div>
                <Select value={textAlign} onValueChange={setTextAlign}>
                    <SelectTrigger className="h-8 text-xs">
                        <SelectValue placeholder={inheritedTextAlign?.value || "Left"} />
                    </SelectTrigger>
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
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Line Spacing</Label>
                    {!lineHeight && inheritedLineHeight && <InheritedBadge sourceDisplayName={inheritedLineHeight.sourceDisplayName} />}
                </div>
                <Input
                    value={lineHeight}
                    onChange={(e) => setLineHeight(e.target.value)}
                    placeholder={inheritedLineHeight?.value || "e.g., 1.5 or 120%"}
                    className="h-8 text-xs"
                />
            </div>
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Above Paragraph</Label>
                    {!marginTop && inheritedMarginTop && <InheritedBadge sourceDisplayName={inheritedMarginTop.sourceDisplayName} />}
                </div>
                <Input
                    value={marginTop}
                    onChange={(e) => setMarginTop(e.target.value)}
                    placeholder={inheritedMarginTop?.value || "0in"}
                    className="h-8 text-xs"
                />
            </div>
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Below Paragraph</Label>
                    {!marginBottom && inheritedMarginBottom && <InheritedBadge sourceDisplayName={inheritedMarginBottom.sourceDisplayName} />}
                </div>
                <Input
                    value={marginBottom}
                    onChange={(e) => setMarginBottom(e.target.value)}
                    placeholder={inheritedMarginBottom?.value || "0in"}
                    className="h-8 text-xs"
                />
            </div>
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Left Indent</Label>
                    {!marginLeft && inheritedMarginLeft && <InheritedBadge sourceDisplayName={inheritedMarginLeft.sourceDisplayName} />}
                </div>
                <Input
                    value={marginLeft}
                    onChange={(e) => setMarginLeft(e.target.value)}
                    placeholder={inheritedMarginLeft?.value || "0in"}
                    className="h-8 text-xs"
                />
            </div>
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">Right Indent</Label>
                    {!marginRight && inheritedMarginRight && <InheritedBadge sourceDisplayName={inheritedMarginRight.sourceDisplayName} />}
                </div>
                <Input
                    value={marginRight}
                    onChange={(e) => setMarginRight(e.target.value)}
                    placeholder={inheritedMarginRight?.value || "0in"}
                    className="h-8 text-xs"
                />
            </div>
            <div className="space-y-1.5">
                <div className="flex items-center justify-between">
                    <Label className="text-xs">First Line / Hanging</Label>
                    {!textIndent && inheritedTextIndent && <InheritedBadge sourceDisplayName={inheritedTextIndent.sourceDisplayName} />}
                </div>
                <Input
                    value={textIndent}
                    onChange={(e) => setTextIndent(e.target.value)}
                    placeholder={inheritedTextIndent?.value || "0.5in"}
                    className="h-8 text-xs"
                />
            </div>
            <div className="flex items-center space-x-2 pt-4">
                <Checkbox id="ctxSpacing" checked={contextualSpacing} onCheckedChange={(v) => setContextualSpacing(!!v)} />
                <Label htmlFor="ctxSpacing" className="text-xs font-normal">Don't add spacing between same styles</Label>
                {!contextualSpacing && inheritedContextualSpacing?.value === 'true' && (
                    <InheritedBadge sourceDisplayName={inheritedContextualSpacing.sourceDisplayName} />
                )}
            </div>
        </div>
    );
}
