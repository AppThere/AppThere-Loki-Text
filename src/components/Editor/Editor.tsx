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
import { ListPlugin } from '@lexical/react/LexicalListPlugin';
import { ImagePlugin } from './plugins/ImagePlugin';
import { PastePlugin, PasteData } from './plugins/PastePlugin';
import { PasteSpecialDialog, PasteOption } from '../Dialogs/PasteSpecialDialog';
import { handleSpecialPaste } from '@/lib/utils/pasteUtils';
import { useDocumentStore } from '@/lib/stores/documentStore';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { useState, useCallback, useRef, useEffect } from 'react';
import type { StyleDefinition } from '@/lib/types/odt';
import { FindReplacePlugin, type FindReplaceHandle } from './plugins/FindReplacePlugin';
import { FindReplaceBar } from '../FindReplaceBar';
import { FootnotePlugin } from '@/editor/plugins/FootnotePlugin';
import { DocumentViewProvider } from '@/editor/views/DocumentViewProvider';

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

    // Find & Replace state
    const [findOpen, setFindOpen] = useState(false);
    const [searchTerm, setSearchTerm] = useState('');
    const [replaceTerm, setReplaceTerm] = useState('');
    const [caseSensitive, setCaseSensitive] = useState(false);
    const [wholeWord, setWholeWord] = useState(false);
    const [matchCount, setMatchCount] = useState(0);
    const [currentMatch, setCurrentMatch] = useState(0);
    const findReplaceRef = useRef<FindReplaceHandle>(null);

    const handleOpenPasteSpecial = useCallback((data: PasteData) => {
        setLastPasteData(data);
        setPasteSpecialOpen(true);
    }, []);

    const handleFindOpen = useCallback(() => {
        setFindOpen(true);
    }, []);

    const handleFindClose = useCallback(() => {
        setFindOpen(false);
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
                findOpen={findOpen}
                searchTerm={searchTerm}
                replaceTerm={replaceTerm}
                caseSensitive={caseSensitive}
                wholeWord={wholeWord}
                matchCount={matchCount}
                currentMatch={currentMatch}
                findReplaceRef={findReplaceRef}
                setSearchTerm={setSearchTerm}
                setReplaceTerm={setReplaceTerm}
                setCaseSensitive={setCaseSensitive}
                setWholeWord={setWholeWord}
                setMatchCount={setMatchCount}
                setCurrentMatch={setCurrentMatch}
                onFindOpen={handleFindOpen}
                onFindClose={handleFindClose}
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
    handleOpenPasteSpecial,
    findOpen,
    searchTerm,
    replaceTerm,
    caseSensitive,
    wholeWord,
    matchCount,
    currentMatch,
    findReplaceRef,
    setSearchTerm,
    setReplaceTerm,
    setCaseSensitive,
    setWholeWord,
    setMatchCount,
    setCurrentMatch,
    onFindOpen,
    onFindClose,
}: any) {
    const [editor] = useLexicalComposerContext();

    const onPasteSelect = (option: PasteOption) => {
        if (lastPasteData) {
            handleSpecialPaste(editor, lastPasteData, option);
        }
        setPasteSpecialOpen(false);
    };

    // Keyboard shortcut for Find (Ctrl/Cmd+F)
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'f') {
                e.preventDefault();
                onFindOpen();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onFindOpen]);

    return (
        <div className="editor-container h-full flex flex-col relative w-full min-h-0">
            <FindReplaceBar
                open={findOpen}
                searchTerm={searchTerm}
                replaceTerm={replaceTerm}
                caseSensitive={caseSensitive}
                wholeWord={wholeWord}
                matchCount={matchCount}
                currentMatch={currentMatch}
                onSearchChange={setSearchTerm}
                onReplaceChange={setReplaceTerm}
                onCaseSensitiveChange={setCaseSensitive}
                onWholeWordChange={setWholeWord}
                onFindNext={() => findReplaceRef.current?.findNext()}
                onFindPrevious={() => findReplaceRef.current?.findPrevious()}
                onReplaceOne={() => findReplaceRef.current?.replaceOne()}
                onReplaceAll={() => findReplaceRef.current?.replaceAll()}
                onClose={onFindClose}
            />

            <DocumentViewProvider>
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
                <MenuPlugin onFindOpen={onFindOpen} />
                <TablePlugin />
                <ListPlugin />
                <LinkPlugin />
                <ImagePlugin />
                <PastePlugin onOpenPasteSpecial={handleOpenPasteSpecial} />
                <FindReplacePlugin
                    searchTerm={searchTerm}
                    replaceTerm={replaceTerm}
                    caseSensitive={caseSensitive}
                    wholeWord={wholeWord}
                    active={findOpen}
                    onMatchCountChange={setMatchCount}
                    onCurrentMatchChange={setCurrentMatch}
                    imperativeRef={findReplaceRef}
                />
                <DebouncedOnChangePlugin onChange={(editorState) => {
                    if (_onContentChange) {
                        _onContentChange(JSON.stringify(editorState.toJSON()));
                    }
                }} />
                <FootnotePlugin />
            </DocumentViewProvider>

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
