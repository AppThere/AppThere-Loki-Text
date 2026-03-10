import {
    HeadingNode,
    type SerializedHeadingNode,
    type HeadingTagType,
} from '@lexical/rich-text';
import {
    type NodeKey,
    type EditorConfig,
    type LexicalNode,
    type Spread,
} from 'lexical';
import { $createParagraphStyleNode, type ParagraphStyleNode } from './ParagraphStyleNode';

export type SerializedHeadingStyleNode = Spread<
    {
        styleName: string | null;
    },
    SerializedHeadingNode
>;

export class HeadingStyleNode extends HeadingNode {
    __styleName: string | null;

    constructor(tag: HeadingTagType, styleName: string | null = null, key?: NodeKey) {
        super(tag, key);
        this.__styleName = styleName;
    }

    static getType(): string {
        return 'heading-style';
    }

    static clone(node: HeadingStyleNode): HeadingStyleNode {
        return new HeadingStyleNode(node.getTag(), node.__styleName, node.__key);
    }

    getStyleName(): string | null {
        return this.__styleName;
    }

    setStyleName(styleName: string | null): void {
        const writable = this.getWritable();
        writable.__styleName = styleName;
    }

    createDOM(config: EditorConfig): HTMLElement {
        const dom = super.createDOM(config);
        if (this.__styleName) {
            dom.dataset.styleName = this.__styleName;
            const safeClass = this.__styleName.replace(/[^a-zA-Z0-9_-]/g, '_');
            dom.classList.add(`odt-style-${safeClass}`);
        }
        return dom;
    }

    updateDOM(
        prevNode: HeadingStyleNode,
        dom: HTMLElement
    ): boolean {
        const replace = super.updateDOM(prevNode, dom);

        if (prevNode.__styleName !== this.__styleName) {
            if (prevNode.__styleName) {
                const prevSafeClass = prevNode.__styleName.replace(/[^a-zA-Z0-9_-]/g, '_');
                dom.classList.remove(`odt-style-${prevSafeClass}`);
            }
            if (this.__styleName) {
                dom.dataset.styleName = this.__styleName;
                const safeClass = this.__styleName.replace(/[^a-zA-Z0-9_-]/g, '_');
                dom.classList.add(`odt-style-${safeClass}`);
            } else {
                delete dom.dataset.styleName;
            }
        }

        return replace;
    }

    exportJSON(): SerializedHeadingStyleNode {
        return {
            ...super.exportJSON(),
            styleName: this.__styleName,
            type: 'heading-style',
            version: 1,
        };
    }

    static importJSON(serializedNode: SerializedHeadingStyleNode): HeadingStyleNode {
        const node = $createHeadingStyleNode(serializedNode.tag, serializedNode.styleName || null);
        node.setFormat(serializedNode.format);
        node.setIndent(serializedNode.indent);
        node.setDirection(serializedNode.direction);
        return node;
    }

    insertNewAfter(): ParagraphStyleNode {
        const newPara = $createParagraphStyleNode(this.__styleName);
        this.insertAfter(newPara);
        return newPara;
    }
}

export function $createHeadingStyleNode(tag: HeadingTagType, styleName?: string | null): HeadingStyleNode {
    return new HeadingStyleNode(tag, styleName || null);
}

export function $isHeadingStyleNode(node: LexicalNode | null | undefined): node is HeadingStyleNode {
    return node instanceof HeadingStyleNode;
}
