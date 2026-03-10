import { useEffect } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { COMMAND_PRIORITY_LOW, KEY_ENTER_COMMAND } from 'lexical';
import { $getSelection, $isRangeSelection } from 'lexical';
import { $isParagraphStyleNode, $createParagraphStyleNode } from '@/lib/editor/nodes/ParagraphStyleNode';
import { $isHeadingStyleNode } from '@/lib/editor/nodes/HeadingStyleNode';
import type { StyleDefinition } from '@/lib/types/odt';

interface NextStylePluginProps {
    styles: Record<string, StyleDefinition>;
}

export function NextStylePlugin({ styles }: NextStylePluginProps) {
    const [editor] = useLexicalComposerContext();

    useEffect(() => {
        const resolveNextStyle = (styleName: string | null): string | null => {
            if (!styleName) return null;
            let currentName: string | null = styleName;
            const visited = new Set<string>();

            while (currentName) {
                if (visited.has(currentName)) break;
                visited.add(currentName);

                const s: StyleDefinition | undefined = styles[currentName];
                if (!s) break;

                if (s.next && s.next !== 'none') {
                    return s.next;
                }
                currentName = s.parent;
            }
            return styleName; // Default to same style
        };

        return editor.registerCommand(
            KEY_ENTER_COMMAND,
            (event: KeyboardEvent | null) => {
                const selection = $getSelection();
                if (!$isRangeSelection(selection) || !selection.isCollapsed()) {
                    return false;
                }

                if (event?.shiftKey) {
                    return false;
                }

                const anchor = selection.anchor;
                const node = anchor.getNode();
                const parent = node.getParentOrThrow();

                // We only apply "Next Style" if we are at the very end of a ParagraphStyleNode or HeadingStyleNode
                if ($isParagraphStyleNode(parent) || $isHeadingStyleNode(parent)) {
                    const styleName = (parent as any).getStyleName();
                    const nextStyle = resolveNextStyle(styleName);

                    // Check if at the end
                    const isAtEnd = anchor.offset === (node.getTextContentSize ? node.getTextContentSize() : 0);
                    const isLastChild = node.getNextSibling() === null;

                    if (isAtEnd && isLastChild) {
                        event?.preventDefault();
                        editor.update(() => {
                            const newPara = $createParagraphStyleNode(nextStyle);
                            parent.insertAfter(newPara);
                            newPara.select();
                        });
                        return true;
                    }
                }

                return false;
            },
            COMMAND_PRIORITY_LOW
        );
    }, [editor, styles]);

    return null;
}
