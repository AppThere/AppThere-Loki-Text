import { useEffect, useRef, useCallback } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';

interface FindReplaceBarProps {
    open: boolean;
    searchTerm: string;
    replaceTerm: string;
    caseSensitive: boolean;
    wholeWord: boolean;
    matchCount: number;
    currentMatch: number;
    onSearchChange: (v: string) => void;
    onReplaceChange: (v: string) => void;
    onCaseSensitiveChange: (v: boolean) => void;
    onWholeWordChange: (v: boolean) => void;
    onFindNext: () => void;
    onFindPrevious: () => void;
    onReplaceOne: () => void;
    onReplaceAll: () => void;
    onClose: () => void;
}

export function FindReplaceBar({
    open,
    searchTerm,
    replaceTerm,
    caseSensitive,
    wholeWord,
    matchCount,
    currentMatch,
    onSearchChange,
    onReplaceChange,
    onCaseSensitiveChange,
    onWholeWordChange,
    onFindNext,
    onFindPrevious,
    onReplaceOne,
    onReplaceAll,
    onClose,
}: FindReplaceBarProps) {
    const searchRef = useRef<HTMLInputElement>(null);

    // Focus search input when bar opens
    useEffect(() => {
        if (open) {
            searchRef.current?.focus();
            searchRef.current?.select();
        }
    }, [open]);

    const handleSearchKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Escape') {
            onClose();
        } else if (e.key === 'Enter') {
            e.preventDefault();
            if (e.shiftKey) {
                onFindPrevious();
            } else {
                onFindNext();
            }
        }
    }, [onClose, onFindNext, onFindPrevious]);

    const handleReplaceKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Escape') {
            onClose();
        } else if (e.key === 'Enter') {
            e.preventDefault();
            onReplaceOne();
        }
    }, [onClose, onReplaceOne]);

    if (!open) return null;

    const matchLabel = matchCount === 0
        ? 'No results'
        : `${currentMatch} of ${matchCount}`;

    return (
        <div
            className="find-replace-bar flex flex-wrap items-center gap-2 px-4 py-2 border-b bg-muted/40 text-sm"
            role="search"
            aria-label="Find and replace"
        >
            {/* Search row */}
            <div className="flex items-center gap-1.5 flex-1 min-w-[200px]">
                <Input
                    ref={searchRef}
                    value={searchTerm}
                    onChange={(e) => onSearchChange(e.target.value)}
                    onKeyDown={handleSearchKeyDown}
                    placeholder="Find…"
                    className="h-7 text-sm w-40"
                    aria-label="Search term"
                />
                <span className="text-muted-foreground text-xs whitespace-nowrap w-20 shrink-0">
                    {matchLabel}
                </span>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onFindPrevious}
                    disabled={matchCount === 0}
                    className="h-7 px-2"
                    title="Previous match (Shift+Enter)"
                    aria-label="Previous match"
                >
                    ▲
                </Button>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onFindNext}
                    disabled={matchCount === 0}
                    className="h-7 px-2"
                    title="Next match (Enter)"
                    aria-label="Next match"
                >
                    ▼
                </Button>
            </div>

            {/* Replace row */}
            <div className="flex items-center gap-1.5 flex-1 min-w-[200px]">
                <Input
                    value={replaceTerm}
                    onChange={(e) => onReplaceChange(e.target.value)}
                    onKeyDown={handleReplaceKeyDown}
                    placeholder="Replace…"
                    className="h-7 text-sm w-40"
                    aria-label="Replace term"
                />
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onReplaceOne}
                    disabled={matchCount === 0}
                    className="h-7 px-2 text-xs"
                    title="Replace current match"
                >
                    Replace
                </Button>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onReplaceAll}
                    disabled={matchCount === 0}
                    className="h-7 px-2 text-xs"
                    title="Replace all matches"
                >
                    All
                </Button>
            </div>

            {/* Options + close */}
            <div className="flex items-center gap-3 ml-auto">
                <div className="flex items-center gap-1.5">
                    <Checkbox
                        id="fr-case"
                        checked={caseSensitive}
                        onCheckedChange={(v) => onCaseSensitiveChange(v === true)}
                    />
                    <Label htmlFor="fr-case" className="text-xs cursor-pointer select-none">
                        Aa
                    </Label>
                </div>
                <div className="flex items-center gap-1.5">
                    <Checkbox
                        id="fr-word"
                        checked={wholeWord}
                        onCheckedChange={(v) => onWholeWordChange(v === true)}
                    />
                    <Label htmlFor="fr-word" className="text-xs cursor-pointer select-none">
                        W
                    </Label>
                </div>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onClose}
                    className="h-7 w-7 p-0 text-base leading-none"
                    aria-label="Close find bar"
                    title="Close (Escape)"
                >
                    ×
                </Button>
            </div>
        </div>
    );
}
