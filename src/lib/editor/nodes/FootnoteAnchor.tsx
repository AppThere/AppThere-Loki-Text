// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { useCallback } from 'react';
import { useFootnoteNumber } from './useFootnoteNumber';

interface FootnoteAnchorProps {
    footnoteId: string;
}

/**
 * Inline superscript rendered inside the body text for a footnote reference.
 * Clicking scrolls to the corresponding footnote panel entry.
 */
export function FootnoteAnchor({ footnoteId }: FootnoteAnchorProps) {
    const number = useFootnoteNumber(footnoteId);

    const handleClick = useCallback(() => {
        const el = document.getElementById(`footnote-entry-${footnoteId}`);
        if (el) {
            el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
            const focusable = el.querySelector<HTMLElement>('[contenteditable]');
            focusable?.focus();
        }
    }, [footnoteId]);

    return (
        <span className="footnote-anchor">
            <sup onClick={handleClick} role="button" tabIndex={0}
                onKeyDown={(e) => e.key === 'Enter' && handleClick()}
                aria-label={`Footnote ${number}`}>
                {number}
            </sup>
        </span>
    );
}
