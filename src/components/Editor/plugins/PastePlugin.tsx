import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import {
    $getSelection,
    $isRangeSelection,
    COMMAND_PRIORITY_HIGH,
    PASTE_COMMAND
} from 'lexical';
import { useEffect } from 'react';

export interface PasteData {
    plain: string;
    html: string;
}

interface PastePluginProps {
    onOpenPasteSpecial: (data: PasteData) => void;
}

export function PastePlugin({ onOpenPasteSpecial }: PastePluginProps) {
    const [editor] = useLexicalComposerContext();

    useEffect(() => {
        return editor.registerCommand(
            PASTE_COMMAND,
            (event: ClipboardEvent | KeyboardEvent | null) => {
                const selection = $getSelection();
                if (!$isRangeSelection(selection) || !(event instanceof ClipboardEvent)) {
                    return false;
                }

                const dataTransfer = event.clipboardData;
                if (!dataTransfer) {
                    return false;
                }

                const html = dataTransfer.getData('text/html');
                const plain = dataTransfer.getData('text/plain');

                // Heuristic: Does the HTML contain meaningful formatting tags or styles?
                // We check for common formatting tags or style attributes.
                const hasFormatting = /<(b|strong|i|em|u|h[1-6]|ul|ol|li|a|table|style|div style|span style)/i.test(html);

                // If it's just plain text or very simple HTML with no formatting, 
                // let Lexical handle it normally.
                if (!html || !hasFormatting) {
                    return false;
                }

                // If it has HTML formatting, intercept and show our custom dialog
                event.preventDefault();
                onOpenPasteSpecial({ plain, html });
                return true;
            },
            COMMAND_PRIORITY_HIGH
        );
    }, [editor, onOpenPasteSpecial]);

    return null;
}
