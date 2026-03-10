import { useEffect } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';

/**
 * Lexical tries to scroll the cursor into view whenever it gains focus.
 * On Android, this triggers a window-level pan.
 * This plugin monkey-patches scrollIntoView on the editor root to be a no-op.
 */
export function ScrollPlugin() {
    const [editor] = useLexicalComposerContext();

    useEffect(() => {
        const rootElement = editor.getRootElement();
        if (rootElement) {
            const originalScrollIntoView = rootElement.scrollIntoView;
            // @ts-ignore - overriding built-in method
            rootElement.scrollIntoView = function () {
                // Do nothing
                console.log('Lexical tried to scrollIntoView - BLOCKED');
            };

            return () => {
                rootElement.scrollIntoView = originalScrollIntoView;
            };
        }
    }, [editor]);

    return null;
}
