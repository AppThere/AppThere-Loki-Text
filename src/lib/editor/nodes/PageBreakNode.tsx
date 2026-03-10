import * as React from 'react';
import {
    DecoratorNode,
    type EditorConfig,
    type LexicalNode,
    type SerializedLexicalNode,
} from 'lexical';

export type SerializedPageBreakNode = SerializedLexicalNode;

export class PageBreakNode extends DecoratorNode<React.JSX.Element> {
    static getType(): string {
        return 'page-break';
    }

    static clone(node: PageBreakNode): PageBreakNode {
        return new PageBreakNode(node.__key);
    }

    createDOM(_config: EditorConfig): HTMLElement {
        const div = document.createElement('div');
        div.className = 'page-break';
        div.style.cssText = 'border-top: 2px dashed #ccc; margin: 2rem 0; page-break-after: always;';
        return div;
    }

    updateDOM(): false {
        return false;
    }

    decorate(): React.JSX.Element {
        return <div className="page-break-decorator text-center text-gray-500 py-4">• • • Page Break • • •</div>;
    }

    exportJSON(): SerializedPageBreakNode {
        return {
            type: 'page-break',
            version: 1,
        };
    }

    static importJSON(_serializedNode: SerializedPageBreakNode): PageBreakNode {
        return new PageBreakNode();
    }
}

export function $createPageBreakNode(): PageBreakNode {
    return new PageBreakNode();
}

export function $isPageBreakNode(node: LexicalNode | null | undefined): node is PageBreakNode {
    return node instanceof PageBreakNode;
}
