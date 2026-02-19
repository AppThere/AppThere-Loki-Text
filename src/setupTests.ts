import '@testing-library/jest-dom';
import { vi } from 'vitest';

// Mock Tauri APIs globally if needed
Object.defineProperty(window, '__TAURI_INTERNALS__', {
	value: {
		invoke: vi.fn()
	},
	writable: true
});
