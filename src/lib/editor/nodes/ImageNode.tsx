import * as React from 'react';
import {
    DecoratorNode,
    type NodeKey,
    type EditorConfig,
    type LexicalNode,
    type SerializedLexicalNode,
    type Spread,
} from 'lexical';

export type SerializedImageNode = Spread<
    {
        src: string;
        altText: string;
    },
    SerializedLexicalNode
>;

export class ImageNode extends DecoratorNode<React.JSX.Element> {
    __src: string;
    __altText: string;

    constructor(src: string, altText: string, key?: NodeKey) {
        super(key);
        this.__src = src;
        this.__altText = altText;
    }

    static getType(): string {
        return 'image';
    }

    static clone(node: ImageNode): ImageNode {
        return new ImageNode(node.__src, node.__altText, node.__key);
    }

    createDOM(_config: EditorConfig): HTMLElement {
        const span = document.createElement('span');
        span.className = 'editor-image-wrapper';
        return span;
    }

    updateDOM(): false {
        return false;
    }

    decorate(): React.JSX.Element {
        return <img src={this.__src} alt={this.__altText} className="max-w-full h-auto" />;
    }

    exportJSON(): SerializedImageNode {
        return {
            type: 'image',
            version: 1,
            src: this.__src,
            altText: this.__altText,
        };
    }

    static importJSON(serializedNode: SerializedImageNode): ImageNode {
        return new ImageNode(serializedNode.src, serializedNode.altText);
    }
}

export function $createImageNode(src: string, altText: string): ImageNode {
    return new ImageNode(src, altText);
}

export function $isImageNode(node: LexicalNode | null | undefined): node is ImageNode {
    return node instanceof ImageNode;
}
