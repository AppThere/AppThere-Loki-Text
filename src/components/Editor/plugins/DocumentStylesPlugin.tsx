import { useEffect } from 'react';
import type { StyleDefinition } from '@/lib/types/odt';
import type { Colour } from '@/lib/vector/types';

/** Convert a typed Colour to a CSS string suitable for use as a color value. */
function colourToCss(c: Colour): string {
    if (c.type === 'Rgb') {
        const r = Math.round(c.r * 255);
        const g = Math.round(c.g * 255);
        const b = Math.round(c.b * 255);
        return c.a >= 1.0
            ? `rgb(${r}, ${g}, ${b})`
            : `rgba(${r}, ${g}, ${b}, ${c.a})`;
    }
    if (c.type === 'Cmyk') {
        const r = Math.round((1 - c.c) * (1 - c.k) * 255);
        const g = Math.round((1 - c.m) * (1 - c.k) * 255);
        const b = Math.round((1 - c.y) * (1 - c.k) * 255);
        return `rgb(${r}, ${g}, ${b})`;
    }
    if (c.type === 'Lab') {
        // Naive approximation; accurate rendering requires ICC profile conversion.
        const r = Math.round((c.l / 100) * 255);
        const g = Math.round(((c.a + 128) / 255) * 255);
        const b = Math.round(((c.b + 128) / 255) * 255);
        return `rgb(${r}, ${g}, ${b})`;
    }
    if (c.type === 'Spot') {
        return colourToCss(c.cmyk_fallback);
    }
    return '#000000';
}

interface DocumentStylesPluginProps {
    styles: Record<string, StyleDefinition>;
}

export function DocumentStylesPlugin({ styles }: DocumentStylesPluginProps) {
    useEffect(() => {
        let styleTag = document.getElementById('loki-document-styles');
        if (!styleTag) {
            styleTag = document.createElement('style');
            styleTag.id = 'loki-document-styles';
            document.head.appendChild(styleTag);
        }

        let css = '';

        // Helper to resolve inherited attributes
        const resolveStyle = (styleName: string, visited = new Set<string>()): { attributes: Record<string, string>, textTransform: string | null, fontColour: import('@/lib/vector/types').Colour | null, backgroundColour: import('@/lib/vector/types').Colour | null } => {
            if (visited.has(styleName)) return { attributes: {}, textTransform: null, fontColour: null, backgroundColour: null };
            visited.add(styleName);

            const def = styles[styleName];
            if (!def) return { attributes: {}, textTransform: null, fontColour: null, backgroundColour: null };

            let parentData: { attributes: Record<string, string>, textTransform: string | null, fontColour: import('@/lib/vector/types').Colour | null, backgroundColour: import('@/lib/vector/types').Colour | null } = { attributes: {}, textTransform: null, fontColour: null, backgroundColour: null };
            if (def.parent && styles[def.parent]) {
                parentData = resolveStyle(def.parent, visited);
            }

            return {
                attributes: { ...parentData.attributes, ...def.attributes },
                textTransform: def.textTransform || parentData.textTransform,
                fontColour: def.fontColour ?? parentData.fontColour,
                backgroundColour: def.backgroundColour ?? parentData.backgroundColour,
            };
        };

        const makeResponsive = (value: string) => {
            if (!value || value === '0' || value === '0in' || value === '0pt') return value;
            // If it's a large absolute value, wrap it in a min() to prevent it from squeezing text on mobile
            // We use 30vw as a conservative cap for margins to ensure text has at least 40% width (30% left + 30% right)
            return `min(${value}, 30vw)`;
        };

        for (const [styleName] of Object.entries(styles)) {
            const resolved = resolveStyle(styleName);
            const attrs = resolved.attributes;
            if (!attrs && !resolved.textTransform) continue;

            const safeClass = styleName.replace(/[^a-zA-Z0-9_-]/g, '_');
            let rule = `.odt-style-${safeClass} { `;

            // Map common ODT styles to CSS
            const cleanFont = (val?: string) => {
                if (!val) return null;
                return val.replace(/['"]/g, '').trim();
            };

            const fontFamily = cleanFont(attrs['fo:font-family'] || attrs['style:font-name']);
            if (fontFamily) {
                rule += `font-family: "${fontFamily}", serif; `;
            }
            if (attrs['fo:font-size']) {
                rule += `font-size: ${attrs['fo:font-size']}; `;
            }
            if (attrs['fo:font-weight']) {
                rule += `font-weight: ${attrs['fo:font-weight']}; `;
            }
            if (attrs['fo:font-style']) {
                rule += `font-style: ${attrs['fo:font-style']}; `;
            }
            if (resolved.fontColour) {
                rule += `color: ${colourToCss(resolved.fontColour)}; `;
            } else if (attrs['fo:color']) {
                rule += `color: ${attrs['fo:color']}; `;
            }
            if (attrs['fo:text-align']) {
                const align = attrs['fo:text-align'];
                if (align !== 'start') {
                    rule += `text-align: ${align === 'end' ? 'right' : align === 'center' ? 'center' : align === 'justify' ? 'justify' : 'left'}; `;
                }
            }
            if (attrs['fo:line-height']) {
                rule += `line-height: ${attrs['fo:line-height']}; `;
            }
            if (attrs['fo:margin-top']) {
                rule += `margin-top: ${attrs['fo:margin-top']}; `;
            }
            if (attrs['fo:margin-bottom']) {
                rule += `margin-bottom: ${attrs['fo:margin-bottom']}; `;
            }
            if (attrs['fo:margin-left']) {
                rule += `margin-left: ${makeResponsive(attrs['fo:margin-left'])}; `;
            }
            if (attrs['fo:margin-right']) {
                rule += `margin-right: ${makeResponsive(attrs['fo:margin-right'])}; `;
            }
            if (attrs['fo:text-indent']) {
                rule += `text-indent: ${attrs['fo:text-indent']}; `;
            }

            // Handle page break
            if (attrs['style:break-before'] === 'page' || attrs['fo:break-before'] === 'page') {
                rule += `break-before: page; page-break-before: always; `;
            }
            if (attrs['style:break-after'] === 'page' || attrs['fo:break-after'] === 'page') {
                rule += `break-after: page; page-break-after: always; `;
            }

            if (attrs['fo:orphans']) {
                rule += `orphans: ${attrs['fo:orphans']}; `;
            }
            if (attrs['fo:widows']) {
                rule += `widows: ${attrs['fo:widows']}; `;
            }
            if (attrs['fo:keep-with-next'] === 'always') {
                rule += `break-after: avoid; page-break-after: avoid; `;
            }

            if (resolved.textTransform === 'uppercase') {
                rule += `text-transform: uppercase; `;
            } else if (resolved.textTransform) {
                rule += `text-transform: ${resolved.textTransform}; `;
            }

            rule += `}\n`;

            // Contextual spacing: if the same style follows, remove the top margin
            if (attrs['style:contextual-spacing'] === 'true') {
                css += `.odt-style-${safeClass} + .odt-style-${safeClass} { margin-top: 0 !important; }\n`;
            }

            css += rule;
        }

        styleTag.innerHTML = css;

        return () => {
            // Cleanup on unmount
            const tag = document.getElementById('loki-document-styles');
            if (tag) tag.innerHTML = '';
        };
    }, [styles]);

    return null;
}
