import { describe, it, expect } from 'vitest';
import { convertLexicalToTiptap, convertTiptapToLexical, TiptapNode } from './lexicalAdapter';
import type { LexicalDocumentData } from '../types/odt';

describe('lexicalAdapter', () => {
    describe('convertTiptapToLexical', () => {
        it('should convert a simple paragraph', () => {
            const tiptapDoc: TiptapNode = {
                type: 'doc',
                content: [
                    {
                        type: 'paragraph',
                        attrs: { styleName: 'Standard', textAlign: 'left', indent: 0 },
                        content: [
                            { type: 'text', text: 'Hello, World!' }
                        ]
                    }
                ]
            };

            const expectedLexical: LexicalDocumentData = {
                root: {
                    type: 'root',
                    version: 1,
                    direction: null,
                    format: '',
                    indent: 0,
                    children: [
                        {
                            type: 'paragraph-style',
                            styleName: 'Standard',
                            format: 'left',
                            indent: 0,
                            version: 1,
                            direction: null,
                            children: [
                                {
                                    type: 'text',
                                    text: 'Hello, World!',
                                    format: 0,
                                    style: '',
                                    mode: 'normal',
                                    detail: 0,
                                    version: 1,
                                    styleName: undefined
                                }
                            ]
                        }
                    ]
                }
            };

            const result = convertTiptapToLexical(tiptapDoc);
            expect(result).toEqual(expectedLexical);
        });

        it('should round-trip complex documents with Links and Tables', () => {
            const tiptapDoc: TiptapNode = {
                type: 'doc',
                content: [
                    {
                        type: 'paragraph',
                        attrs: { styleName: 'Standard', textAlign: undefined, indent: undefined },
                        content: [
                            { type: 'text', text: 'Click ', marks: [{ type: 'bold' }] },
                            { type: 'text', text: 'here', marks: [{ type: 'link', attrs: { href: 'https://example.com', target: '_blank' } }] }
                        ]
                    },
                    {
                        type: 'table',
                        content: [
                            {
                                type: 'tableRow',
                                content: [
                                    {
                                        type: 'tableCell',
                                        attrs: { colspan: 1, rowspan: 1 },
                                        content: [
                                            {
                                                type: 'paragraph',
                                                attrs: { styleName: 'Standard', textAlign: undefined, indent: undefined },
                                                content: [{ type: 'text', text: 'Cell' }]
                                            }
                                        ]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            };

            const lexical = convertTiptapToLexical(tiptapDoc);
            const backToTiptap = convertLexicalToTiptap(lexical);

            // Should accurately round-trip
            expect(backToTiptap).toEqual(tiptapDoc);
        });
    });
});
