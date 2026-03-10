import {
    $getSelection,
    $isRangeSelection,
    LexicalEditor,
    $insertNodes
} from 'lexical';
import { $generateNodesFromDOM } from '@lexical/html';
import { PasteOption } from '../../components/Dialogs/PasteSpecialDialog';
import { PasteData } from '../../components/Editor/plugins/PastePlugin';

/**
 * Handles special paste options by processing the extracted PasteData
 * and inserting nodes into the editor.
 */
export function handleSpecialPaste(
    editor: LexicalEditor,
    data: PasteData,
    option: PasteOption
) {
    editor.update(() => {
        const selection = $getSelection();
        if (!$isRangeSelection(selection)) {
            return;
        }

        if (option === 'plain') {
            const text = data.plain;
            selection.insertText(text);
            return;
        }

        let html = data.html;
        if (!html) {
            // Fallback to plain text if no HTML
            const text = data.plain;
            selection.insertText(text);
            return;
        }

        if (option === 'semantic') {
            html = cleanHtmlSemantically(html);
        }

        const parser = new DOMParser();
        const dom = parser.parseFromString(html, 'text/html');
        const nodes = $generateNodesFromDOM(editor, dom);

        $insertNodes(nodes);
    });
}

/**
 * Strips formatting (style, class, etc.) from HTML but keeps 
 * structural elements like bold, italic, headings, and lists.
 */
function cleanHtmlSemantically(html: string): string {
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');

    // Recursively clean elements
    const cleanElement = (el: Element) => {
        // Remove all attributes except those needed for structure (like 'href' on <a>)
        const attributes = el.attributes;
        for (let i = attributes.length - 1; i >= 0; i--) {
            const attr = attributes[i];
            if (attr.name !== 'href' && attr.name !== 'src') {
                el.removeAttribute(attr.name);
            }
        }

        // Process children
        for (const child of Array.from(el.children)) {
            cleanElement(child);
        }
    };

    if (doc.body) {
        for (const child of Array.from(doc.body.children)) {
            cleanElement(child);
        }
        return doc.body.innerHTML;
    }

    return html;
}
