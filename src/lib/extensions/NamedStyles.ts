import { Extension, Mark, mergeAttributes } from '@tiptap/core';

export const NamedSpanStyle = Mark.create({
	name: 'namedSpanStyle',

	addAttributes() {
		return {
			styleName: {
				default: null,
				parseHTML: (element) => element.getAttribute('data-style-name'),
				renderHTML: (attributes) => {
					if (!attributes.styleName) {
						return {};
					}
					return { 'data-style-name': attributes.styleName };
				}
			}
		};
	},

	parseHTML() {
		return [
			{
				tag: 'span[data-style-name]'
			}
		];
	},

	renderHTML({ HTMLAttributes }) {
		return ['span', mergeAttributes(HTMLAttributes), 0];
	}
});

export const NamedBlockStyle = Extension.create({
	name: 'namedBlockStyle',

	addGlobalAttributes() {
		return [
			{
				types: ['paragraph', 'heading'],
				attributes: {
					styleName: {
						default: null,
						parseHTML: (element) => element.getAttribute('data-style-name'),
						renderHTML: (attributes) => {
							if (!attributes.styleName) {
								return {};
							}
							return { 'data-style-name': attributes.styleName };
						}
					}
				}
			}
		];
	}
});
