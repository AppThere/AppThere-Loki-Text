import { useEffect, useRef } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { $getSelection, $isRangeSelection, $isElementNode } from 'lexical';
import { $isParagraphStyleNode, $createParagraphStyleNode } from '@/lib/editor/nodes/ParagraphStyleNode';
import { $isHeadingStyleNode, $createHeadingStyleNode } from '@/lib/editor/nodes/HeadingStyleNode';
import type { StyleDefinition } from '@/lib/types/odt';

interface StylePluginProps {
    currentStyle?: string;
    onStyleChange?: (styleName: string) => void;
    styles: Record<string, StyleDefinition>;
}

export function StylePlugin({ currentStyle, onStyleChange, styles }: StylePluginProps) {
    const [editor] = useLexicalComposerContext();
    const lastEmittedStyle = useRef<string | undefined>(currentStyle);

    // Sync external currentStyle prop into editor if it changes AND it's not simply an echo of what we just emitted
    useEffect(() => {
        if (!currentStyle || currentStyle === lastEmittedStyle.current) return;

        editor.update(() => {
            const selection = $getSelection();
            if ($isRangeSelection(selection)) {
                const anchor = selection.anchor.getNode();
                const currentNode = anchor.getParent();
                // Start from anchor itself: on an empty paragraph the selection anchor IS
                // the ParagraphStyleNode (element-type selection), so getParent() would
                // skip past it to the root and the traversal would find nothing.
                let styledParent = anchor;

                // Find the paragraph/heading style node
                while (styledParent && !$isParagraphStyleNode(styledParent) && !$isHeadingStyleNode(styledParent)) {
                    styledParent = styledParent.getParent();
                }

                // If the targeted style implies a specific outline level, we might need a heading
                const targetStyleDef = styles[currentStyle];
                const isTargetHeading = targetStyleDef?.outlineLevel || targetStyleDef?.name.toLowerCase().includes('heading');

                if ($isParagraphStyleNode(styledParent) || $isHeadingStyleNode(styledParent)) {
                    // Update the style name, but also we might need to swap node type
                    if (isTargetHeading && !$isHeadingStyleNode(styledParent)) {
                        const level = targetStyleDef?.outlineLevel ? Math.min(Math.max(targetStyleDef.outlineLevel, 1), 6) : 1;
                        const headingNode = $createHeadingStyleNode(`h${level}` as any, currentStyle);
                        headingNode.append(...styledParent.getChildren());
                        styledParent.replace(headingNode);
                        headingNode.select();
                    } else if (!isTargetHeading && !$isParagraphStyleNode(styledParent)) {
                        const paragraphNode = $createParagraphStyleNode(currentStyle);
                        paragraphNode.append(...styledParent.getChildren());
                        styledParent.replace(paragraphNode);
                        paragraphNode.select();
                    } else {
                        // Safe to just update the name
                        styledParent.setStyleName(currentStyle);
                    }
                } else if (currentNode && $isElementNode(currentNode)) {
                    const styledNode = isTargetHeading
                        ? $createHeadingStyleNode('h1' as any, currentStyle)
                        : $createParagraphStyleNode(currentStyle);
                    styledNode.append(...currentNode.getChildren());
                    currentNode.replace(styledNode);
                    styledNode.select();
                }
            }
        });

    }, [editor, currentStyle, styles]);

    // Sync editor selection style outward to UI
    useEffect(() => {
        return editor.registerUpdateListener(({ editorState }) => {
            editorState.read(() => {
                const selection = $getSelection();
                if ($isRangeSelection(selection) && onStyleChange) {
                    const anchor = selection.anchor.getNode();
                    // Start from anchor itself for the same reason as above: empty
                    // paragraphs have an element-type anchor pointing to the paragraph.
                    let currentNode = anchor;

                    while (currentNode && !$isParagraphStyleNode(currentNode) && !$isHeadingStyleNode(currentNode)) {
                        currentNode = currentNode.getParent();
                    }

                    if ($isParagraphStyleNode(currentNode) || $isHeadingStyleNode(currentNode)) {
                        const style = currentNode.getStyleName();
                        if (style && style !== lastEmittedStyle.current) {
                            lastEmittedStyle.current = style;
                            onStyleChange(style);
                        }
                    }
                }
            });
        });
    }, [editor, onStyleChange]);

    return null;
}
