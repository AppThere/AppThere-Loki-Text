// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { createContext, useContext } from 'react';

export type DocumentViewMode = 'scroll' | 'preview';

export interface DocumentViewContextValue {
    viewMode: DocumentViewMode;
    setViewMode: (mode: DocumentViewMode) => void;
}

export const DocumentViewContext = createContext<DocumentViewContextValue>({
    viewMode: 'scroll',
    setViewMode: () => {},
});

export function useDocumentView(): DocumentViewContextValue {
    return useContext(DocumentViewContext);
}
