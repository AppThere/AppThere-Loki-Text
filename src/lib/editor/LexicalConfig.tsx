import { InitialConfigType } from '@lexical/react/LexicalComposer';
import { ParagraphNode } from 'lexical';
import { QuoteNode } from '@lexical/rich-text';
import { HeadingNode } from '@lexical/rich-text';
import { ListNode, ListItemNode } from '@lexical/list';
import { TableNode, TableCellNode, TableRowNode } from '@lexical/table';
import { LinkNode } from '@lexical/link';
import { ImageNode } from './nodes/ImageNode';
import { PageBreakNode } from './nodes/PageBreakNode';
import { ParagraphStyleNode } from './nodes/ParagraphStyleNode';
import { HeadingStyleNode } from './nodes/HeadingStyleNode';

export const editorConfig: InitialConfigType = {
    namespace: 'AppThereLoki',
    theme: {
        // Tailwind classes for editor empty root styles
        paragraph: 'mb-2',
        list: {
            ul: 'list-disc ml-6 mb-2',
            ol: 'list-decimal ml-6 mb-2',
            listitem: 'mb-1',
        },
        text: {
            bold: 'font-bold',
            italic: 'italic',
            underline: 'underline',
            strikethrough: 'line-through',
        },
        link: 'text-blue-600 underline',
    },
    nodes: [
        {
            replace: HeadingNode,
            with: (node: HeadingNode) => {
                return new HeadingStyleNode(node.getTag(), null);
            },
        },
        HeadingStyleNode,
        QuoteNode,
        ListNode,
        ListItemNode,
        TableNode,
        TableCellNode,
        TableRowNode,
        LinkNode,
        ImageNode,
        PageBreakNode,
        {
            replace: ParagraphNode,
            with: (_node: ParagraphNode) => {
                return new ParagraphStyleNode(null);
            },
        },
        ParagraphStyleNode,  // Custom node for ODT paragraph styles
    ],
    onError: (error: Error) => {
        console.error('Lexical error:', error);
    },
};
