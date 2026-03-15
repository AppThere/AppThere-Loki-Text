import type { LexicalNode, LexicalDocumentData } from '../types/odt';
import type { TiptapNode } from './lexicalAdapter';

export function convertLexicalToTiptap(lexicalDoc: LexicalDocumentData): TiptapNode {
    return {
        type: 'doc',
        content: (lexicalDoc.root.children || []).flatMap(c => convertLexicalNodeToTiptap(c)).filter(Boolean) as TiptapNode[]
    };
}

function convertLexicalNodeToTiptap(node: LexicalNode, inheritedMarks: Array<{ type: string, attrs?: any }> = []): TiptapNode | TiptapNode[] | null {
    switch (node.type) {
        case 'paragraph-style':
        case 'paragraph': {
            return {
                type: 'paragraph',
                attrs: {
                    styleName: (node as any).styleName,
                    textAlign: (node as any).format || undefined,
                    indent: (node as any).indent || undefined,
                },
                content: ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, inheritedMarks)).filter(Boolean) as TiptapNode[],
            };
        }
        case 'heading-style':
        case 'heading': {
            const tag = (node as any).tag || 'h1';
            const level = parseInt(tag.replace('h', ''), 10) || 1;
            return {
                type: 'heading',
                attrs: {
                    level,
                    styleName: (node as any).styleName,
                    textAlign: (node as any).format || undefined,
                    indent: (node as any).indent || undefined,
                },
                content: ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, inheritedMarks)).filter(Boolean) as TiptapNode[],
            };
        }
        case 'link': {
            const linkMark = { type: 'link', attrs: { href: (node as any).url, target: (node as any).target || '_blank' } };
            const newMarks = [...inheritedMarks, linkMark];
            return ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, newMarks)).filter(Boolean) as TiptapNode[];
        }
        case 'text': {
            const marks: Array<{ type: string; attrs?: Record<string, any> }> = [...inheritedMarks];
            const format = (node as any).format || 0;

            if (format & 1) marks.push({ type: 'bold' });
            if (format & 2) marks.push({ type: 'italic' });
            if (format & 4) marks.push({ type: 'strike' });
            if (format & 8) marks.push({ type: 'underline' });
            if (format & 32) marks.push({ type: 'subscript' });
            if (format & 64) marks.push({ type: 'superscript' });

            if ((node as any).styleName) {
                marks.push({ type: 'namedSpanStyle', attrs: { styleName: (node as any).styleName } });
            }

            return {
                type: 'text',
                text: (node as any).text || '',
                ...(marks.length > 0 ? { marks } : {})
            };
        }
        case 'page-break': {
            return { type: 'pageBreak' };
        }
        case 'linebreak': {
            return { type: 'hardBreak' };
        }
        case 'image': {
            return {
                type: 'image',
                attrs: {
                    src: (node as any).src,
                    alt: (node as any).altText,
                }
            };
        }
        case 'table': {
            return {
                type: 'table',
                content: ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, inheritedMarks)).filter(Boolean) as TiptapNode[],
            };
        }
        case 'tablerow': {
            return {
                type: 'tableRow',
                content: ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, inheritedMarks)).filter(Boolean) as TiptapNode[],
            };
        }
        case 'tablecell': {
            return {
                type: (node as any).headerState ? 'tableHeader' : 'tableCell',
                attrs: {
                    colspan: (node as any).colSpan || undefined,
                    rowspan: (node as any).rowSpan || undefined,
                },
                content: ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, inheritedMarks)).filter(Boolean) as TiptapNode[],
            };
        }
        case 'list': {
            const listType = (node as any).listType === 'number' ? 'orderedList' : 'bulletList';
            return {
                type: listType,
                content: ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, inheritedMarks)).filter(Boolean) as TiptapNode[],
            };
        }
        case 'listitem': {
            return {
                type: 'listItem',
                content: ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, inheritedMarks)).filter(Boolean) as TiptapNode[],
            };
        }
        case 'quote': {
            return {
                type: 'blockquote',
                content: ((node as any).children || []).flatMap((c: any) => convertLexicalNodeToTiptap(c, inheritedMarks)).filter(Boolean) as TiptapNode[],
            };
        }
        default:
            console.warn('Unknown Lexical node type during conversion:', (node as any).type);
            return null;
    }
}
