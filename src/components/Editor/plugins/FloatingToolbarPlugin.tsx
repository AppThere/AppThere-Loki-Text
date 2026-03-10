import { useEffect, useState, useRef, useCallback } from 'react';
import { createPortal } from 'react-dom';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { FORMAT_TEXT_COMMAND, SELECTION_CHANGE_COMMAND, COMMAND_PRIORITY_LOW, $getSelection, $isRangeSelection } from 'lexical';
import { Bold, Italic, Underline, Strikethrough, Superscript, Subscript, Scissors, Copy } from 'lucide-react';
import { Button } from '@/components/ui/button';

export function FloatingToolbarPlugin() {
    const [editor] = useLexicalComposerContext();
    const [isTextSelected, setIsTextSelected] = useState(false);
    const [isBold, setIsBold] = useState(false);
    const [isItalic, setIsItalic] = useState(false);
    const [isUnderline, setIsUnderline] = useState(false);
    const [isStrikethrough, setIsStrikethrough] = useState(false);
    const [isSubscript, setIsSubscript] = useState(false);
    const [isSuperscript, setIsSuperscript] = useState(false);

    const [position, setPosition] = useState({ top: 0, left: 0 });
    const toolbarRef = useRef<HTMLDivElement>(null);

    const updateToolbar = useCallback(() => {
        const selection = $getSelection();
        if ($isRangeSelection(selection)) {
            if (selection.isCollapsed() || selection.getTextContent() === '') {
                setIsTextSelected(false);
                return;
            }

            setIsBold(selection.hasFormat('bold'));
            setIsItalic(selection.hasFormat('italic'));
            setIsUnderline(selection.hasFormat('underline'));
            setIsStrikethrough(selection.hasFormat('strikethrough'));
            setIsSubscript(selection.hasFormat('subscript'));
            setIsSuperscript(selection.hasFormat('superscript'));
            setIsTextSelected(true);
        }
    }, []);

    useEffect(() => {
        return editor.registerUpdateListener(({ editorState }) => {
            editorState.read(() => {
                updateToolbar();
            });
        });
    }, [editor, updateToolbar]);

    useEffect(() => {
        return editor.registerCommand(
            SELECTION_CHANGE_COMMAND,
            () => {
                updateToolbar();
                return false;
            },
            COMMAND_PRIORITY_LOW
        );
    }, [editor, updateToolbar]);

    // Position calculation
    useEffect(() => {
        const handleSelectionChange = () => {
            if (!isTextSelected) return;

            const domSelection = window.getSelection();
            if (!domSelection || domSelection.isCollapsed) return;

            const range = domSelection.getRangeAt(0);
            const rect = range.getBoundingClientRect();
            const toolbarDiv = toolbarRef.current;

            if (!toolbarDiv) return;

            // Position above the selection
            const top = rect.top - toolbarDiv.offsetHeight - 10;
            const left = rect.left + rect.width / 2 - toolbarDiv.offsetWidth / 2;

            setPosition({ top: Math.max(0, top), left: Math.max(0, left) });
        };

        if (isTextSelected) {
            document.addEventListener('selectionchange', handleSelectionChange);
            window.addEventListener('resize', handleSelectionChange);
            // Run once immediately
            requestAnimationFrame(handleSelectionChange);
        }

        return () => {
            document.removeEventListener('selectionchange', handleSelectionChange);
            window.removeEventListener('resize', handleSelectionChange);
        };
    }, [isTextSelected]);

    if (!isTextSelected) return null;

    return createPortal(
        <div
            ref={toolbarRef}
            className="absolute z-50 flex items-center gap-1 bg-sky-50 dark:bg-blue-950 border border-sky-200 dark:border-blue-900 shadow-xl rounded-md px-2 py-1 select-none animate-in fade-in duration-200"
            style={{ top: position.top, left: position.left, opacity: position.top > 0 ? 1 : 0 }}
            onMouseDown={(e) => e.preventDefault()} // Prevent taking focus from editor
        >
            <Button size="icon" variant={isBold ? "secondary" : "ghost"} className="h-7 w-7" onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'bold')}>
                <Bold className="h-4 w-4" />
            </Button>
            <Button size="icon" variant={isItalic ? "secondary" : "ghost"} className="h-7 w-7" onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'italic')}>
                <Italic className="h-4 w-4" />
            </Button>
            <Button size="icon" variant={isUnderline ? "secondary" : "ghost"} className="h-7 w-7" onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'underline')}>
                <Underline className="h-4 w-4" />
            </Button>
            <Button size="icon" variant={isStrikethrough ? "secondary" : "ghost"} className="h-7 w-7" onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'strikethrough')}>
                <Strikethrough className="h-4 w-4" />
            </Button>
            <Button size="icon" variant={isSubscript ? "secondary" : "ghost"} className="h-7 w-7" onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'subscript')}>
                <Subscript className="h-4 w-4" />
            </Button>
            <Button size="icon" variant={isSuperscript ? "secondary" : "ghost"} className="h-7 w-7" onClick={() => editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'superscript')}>
                <Superscript className="h-4 w-4" />
            </Button>

            <div className="w-px h-5 bg-gray-300 mx-1" />

            <Button size="icon" variant="ghost" className="h-7 w-7 hover:text-blue-600" onClick={() => document.execCommand('cut')}>
                <Scissors className="h-4 w-4" />
            </Button>
            <Button size="icon" variant="ghost" className="h-7 w-7 hover:text-blue-600" onClick={() => document.execCommand('copy')}>
                <Copy className="h-4 w-4" />
            </Button>
        </div>,
        document.body
    );
}
