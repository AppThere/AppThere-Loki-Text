import {
    ParagraphNode,
    type SerializedParagraphNode,
    type NodeKey,
    type EditorConfig,
    type LexicalNode,
    type Spread,
} from 'lexical';

export type SerializedParagraphStyleNode = Spread<
    {
        styleName: string | null;
    },
    SerializedParagraphNode
>;

export class ParagraphStyleNode extends ParagraphNode {
    __styleName: string | null;

    constructor(styleName: string | null = null, key?: NodeKey) {
        super(key);
        this.__styleName = styleName;
    }

    static getType(): string {
        return 'paragraph-style';
    }

    static clone(node: ParagraphStyleNode): ParagraphStyleNode {
        return new ParagraphStyleNode(node.__styleName, node.__key);
    }

    getStyleName(): string | null {
        return this.__styleName;
    }

    setStyleName(styleName: string | null): void {
        const writable = this.getWritable();
        writable.__styleName = styleName;
    }

    createDOM(config: EditorConfig): HTMLElement {
        // Let the base ParagraphNode create the element with alignment/dir
        const dom = super.createDOM(config);
        if (this.__styleName) {
            dom.dataset.styleName = this.__styleName;
            const safeClass = this.__styleName.replace(/[^a-zA-Z0-9_-]/g, '_');
            dom.classList.add(`odt-style-${safeClass}`);
        }
        return dom;
    }

    updateDOM(
        prevNode: ParagraphStyleNode,
        dom: HTMLElement,
        config: EditorConfig
    ): boolean {
        const replace = super.updateDOM(prevNode, dom, config);

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

    exportJSON(): SerializedParagraphStyleNode {
        return {
            ...super.exportJSON(),
            styleName: this.__styleName,
            type: 'paragraph-style',
            version: 1,
        };
    }

    static importJSON(serializedNode: SerializedParagraphStyleNode): ParagraphStyleNode {
        const node = $createParagraphStyleNode(serializedNode.styleName || null);
        node.setFormat(serializedNode.format);
        node.setIndent(serializedNode.indent);
        node.setDirection(serializedNode.direction);
        node.setTextFormat(serializedNode.textFormat);
        node.setTextStyle(serializedNode.textStyle);
        return node;
    }

    insertNewAfter(): ParagraphStyleNode {
        const newPara = $createParagraphStyleNode(this.__styleName);
        this.insertAfter(newPara);
        return newPara;
    }
}

export function $createParagraphStyleNode(styleName?: string | null): ParagraphStyleNode {
    return new ParagraphStyleNode(styleName || null);
}

export function $isParagraphStyleNode(node: LexicalNode | null | undefined): node is ParagraphStyleNode {
    return node instanceof ParagraphStyleNode;
}
