/**
 * Pure, side-effect-free helpers for Find & Replace match scanning.
 * No Lexical imports — easily unit-testable.
 */

export interface TextMatch {
    /** Inclusive start offset within the text string. */
    start: number;
    /** Exclusive end offset within the text string. */
    end: number;
}

export interface NodeMatch extends TextMatch {
    /** The Lexical TextNode key that owns this match. */
    nodeKey: string;
}

/**
 * Find all substring matches of `searchTerm` within `text`.
 *
 * @param text          The haystack string.
 * @param searchTerm    The needle. Returns [] when empty.
 * @param caseSensitive When false, matching is case-insensitive.
 * @param wholeWord     When true, only whole-word occurrences match.
 */
export function findMatches(
    text: string,
    searchTerm: string,
    caseSensitive: boolean,
    wholeWord: boolean,
): TextMatch[] {
    if (!searchTerm) return [];

    const escaped = searchTerm.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const pattern = wholeWord ? `\\b${escaped}\\b` : escaped;
    const flags = caseSensitive ? 'g' : 'gi';

    try {
        const re = new RegExp(pattern, flags);
        const matches: TextMatch[] = [];
        let m: RegExpExecArray | null;
        while ((m = re.exec(text)) !== null) {
            matches.push({ start: m.index, end: m.index + m[0].length });
            // Avoid infinite loops on zero-width matches
            if (m[0].length === 0) re.lastIndex++;
        }
        return matches;
    } catch {
        return [];
    }
}
