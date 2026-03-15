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

export { convertLexicalToTiptap } from './tiptapConverter';
