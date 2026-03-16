import { useState, useEffect, useMemo } from 'react';
import { StyleDefinition, StyleFamily } from '@/lib/types/odt';

const slugify = (text: string) => {
    return text
        .toString()
        .normalize('NFD')
        .replace(/[\u0300-\u036f]/g, '')
        .trim()
        .replace(/\s+/g, '_')
        .replace(/[^\w-]+/g, '')
        .replace(/--+/g, '_');
};

interface UseStyleDialogProps {
    open: boolean;
    existingStyles: Record<string, StyleDefinition>;
    onSave: (style: StyleDefinition) => void;
    onOpenChange: (open: boolean) => void;
    initialStyleName?: string;
}

export function useStyleDialog({
    open,
    existingStyles,
    onSave,
    onOpenChange,
    initialStyleName,
}: UseStyleDialogProps) {
    const [selectedStyleId, setSelectedStyleId] = useState<string>('__new__');
    const [name, setName] = useState('');
    const [displayName, setDisplayName] = useState('');
    const family = StyleFamily.Paragraph;
    const [parent, setParent] = useState('');
    const [next, setNext] = useState('');

    const [fontSize, setFontSize] = useState('');
    const [fontFamily, setFontFamily] = useState('');
    const [fontWeight, setFontWeight] = useState('');
    const [fontStyle, setFontStyle] = useState('');
    const [color, setColor] = useState('');
    const [textTransform, setTextTransform] = useState('');

    const [textAlign, setTextAlign] = useState('');
    const [lineHeight, setLineHeight] = useState('');
    const [marginTop, setMarginTop] = useState('');
    const [marginBottom, setMarginBottom] = useState('');
    const [marginLeft, setMarginLeft] = useState('');
    const [marginRight, setMarginRight] = useState('');
    const [textIndent, setTextIndent] = useState('');

    const [breakBefore, setBreakBefore] = useState(false);
    const [breakAfter, setBreakAfter] = useState(false);
    const [orphans, setOrphans] = useState('');
    const [widows, setWidows] = useState('');
    const [keepWithNext, setKeepWithNext] = useState(false);
    const [hyphenation, setHyphenation] = useState(false);
    const [contextualSpacing, setContextualSpacing] = useState(false);
    const [outlineLevel, setOutlineLevel] = useState('none');

    const [isStylePopoverOpen, setIsStylePopoverOpen] = useState(false);
    const [isFontPopoverOpen, setIsFontPopoverOpen] = useState(false);
    const [isParentPopoverOpen, setIsParentPopoverOpen] = useState(false);
    const [isNextPopoverOpen, setIsNextPopoverOpen] = useState(false);

    const handleStyleSelect = (styleId: string) => {
        setSelectedStyleId(styleId);
        if (styleId === '__new__') {
            setName(''); setDisplayName(''); setParent(''); setNext('');
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

    useEffect(() => {
        if (selectedStyleId === '__new__' && displayName) {
            setName(slugify(displayName));
        }
    }, [displayName, selectedStyleId]);

    const handleSave = () => {
        let finalName = name;
        if (selectedStyleId === '__new__') {
            let counter = 1;
            while (existingStyles[finalName]) {
                finalName = `${name}_${counter}`;
                counter++;
            }
        }

        const baseAttributes = (selectedStyleId !== '__new__' && existingStyles[selectedStyleId])
            ? existingStyles[selectedStyleId].attributes : {};
        const attributes = { ...baseAttributes };

        if (fontSize) attributes['fo:font-size'] = fontSize; else delete attributes['fo:font-size'];
        if (fontFamily) { attributes['fo:font-family'] = fontFamily; delete attributes['style:font-name']; } else { delete attributes['fo:font-family']; }
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
            fontColour: null,
            backgroundColour: null,
        };
        onSave(style);
        onOpenChange(false);
    };

    const previewStyle = useMemo((): React.CSSProperties => {
        const s: React.CSSProperties = {};
        if (fontFamily) s.fontFamily = fontFamily;
        if (fontSize) s.fontSize = fontSize;
        if (fontWeight === 'bold') s.fontWeight = 'bold';
        else if (fontWeight && fontWeight !== 'none') s.fontWeight = fontWeight as any;
        if (fontStyle === 'italic') s.fontStyle = 'italic';
        else if (fontStyle === 'oblique') s.fontStyle = 'oblique';
        if (color) s.color = color;
        if (textTransform && textTransform !== 'none') s.textTransform = textTransform as any;
        if (textAlign && textAlign !== 'none') s.textAlign = (textAlign === 'start' ? 'left' : textAlign === 'end' ? 'right' : textAlign) as any;
        if (lineHeight) s.lineHeight = lineHeight;
        if (marginTop) s.marginTop = '4px';
        if (marginBottom) s.marginBottom = '4px';
        if (textIndent) s.textIndent = '12px';
        return s;
    }, [fontFamily, fontSize, fontWeight, fontStyle, color, textTransform, textAlign, lineHeight, marginTop, marginBottom, textIndent]);

    return {
        selectedStyleId, family,
        name, setName, displayName, setDisplayName,
        parent, setParent, next, setNext,
        fontSize, setFontSize, fontFamily, setFontFamily,
        fontWeight, setFontWeight, fontStyle, setFontStyle,
        color, setColor, textTransform, setTextTransform,
        textAlign, setTextAlign, lineHeight, setLineHeight,
        marginTop, setMarginTop, marginBottom, setMarginBottom,
        marginLeft, setMarginLeft, marginRight, setMarginRight,
        textIndent, setTextIndent,
        breakBefore, setBreakBefore, breakAfter, setBreakAfter,
        orphans, setOrphans, widows, setWidows,
        keepWithNext, setKeepWithNext, hyphenation, setHyphenation,
        contextualSpacing, setContextualSpacing, outlineLevel, setOutlineLevel,
        isStylePopoverOpen, setIsStylePopoverOpen,
        isFontPopoverOpen, setIsFontPopoverOpen,
        isParentPopoverOpen, setIsParentPopoverOpen,
        isNextPopoverOpen, setIsNextPopoverOpen,
        handleStyleSelect, handleSave, previewStyle,
    };
}
