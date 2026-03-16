// Copyright 2024 AppThere
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { describe, it, expect, afterEach } from 'vitest';
import { render, screen, cleanup } from '@testing-library/react';

afterEach(cleanup);
import { ViolationList } from './ViolationList';
import type { ConformanceViolation } from '@/lib/vector/pdfTypes';

describe('ViolationList', () => {
    it('renders empty state when no violations', () => {
        render(<ViolationList violations={[]} />);
        expect(screen.getByText(/No violations found/i)).toBeTruthy();
    });

    it('shows loading skeleton when loading=true', () => {
        const { container } = render(<ViolationList violations={[]} loading={true} />);
        // Skeleton items are rendered as animate-pulse list items
        const items = container.querySelectorAll('.animate-pulse');
        expect(items.length).toBeGreaterThan(0);
    });

    it('renders error violations with Error badge', () => {
        const violations: ConformanceViolation[] = [
            { rule: 'X1a/no-transparency', message: 'Transparency not allowed in X1a' },
        ];
        render(<ViolationList violations={violations} />);
        expect(screen.getByText('Error')).toBeTruthy();
        expect(screen.getByText('Transparency not allowed in X1a')).toBeTruthy();
        expect(screen.getByText('X1a/no-transparency')).toBeTruthy();
    });

    it('renders X/empty-document as warning', () => {
        const violations: ConformanceViolation[] = [
            { rule: 'X/empty-document', message: 'Document has no objects' },
        ];
        render(<ViolationList violations={violations} />);
        expect(screen.getByText('Warning')).toBeTruthy();
        expect(screen.getByText('Document has no objects')).toBeTruthy();
    });

    it('shows error and warning counts separately', () => {
        const violations: ConformanceViolation[] = [
            { rule: 'X/empty-document', message: 'empty' },
            { rule: 'X1a/no-rgb', message: 'no rgb allowed' },
        ];
        render(<ViolationList violations={violations} />);
        expect(screen.getByText('Error')).toBeTruthy();
        expect(screen.getByText('Warning')).toBeTruthy();
    });
});
