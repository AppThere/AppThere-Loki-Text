import { useEffect } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { listen } from '@tauri-apps/api/event';
import {
    UNDO_COMMAND,
    REDO_COMMAND,
    FORMAT_TEXT_COMMAND
} from 'lexical';

export function MenuPlugin() {
    const [editor] = useLexicalComposerContext();

    useEffect(() => {
        const unlistenPromises = [
            listen('menu-undo', () => {
                editor.dispatchCommand(UNDO_COMMAND, undefined);
            }),
            listen('menu-redo', () => {
                editor.dispatchCommand(REDO_COMMAND, undefined);
            }),
            listen('menu-bold', () => {
                editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'bold');
            }),
            listen('menu-italic', () => {
                editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'italic');
            }),
            listen('menu-underline', () => {
                editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'underline');
            }),
            listen('menu-print', () => {
                window.print();
            }),
        ];

        // Also handle standard keyboard shortcuts for formatting that might not be 
        // covered by the native menu if the user hasn't selected them from the menu
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.metaKey || e.ctrlKey) {
                switch (e.key.toLowerCase()) {
                    case 'b':
                        e.preventDefault();
                        editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'bold');
                        break;
                    case 'i':
                        e.preventDefault();
                        editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'italic');
                        break;
                    case 'u':
                        e.preventDefault();
                        editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'underline');
                        break;
                    case 'p':
                        e.preventDefault();
                        window.print();
                        break;
                }
            }
        };

        window.addEventListener('keydown', handleKeyDown);

        return () => {
            unlistenPromises.forEach(async (p) => (await p)());
            window.removeEventListener('keydown', handleKeyDown);
        };
    }, [editor]);

    return null;
}
