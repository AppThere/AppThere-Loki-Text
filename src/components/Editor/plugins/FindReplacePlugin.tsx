import { useEffect, useRef, useImperativeHandle, useCallback } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { $getNodeByKey, $nodesOfType, TextNode } from 'lexical';
import { findMatches, type NodeMatch } from '@/lib/utils/findReplace';

export interface FindReplaceHandle {
    findNext: () => void;
    findPrevious: () => void;
    replaceOne: () => void;
    replaceAll: () => void;
}

interface FindReplacePluginProps {
    searchTerm: string;
    replaceTerm: string;
    caseSensitive: boolean;
    wholeWord: boolean;
    active: boolean;
    onMatchCountChange: (count: number) => void;
    onCurrentMatchChange: (index: number) => void;
    imperativeRef: React.RefObject<FindReplaceHandle>;
}

const HIGHLIGHT_STYLE = 'CSS' in window && 'highlights' in (CSS as unknown as Record<string, unknown>);

function applyHighlights(
    matches: NodeMatch[],
    currentIdx: number,
    getEl: (key: string) => HTMLElement | null,
): void {
    if (!HIGHLIGHT_STYLE) return;
    const h = (CSS as unknown as { highlights: Map<string, unknown> }).highlights;
    const allRanges: Range[] = [];
    const currentRanges: Range[] = [];

    for (let i = 0; i < matches.length; i++) {
        const m = matches[i];
        const el = getEl(m.nodeKey);
        if (!el) continue;
        const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
        const textNode = walker.nextNode();
        if (!textNode) continue;
        const range = new Range();
        range.setStart(textNode, m.start);
        range.setEnd(textNode, m.end);
        if (i === currentIdx) {
            currentRanges.push(range);
        } else {
            allRanges.push(range);
        }
    }

    const HighlightCls = (window as unknown as { Highlight: new (...r: Range[]) => unknown }).Highlight;
    h.set('loki-find-match', new HighlightCls(...allRanges));
    h.set('loki-find-match-current', new HighlightCls(...currentRanges));
}

function clearHighlights(): void {
    if (!HIGHLIGHT_STYLE) return;
    const h = (CSS as unknown as { highlights: Map<string, unknown> }).highlights;
    h.delete('loki-find-match');
    h.delete('loki-find-match-current');
}

export function FindReplacePlugin({
    searchTerm,
    replaceTerm,
    caseSensitive,
    wholeWord,
    active,
    onMatchCountChange,
    onCurrentMatchChange,
    imperativeRef,
}: FindReplacePluginProps) {
    const [editor] = useLexicalComposerContext();
    const matchesRef = useRef<NodeMatch[]>([]);
    const currentIdxRef = useRef(0);

    const scanMatches = useCallback(() => {
        if (!active || !searchTerm) {
            matchesRef.current = [];
            currentIdxRef.current = 0;
            onMatchCountChange(0);
            onCurrentMatchChange(0);
            clearHighlights();
            return;
        }

        const newMatches: NodeMatch[] = [];
        editor.getEditorState().read(() => {
            const textNodes = $nodesOfType(TextNode);
            for (const node of textNodes) {
                const text = node.getTextContent();
                const hits = findMatches(text, searchTerm, caseSensitive, wholeWord);
                for (const hit of hits) {
                    newMatches.push({ nodeKey: node.getKey(), ...hit });
                }
            }
        });

        matchesRef.current = newMatches;
        if (currentIdxRef.current >= newMatches.length) {
            currentIdxRef.current = newMatches.length > 0 ? 0 : 0;
        }
        onMatchCountChange(newMatches.length);
        onCurrentMatchChange(newMatches.length > 0 ? currentIdxRef.current + 1 : 0);
        applyHighlights(newMatches, currentIdxRef.current, (k) => editor.getElementByKey(k));
    }, [active, searchTerm, caseSensitive, wholeWord, editor, onMatchCountChange, onCurrentMatchChange]);

    // Re-scan whenever the editor state or search options change
    useEffect(() => {
        scanMatches();
        return editor.registerUpdateListener(() => scanMatches());
    }, [editor, scanMatches]);

    // Scroll current match into view
    const scrollToCurrent = useCallback(() => {
        const m = matchesRef.current[currentIdxRef.current];
        if (!m) return;
        editor.getElementByKey(m.nodeKey)?.scrollIntoView({ block: 'center' });
        applyHighlights(matchesRef.current, currentIdxRef.current, (k) => editor.getElementByKey(k));
        onCurrentMatchChange(matchesRef.current.length > 0 ? currentIdxRef.current + 1 : 0);
    }, [editor, onCurrentMatchChange]);

    const findNext = useCallback(() => {
        if (!matchesRef.current.length) return;
        currentIdxRef.current = (currentIdxRef.current + 1) % matchesRef.current.length;
        scrollToCurrent();
    }, [scrollToCurrent]);

    const findPrevious = useCallback(() => {
        if (!matchesRef.current.length) return;
        const len = matchesRef.current.length;
        currentIdxRef.current = (currentIdxRef.current - 1 + len) % len;
        scrollToCurrent();
    }, [scrollToCurrent]);

    const replaceOne = useCallback(() => {
        const m = matchesRef.current[currentIdxRef.current];
        if (!m) return;
        editor.update(() => {
            const node = $getNodeByKey(m.nodeKey);
            if (node instanceof TextNode) {
                node.spliceText(m.start, m.end - m.start, replaceTerm, true);
            }
        }, { tag: 'find-replace' });
    }, [editor, replaceTerm]);

    const replaceAll = useCallback(() => {
        const matches = [...matchesRef.current].reverse();
        if (!matches.length) return;
        editor.update(() => {
            for (const m of matches) {
                const node = $getNodeByKey(m.nodeKey);
                if (node instanceof TextNode) {
                    node.spliceText(m.start, m.end - m.start, replaceTerm, false);
                }
            }
        }, { tag: 'find-replace-all' });
    }, [editor, replaceTerm]);

    useImperativeHandle(imperativeRef, () => ({
        findNext,
        findPrevious,
        replaceOne,
        replaceAll,
    }));

    return null;
}
