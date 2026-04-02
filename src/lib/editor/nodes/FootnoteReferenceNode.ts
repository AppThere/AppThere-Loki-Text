// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import * as React from 'react';
import {
    DecoratorNode,
    type EditorConfig,
    type LexicalNode,
    type SerializedLexicalNode,
    type Spread,
} from 'lexical';
import { FootnoteAnchor } from './FootnoteAnchor';
import './FootnoteReferenceNode.css';

export type SerializedFootnoteReferenceNode = Spread<
    { id: string },
    SerializedLexicalNode
>;

/**
 * Inline Lexical node representing a footnote reference anchor in the body.
 * The display number is derived at render time from document order.
 */
export class FootnoteReferenceNode extends DecoratorNode<React.JSX.Element> {
    __id: string;

    static getType(): string {
        return 'footnote-ref';
    }

    static clone(node: FootnoteReferenceNode): FootnoteReferenceNode {
        return new FootnoteReferenceNode(node.__id, node.__key);
    }

    constructor(id: string, key?: string) {
        super(key);
        this.__id = id;
    }

    getFootnoteId(): string {
        return this.__id;
    }

    createDOM(_config: EditorConfig): HTMLElement {
        const span = document.createElement('span');
        span.className = 'footnote-anchor';
        return span;
    }

    updateDOM(): false {
        return false;
    }

    isInline(): true {
        return true;
    }

    decorate(): React.JSX.Element {
        return React.createElement(FootnoteAnchor, { footnoteId: this.__id });
    }

    exportJSON(): SerializedFootnoteReferenceNode {
        return {
            type: 'footnote-ref',
            version: 1,
            id: this.__id,
        };
    }

    static importJSON(serialized: SerializedFootnoteReferenceNode): FootnoteReferenceNode {
        return new FootnoteReferenceNode(serialized.id);
    }
}

export function $createFootnoteReferenceNode(id: string): FootnoteReferenceNode {
    return new FootnoteReferenceNode(id);
}

export function $isFootnoteReferenceNode(
    node: LexicalNode | null | undefined,
): node is FootnoteReferenceNode {
    return node instanceof FootnoteReferenceNode;
}
