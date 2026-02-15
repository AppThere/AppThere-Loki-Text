import { Extension } from '@tiptap/core';

export const NextParagraphStyle = Extension.create({
    name: 'nextParagraphStyle',

    addKeyboardShortcuts() {
        return {
            'Enter': ({ editor }) => {
                const { state } = editor.view;
                const { $from } = state.selection;

                // Get the current block node
                const currentNode = $from.node($from.depth);
                const currentStyle = currentNode.attrs.styleName;

                if (!currentStyle) {
                    return false; // Use default behavior
                }

                // Access styleRegistry from window (we'll set this in Editor.svelte)
                const getNextStyle = (window as any).__getNextStyle;
                if (!getNextStyle) {
                    return false;
                }

                const nextStyle = getNextStyle(currentStyle);

                if (nextStyle && nextStyle !== currentStyle) {
                    // Use chain API to split block and set attributes
                    return editor
                        .chain()
                        .splitBlock()
                        .updateAttributes('paragraph', { styleName: nextStyle })
                        .updateAttributes('heading', { styleName: nextStyle })
                        .run();
                }

                return false; // Use default behavior
            }
        };
    }
});
