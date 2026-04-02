// Copyright 2024 AppThere Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

import { createCommand, type LexicalCommand } from 'lexical';

/** Dispatched to insert a new footnote reference at the current selection. */
export const INSERT_FOOTNOTE_COMMAND: LexicalCommand<undefined> =
    createCommand('INSERT_FOOTNOTE_COMMAND');
