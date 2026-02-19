import { resolveStyle, type BlockStyle } from '../styleStore';

export function generateEditorStyles(registry: BlockStyle[]): string {
	const baseStyles = registry
		.map((style) => {
			const s = resolveStyle(style.id, registry);
			const rules = [];
			if (s.fontFamily) {
				const font = s.fontFamily.includes(',') ? s.fontFamily : `"${s.fontFamily}"`;
				rules.push(`font-family: ${font};`);
			}
			if (s.fontSize) rules.push(`font-size: ${s.fontSize};`);
			if (s.fontWeight) rules.push(`font-weight: ${s.fontWeight};`);
			if (s.lineHeight) rules.push(`line-height: ${s.lineHeight};`);
			if (s.marginTop) rules.push(`margin-top: ${s.marginTop};`);
			if (s.marginBottom) rules.push(`margin-bottom: ${s.marginBottom};`);
			if (s.marginLeft) rules.push(`margin-left: ${s.marginLeft};`);
			if (s.marginRight) rules.push(`margin-right: ${s.marginRight};`);
			if (s.textIndent) rules.push(`text-indent: ${s.textIndent};`);
			if (s.textAlign) rules.push(`text-align: ${s.textAlign};`);
			if (s.textTransform) rules.push(`text-transform: ${s.textTransform};`);

			if (s.breakBefore === 'page') {
				rules.push('break-before: page;');
				rules.push('margin-top: 3rem;');
				rules.push('position: relative;');
			}
			if (s.breakAfter === 'page') {
				rules.push('break-after: page;');
				rules.push('margin-bottom: 3rem;');
				rules.push('position: relative;');
			}

			let css = `.ProseMirror [data-style-name="${style.id}"] {\n  ${rules.join('\n  ')}\n}`;

			if (s.breakBefore === 'page') {
				css += `\n.ProseMirror [data-style-name="${style.id}"]::before {
                    content: 'Page Break';
                    display: block;
                    width: 100%;
                    border-top: 1px dashed #ccc;
                    margin-bottom: 2rem;
                    position: absolute;
                    top: -1.5rem;
                    left: 0;
                    color: #ccc;
                    font-size: 0.8rem;
                    text-transform: uppercase;
                    text-align: center;
                    pointer-events: none;
                }`;
			}

			if (s.breakAfter === 'page') {
				css += `\n.ProseMirror [data-style-name="${style.id}"]::after {
                    content: 'Page Break';
                    display: block;
                    width: 100%;
                    border-top: 1px dashed #ccc;
                    margin-top: 2rem;
                    position: absolute;
                    bottom: -1.5rem;
                    left: 0;
                    color: #ccc;
                    font-size: 0.8rem;
                    text-transform: uppercase;
                    text-align: center;
                    pointer-events: none;
                }`;
			}

			return css;
		})
		.join('\n');

	const mobileStyles = registry
		.map((style) => {
			const s = resolveStyle(style.id, registry);
			const rules = [];
			if (s.mobileMarginLeft) rules.push(`margin-left: ${s.mobileMarginLeft};`);
			if (s.mobileMarginRight) rules.push(`margin-right: ${s.mobileMarginRight};`);

			if (rules.length > 0) {
				return `.ProseMirror [data-style-name="${style.id}"] {\n  ${rules.join('\n  ')}\n}`;
			}
			return '';
		})
		.filter((css) => css !== '')
		.join('\n');

	return `
${baseStyles}

@media (max-width: 600px) {
${mobileStyles}
}

hr.page-break {
    border: none;
    border-top: 1px dashed #ccc;
    margin: 2rem 0;
    position: relative;
}

hr.page-break::after {
    content: 'Page Break';
    position: absolute;
    top: -0.7em;
    left: 50%;
    transform: translateX(-50%);
    background: var(--bg-color);
    padding: 0 0.5rem;
    color: #ccc;
    font-size: 0.8rem;
    text-transform: uppercase;
}
`;
}
