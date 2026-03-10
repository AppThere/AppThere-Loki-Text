import { LexicalComposer } from '@lexical/react/LexicalComposer';
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin';
import { ContentEditable } from '@lexical/react/LexicalContentEditable';
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin';
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary';
import { DebouncedOnChangePlugin } from './plugins/DebouncedOnChangePlugin';
import { editorConfig } from '@/lib/editor/LexicalConfig';
import { Toolbar } from './Toolbar';
import { StylePlugin } from './plugins/StylePlugin';
import { NextStylePlugin } from './plugins/NextStylePlugin';
import { FloatingToolbarPlugin } from './plugins/FloatingToolbarPlugin';
import { DocumentStylesPlugin } from './plugins/DocumentStylesPlugin';
import { ScrollPlugin } from './plugins/ScrollPlugin';
import { MenuPlugin } from './plugins/MenuPlugin';
import { TablePlugin } from '@lexical/react/LexicalTablePlugin';
import { LinkPlugin } from '@lexical/react/LexicalLinkPlugin';
import { ImagePlugin } from './plugins/ImagePlugin';
import { PastePlugin, PasteData } from './plugins/PastePlugin';
import { PasteSpecialDialog, PasteOption } from '../Dialogs/PasteSpecialDialog';
import { handleSpecialPaste } from '@/lib/utils/pasteUtils';
import { useDocumentStore } from '@/lib/stores/documentStore';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { useState, useCallback } from 'react';
import type { StyleDefinition } from '@/lib/types/odt';

interface EditorProps {
    initialContent?: string;
    onContentChange?: (content: string) => void;
    styles: Record<string, StyleDefinition>;
    onStylesClick?: () => void;
}

export function Editor({
    initialContent: _initialContent,
    onContentChange: _onContentChange,
    styles,
    onStylesClick,
}: EditorProps) {
    const { currentStyle, setStyle } = useDocumentStore();
    const [pasteSpecialOpen, setPasteSpecialOpen] = useState(false);
    const [lastPasteData, setLastPasteData] = useState<PasteData | null>(null);

    const handleOpenPasteSpecial = useCallback((data: PasteData) => {
        setLastPasteData(data);
        setPasteSpecialOpen(true);
    }, []);

    const config = {
        ...editorConfig,
        editorState: _initialContent,
    };

    return (
        <LexicalComposer initialConfig={config}>
            <EditorInner
                styles={styles}
                currentStyle={currentStyle}
                setStyle={setStyle}
                onStylesClick={onStylesClick}
                onContentChange={_onContentChange}
                pasteSpecialOpen={pasteSpecialOpen}
                setPasteSpecialOpen={setPasteSpecialOpen}
                lastPasteData={lastPasteData}
                handleOpenPasteSpecial={handleOpenPasteSpecial}
            />
        </LexicalComposer>
    );
}

// Split the inner part to use useLexicalComposerContext
function EditorInner({
    styles,
    currentStyle,
    setStyle,
    onStylesClick,
    onContentChange: _onContentChange,
    pasteSpecialOpen,
    setPasteSpecialOpen,
    lastPasteData,
    handleOpenPasteSpecial
}: any) {
    const [editor] = useLexicalComposerContext();

    const onPasteSelect = (option: PasteOption) => {
        if (lastPasteData) {
            handleSpecialPaste(editor, lastPasteData, option);
        }
        setPasteSpecialOpen(false);
    };

    return (
        <div className="editor-container h-full flex flex-col relative w-full bg-slate-200 dark:bg-stone-900 min-h-0">
            <div className="editor-inner relative flex-1 px-8 lg:px-24 pt-5 pb-32 overflow-y-auto max-w-4xl mx-auto w-full min-h-0 bg-background shadow-md border-x text-foreground">
                <RichTextPlugin
                    contentEditable={
                        <ContentEditable className="editor-input min-h-[200px] outline-none font-serif" />
                    }
                    placeholder={null}
                    ErrorBoundary={LexicalErrorBoundary}
                />

                <HistoryPlugin />
                <TablePlugin />
                <ScrollPlugin />
                <DocumentStylesPlugin styles={styles} />
                <StylePlugin currentStyle={currentStyle} onStyleChange={setStyle} styles={styles} />
                <NextStylePlugin styles={styles} />
                <FloatingToolbarPlugin />
                <MenuPlugin />
                <TablePlugin />
                <LinkPlugin />
                <ImagePlugin />
                <PastePlugin onOpenPasteSpecial={handleOpenPasteSpecial} />
                <DebouncedOnChangePlugin onChange={(editorState) => {
                    if (_onContentChange) {
                        _onContentChange(JSON.stringify(editorState.toJSON()));
                    }
                }} />
            </div>

            <Toolbar
                styles={styles}
                currentStyle={currentStyle}
                onStyleChange={setStyle}
                onStylesClick={onStylesClick}
            />

            <PasteSpecialDialog
                open={pasteSpecialOpen}
                onOpenChange={setPasteSpecialOpen}
                onSelect={onPasteSelect}
            />
        </div>
    );
}
