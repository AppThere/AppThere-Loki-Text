import { Extension } from '@tiptap/core';

export const NextParagraphStyle = Extension.create({
    name: 'nextParagraphStyle',

    addKeyboardShortcuts() {
        return {
            'Enter': ({ editor }) => {
                const { state } = editor.view;
                const { selection } = state;
                const { $from, empty } = selection;

                // Get the current block node
                const currentNode = $from.node($from.depth);
                const currentStyle = currentNode.attrs.styleName;

                if (!currentStyle) {
                    return false; // Use default behavior
                }

                // Check if we are at end of block
                const isAtEnd = empty && $from.parentOffset === currentNode.content.size;

                if (!isAtEnd) {
                    return false; // Middle of block: split normally (persisting style)
                }

                // Access styleRegistry from window
                const getNextStyle = (window as any).__getNextStyle;
                if (!getNextStyle) {
                    return false;
                }

                const nextStyle = getNextStyle(currentStyle);

                if (nextStyle && nextStyle !== currentStyle) {
                    // We are at the end and have a next style
                    // Manually insert the new block to avoid double-actions or retaining attrs
                    return editor
                        .chain()
                        .insertContentAt(selection.to, {
                            type: 'paragraph',
                            attrs: { styleName: nextStyle }
                        })
                        .scrollIntoView()
                        .run();
                }

                return false; // Use default behavior
            }
        };
    }
});
