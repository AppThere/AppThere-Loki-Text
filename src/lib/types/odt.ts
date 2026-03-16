// Must match Rust structs exactly for JSON serialization

import type { Colour } from '@/lib/vector/types';
export type { Colour };

export enum StyleFamily {
    Paragraph = "Paragraph",
    Text = "Text",
}

export interface StyleDefinition {
    name: string;
    family: StyleFamily;
    parent: string | null;
    next: string | null;  // Key feature: auto-apply next style
    displayName: string | null;
    attributes: Record<string, string>;
    textTransform: string | null;
    outlineLevel: number | null;
    autocomplete: boolean | null;
    /** Typed font colour, populated from fo:color / loki:colour. Null when not set. */
    fontColour: Colour | null;
    /** Typed background colour, populated from fo:background-color. Null when not set. */
    backgroundColour: Colour | null;
}

export interface Metadata {
    identifier: string | null;
    title: string | null;
    language: string | null;
    description: string | null;
    subject: string | null;
    creator: string | null;
    creationDate: string | null;
    generator: string | null;
}

// Lexical node representation (replaces TiptapNode)
export interface LexicalDocumentData {
    root: {
        children: LexicalNode[];
        direction: "ltr" | "rtl" | null;
        format: string;
        indent: number;
        type: "root";
        version: number;
    };
}

export type LexicalNode =
    | ParagraphNode
    | HeadingNode
    | TextNode
    | ListNode
    | ListItemNode
    | QuoteNode
    | ImageNode
    | LinkNode
    | TableNode
    | TableRowNode
    | TableCellNode
    | PageBreakNode
    | LineBreakNode;

export interface ParagraphNode {
    type: "paragraph" | "paragraph-style";
    children: LexicalNode[];
    format?: string;
    indent?: number;
    styleName?: string;  // ODT paragraph style
    textAlign?: "left" | "center" | "right" | "justify";
    version?: number;
    direction?: "ltr" | "rtl" | null;
}

export interface HeadingNode {
    type: "heading" | "heading-style";
    tag: "h1" | "h2" | "h3" | "h4" | "h5" | "h6";
    children: LexicalNode[];
    styleName?: string;
    format?: string;
    indent?: number;
}

export interface TextNode {
    type: "text";
    text: string;
    format?: number;  // Lexical format bitmask (bold, italic, etc.)
    styleName?: string;  // ODT character style
    style?: string;
    mode?: string;
    detail?: number;
    version?: number;
}

export interface ListNode {
    type: "list";
    listType: "bullet" | "number";
    children: ListItemNode[];
}

export interface ListItemNode {
    type: "listitem";
    children: LexicalNode[];
}

export interface QuoteNode {
    type: "quote";
    children: LexicalNode[];
}

export interface ImageNode {
    type: "image";
    src: string;
    altText: string;
}

export interface LinkNode {
    type: "link";
    url: string;
    target?: string;
    rel?: string;
    title?: string | null;
    children: LexicalNode[];
}

export interface TableNode {
    type: "table";
    children: LexicalNode[];
}

export interface TableRowNode {
    type: "tablerow";
    children: LexicalNode[];
}

export interface TableCellNode {
    type: "tablecell";
    colSpan?: number;
    rowSpan?: number;
    headerState?: number;
    children: LexicalNode[];
}

export interface PageBreakNode {
    type: "page-break";
    version: number;
}

export interface LineBreakNode {
    type: "linebreak";
    version: number;
}

export interface DocumentResponse {
    content: LexicalDocumentData;
    styles: Record<string, StyleDefinition>;
    metadata: Metadata;
}
