import { Node, mergeAttributes } from '@tiptap/core';

export interface PageBreakOptions {
	HTMLAttributes: Record<string, any>;
}

declare module '@tiptap/core' {
	interface Commands<ReturnType> {
		pageBreak: {
			/**
			 * Add a page break
			 */
			setPageBreak: () => ReturnType;
		};
	}
}

export const PageBreak = Node.create<PageBreakOptions>({
	name: 'pageBreak',

	addOptions() {
		return {
			HTMLAttributes: {
				class: 'page-break'
			}
		};
	},

	group: 'block',

	parseHTML() {
		return [{ tag: 'div.page-break' }, { tag: 'hr.page-break' }];
	},

	renderHTML({ HTMLAttributes }) {
		return ['hr', mergeAttributes(this.options.HTMLAttributes, HTMLAttributes)];
	},

	addCommands() {
		return {
			setPageBreak:
				() =>
				({ chain }) => {
					return chain().insertContent({ type: this.name }).createParagraphNear().run();
				}
		};
	},

	addKeyboardShortcuts() {
		return {
			'Mod-Enter': () => this.editor.commands.setPageBreak()
		};
	}
});
