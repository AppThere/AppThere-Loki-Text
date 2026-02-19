import { resolveStyle, type BlockStyle } from '../styleStore';
import type { Editor } from '@tiptap/core';
import { SvelteSet } from 'svelte/reactivity';

export class AutocompleteController {
	index = $state<Record<string, Set<string>>>({});
	suggestions = $state<string[]>([]);
	selectedSuggestionIndex = $state(0);
	showSuggestions = $state(false);
	suggestionPosition = $state({ top: 0, left: 0 });
	suggestionQuery = $state('');

	buildIndex(editor: Editor, registry: BlockStyle[]) {
		const idx: Record<string, Set<string>> = {};

		editor.state.doc.descendants((node) => {
			if (node.type.name === 'paragraph' || node.type.name === 'heading') {
				const styleName = node.attrs.styleName || 'Normal Text';
				const style = resolveStyle(styleName, registry);

				if (style.autocomplete) {
					if (!idx[styleName]) idx[styleName] = new SvelteSet();
					const text = node.textContent.trim();
					if (text) idx[styleName].add(text);
				}
			}
			return true;
		});
		this.index = idx;
	}

	check(editor: Editor, registry: BlockStyle[]) {
		const { selection } = editor.state;
		const { $from: fromPos } = selection;
		const node = fromPos.node(fromPos.depth);

		const styleName = node.attrs.styleName || 'Normal Text';
		const style = resolveStyle(styleName, registry);

		if (!style.autocomplete) {
			this.showSuggestions = false;
			return;
		}

		const textBefore = fromPos.parent.textBetween(0, fromPos.parentOffset, '\n', '\uFFFC');

		const query = textBefore.trim();
		this.suggestionQuery = query;

		if (query.length < 1) {
			this.showSuggestions = false;
			return;
		}

		const candidates = this.index[styleName] || new SvelteSet();
		const matches = Array.from(candidates)
			.filter(
				(c) =>
					c.toLowerCase().startsWith(query.toLowerCase()) && c.toLowerCase() !== query.toLowerCase()
			)
			.sort()
			.slice(0, 5);

		if (matches.length > 0) {
			this.suggestions = matches;
			this.selectedSuggestionIndex = 0;

			const coords = editor.view.coordsAtPos(fromPos.pos);

			this.suggestionPosition = {
				top: coords.bottom + 5,
				left: coords.left
			};
			this.showSuggestions = true;
		} else {
			this.showSuggestions = false;
		}
	}

	accept(editor: Editor) {
		if (!this.showSuggestions) return;
		const suggestion = this.suggestions[this.selectedSuggestionIndex];
		if (!suggestion) return;

		const { selection } = editor.state;
		const { $from: fromPos } = selection;
		const node = fromPos.node(fromPos.depth);
		const startPos = fromPos.start();

		editor
			.chain()
			.deleteRange({ from: startPos, to: startPos + node.content.size })
			.insertContentAt(startPos, suggestion)
			.run();

		this.showSuggestions = false;
	}

	handleKeyDown(event: KeyboardEvent, editor: Editor): boolean {
		if (!this.showSuggestions) return false;

		if (event.key === 'ArrowDown') {
			event.preventDefault();
			this.selectedSuggestionIndex = (this.selectedSuggestionIndex + 1) % this.suggestions.length;
			return true;
		}
		if (event.key === 'ArrowUp') {
			event.preventDefault();
			this.selectedSuggestionIndex =
				(this.selectedSuggestionIndex - 1 + this.suggestions.length) % this.suggestions.length;
			return true;
		}
		if (event.key === 'Enter' || event.key === 'Tab') {
			event.preventDefault();
			this.accept(editor);
			return true;
		}
		if (event.key === 'Escape') {
			this.showSuggestions = false;
			return true;
		}
		return false;
	}
}
