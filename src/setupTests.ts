import '@testing-library/jest-dom';

// Mock Tauri APIs globally if needed
Object.defineProperty(window, '__TAURI_INTERNALS__', {
	value: {
		invoke: vi.fn()
	},
	writable: true
});
