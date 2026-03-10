import type { LexicalNode, LexicalDocumentData } from '../types/odt';

export interface TiptapNode {
    type: string;
    text?: string;
    attrs?: Record<string, any>;
    marks?: Array<{ type: string; attrs?: Record<string, any> }>;
    content?: TiptapNode[];
}

export interface TiptapResponse {
    content: TiptapNode;
    styles: Record<string, any>;
    metadata: any;
}

export function convertTiptapToLexical(tiptapDoc: TiptapNode): LexicalDocumentData {
    if (tiptapDoc.type !== 'doc') {
        throw new Error('Expected root TiptapNode to be of type "doc"');
    }

    const rootChildren = (tiptapDoc.content || []).map(convertNode).filter(Boolean) as LexicalNode[];

    // Ensure root is never completely empty
    if (rootChildren.length === 0) {
        rootChildren.push({
            type: 'paragraph-style',
            styleName: 'Standard',
            children: [],
            direction: null,
            format: '',
            indent: 0,
            version: 1,
        } as any);
    }

    return {
        root: {
            children: rootChildren,
            direction: null,
            format: '',
            indent: 0,
            type: 'root',
            version: 1,
        }
    };
}

function convertNode(node: TiptapNode): LexicalNode | null {
    switch (node.type) {
        case 'paragraph': {
            const styleName = node.attrs?.styleName || 'Standard';
            return {
                type: 'paragraph-style',
                styleName,
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: node.attrs?.textAlign || '',
                indent: node.attrs?.indent || 0,
                version: 1,
            } as any;
        }
        case 'heading': {
            const level = node.attrs?.level || 1;
            const tag = `h${Math.min(Math.max(level, 1), 6)}` as any;
            return {
                type: 'heading-style',
                tag,
                styleName: node.attrs?.styleName,
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: node.attrs?.textAlign || '',
                indent: node.attrs?.indent || 0,
                version: 1,
            } as any;
        }
        case 'text': {
            let format = 0;
            let styleName = '';
            let linkAttrs: { href: string; target?: string } | null = null;
            if (node.marks) {
                for (const mark of node.marks) {
                    switch (mark.type) {
                        case 'bold': format |= 1; break;
                        case 'italic': format |= 2; break;
                        case 'strike': format |= 4; break;
                        case 'underline': format |= 8; break;
                        case 'subscript': format |= 32; break;
                        case 'superscript': format |= 64; break;
                        case 'namedSpanStyle':
                            styleName = mark.attrs?.styleName || '';
                            break;
                        case 'link':
                            linkAttrs = {
                                href: mark.attrs?.href || '',
                                target: mark.attrs?.target || '_blank'
                            };
                            break;
                    }
                }
            }

            const lexicalTextNode = {
                type: 'text',
                text: node.text || '',
                format,
                style: '',
                mode: 'normal',
                detail: 0,
                styleName: styleName || undefined,
                version: 1,
            } as any;

            if (linkAttrs) {
                return {
                    type: 'link',
                    url: linkAttrs.href,
                    target: linkAttrs.target,
                    rel: 'noopener noreferrer',
                    children: [lexicalTextNode],
                    direction: null,
                    format: '',
                    indent: 0,
                    version: 1,
                } as any;
            }

            return lexicalTextNode;
        }
        case 'pageBreak': {
            return {
                type: 'page-break',
                version: 1,
            } as any;
        }
        case 'hardBreak': {
            return {
                type: 'linebreak',
                version: 1,
            } as any;
        }
        case 'image': {
            return {
                type: 'image',
                src: node.attrs?.src || '',
                altText: node.attrs?.alt || '',
                version: 1,
            } as any;
        }
        case 'table': {
            return {
                type: 'table',
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: '',
                indent: 0,
                version: 1,
            } as any;
        }
        case 'tableRow': {
            return {
                type: 'tablerow',
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: '',
                indent: 0,
                version: 1,
            } as any;
        }
        case 'tableCell':
        case 'tableHeader': {
            return {
                type: 'tablecell',
                colSpan: node.attrs?.colspan || 1,
                rowSpan: node.attrs?.rowspan || 1,
                headerState: node.type === 'tableHeader' ? 1 : 0,
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: '',
                indent: 0,
                version: 1,
            } as any;
        }
        case 'bulletList': {
            return {
                type: 'list',
                listType: 'bullet',
                start: 1,
                tag: 'ul',
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: '',
                indent: 0,
                version: 1,
            } as any;
        }
        case 'orderedList': {
            return {
                type: 'list',
                listType: 'number',
                start: 1,
                tag: 'ol',
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: '',
                indent: 0,
                version: 1,
            } as any;
        }
        case 'listItem': {
            return {
                type: 'listitem',
                value: 1,
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: '',
                indent: 0,
                version: 1,
            } as any;
        }
        case 'blockquote': {
            return {
                type: 'quote',
                children: (node.content || []).map(convertNode).filter(Boolean) as LexicalNode[],
                direction: null,
                format: '',
                indent: 0,
                version: 1,
            } as any;
        }
        default:
            console.warn('Unknown TipTap node type during conversion:', node.type);
            return null;
    }
}

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
