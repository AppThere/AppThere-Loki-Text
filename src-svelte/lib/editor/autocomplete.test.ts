import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AutocompleteController } from './autocomplete.svelte'; // Note: .svelte extension handling in test
import type { Editor } from '@tiptap/core';
import { type BlockStyle } from '../styleStore';

// Mock types
type MockNode = {
	type: { name: string };
	attrs: { styleName?: string };
	textContent: string;
	content: { size: number };
};

describe('AutocompleteController', () => {
	let controller: AutocompleteController;
	let mockEditor: any;
	let mockRegistry: BlockStyle[];

	beforeEach(() => {
		controller = new AutocompleteController();
		mockRegistry = [
			{
				id: 'Character',
				name: 'Character',
				description: 'Character name',
				autocomplete: true
			},
			{
				id: 'Scene Heading',
				name: 'Scene Heading',
				description: 'Location and time',
				autocomplete: true
			},
			{
				id: 'Action',
				name: 'Action',
				description: 'Physical movement',
				autocomplete: false
			}
		];

		// Basic mock of Tiptap Editor
		mockEditor = {
			state: {
				doc: {
					descendants: vi.fn()
				},
				selection: {
					$from: {
						pos: 100,
						depth: 1,
						parentOffset: 5,
						node: vi.fn(), // Helper to get node at depth
						start: vi.fn().mockReturnValue(90),
						parent: {
							textBetween: vi.fn()
						}
					}
				}
			},
			view: {
				coordsAtPos: vi.fn().mockReturnValue({ left: 100, bottom: 200 }),
				dom: {
					getBoundingClientRect: vi.fn().mockReturnValue({})
				}
			},
			chain: vi.fn().mockReturnValue({
				deleteRange: vi.fn().mockReturnThis(),
				insertContentAt: vi.fn().mockReturnThis(),
				run: vi.fn()
			})
		};
	});

	it('should build index correctly', () => {
		// Setup mock nodes
		const nodes: MockNode[] = [
			{
				type: { name: 'paragraph' },
				attrs: { styleName: 'Character' },
				textContent: 'JOHN DOI',
				content: { size: 8 }
			},
			{
				type: { name: 'paragraph' },
				attrs: { styleName: 'Character' },
				textContent: 'JANE SMITH',
				content: { size: 10 }
			},
			{
				type: { name: 'paragraph' },
				attrs: { styleName: 'Action' },
				textContent: 'Walking',
				content: { size: 7 }
			}
		];

		mockEditor.state.doc.descendants.mockImplementation((callback: (node: any) => boolean | void) => {
			nodes.forEach((node) => callback(node));
		});

		controller.buildIndex(mockEditor, mockRegistry);

		const index = controller.index;
		expect(index['Character']).toBeDefined();
		expect(index['Character'].has('JOHN DOI')).toBe(true);
		expect(index['Character'].has('JANE SMITH')).toBe(true);
		expect(index['Action']).toBeUndefined(); // Should not index non-autocomplete styles
	});

	it('should provide suggestions matching query', () => {
		// Populate index
		controller.index = {
			Character: new Set(['JOHN DOI', 'JANE SMITH', 'JIMMY'])
		};

		// Mock selection in a Character block with text "J"
		const currentBlock = {
			type: { name: 'paragraph' },
			attrs: { styleName: 'Character' }
		};

		mockEditor.state.selection.$from.node.mockReturnValue(currentBlock);
		mockEditor.state.selection.$from.parent.textBetween.mockReturnValue('J');

		controller.check(mockEditor, mockRegistry);

		expect(controller.showSuggestions).toBe(true);
		expect(controller.suggestions).toEqual(['JANE SMITH', 'JIMMY', 'JOHN DOI']);
		expect(controller.suggestionQuery).toBe('J');
	});

	it('should not suggest if query matches full entry (already typed)', () => {
		controller.index = {
			Character: new Set(['JOHN DOI'])
		};

		const currentBlock = {
			type: { name: 'paragraph' },
			attrs: { styleName: 'Character' }
		};

		mockEditor.state.selection.$from.node.mockReturnValue(currentBlock);
		// User typed full name
		mockEditor.state.selection.$from.parent.textBetween.mockReturnValue('JOHN DOI');

		controller.check(mockEditor, mockRegistry);

		expect(controller.showSuggestions).toBe(false);
	});

	it('should accept suggestion and replace content', () => {
		controller.showSuggestions = true;
		controller.suggestions = ['JOHN DOI'];
		controller.selectedSuggestionIndex = 0;

		const currentBlock = {
			type: { name: 'paragraph' },
			attrs: { styleName: 'Character' },
			content: { size: 5 } // Length of text so far? No, node size.
		};

		mockEditor.state.selection.$from.node.mockReturnValue(currentBlock);

		controller.accept(mockEditor);

		expect(mockEditor.chain).toHaveBeenCalled();
		// Check deleteRange and insertContentAt calls
		// This is simplified but verify flow
		expect(controller.showSuggestions).toBe(false);
	});
});
