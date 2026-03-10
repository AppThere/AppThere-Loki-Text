import { useEffect, useRef } from 'react';
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext';
import { EditorState } from 'lexical';

interface DebouncedOnChangePluginProps {
    onChange: (editorState: EditorState) => void;
    debounceMs?: number;
}

export function DebouncedOnChangePlugin({ onChange, debounceMs = 500 }: DebouncedOnChangePluginProps) {
    const [editor] = useLexicalComposerContext();
    const timeoutRef = useRef<NodeJS.Timeout | null>(null);
    const isDirtyRef = useRef(false);

    useEffect(() => {
        return editor.registerUpdateListener(({ editorState, dirtyElements, dirtyLeaves }) => {
            // Skip if no changes (Lexical fires updates even when nothing changed)
            if (dirtyElements.size === 0 && dirtyLeaves.size === 0) {
                return;
            }

            isDirtyRef.current = true;

            // Clear existing timeout
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
            }

            // Set new timeout
            timeoutRef.current = setTimeout(() => {
                if (isDirtyRef.current) {
                    onChange(editorState);
                    isDirtyRef.current = false;
                }
            }, debounceMs);
        });
    }, [editor, onChange, debounceMs]);

    // Cleanup on unmount
    useEffect(() => {
        return () => {
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
            }
        };
    }, []);

    return null;
}
