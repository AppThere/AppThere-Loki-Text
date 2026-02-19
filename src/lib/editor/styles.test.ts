import { describe, it, expect } from 'vitest';
import { generateEditorStyles } from './styles';
import type { BlockStyle } from '../styleStore';

describe('generateEditorStyles', () => {
	it('should generate base styles for a simple style', () => {
		const styles: BlockStyle[] = [
			{
				id: 'test-style',
				name: 'Test Style',
				fontFamily: 'Arial',
				fontSize: '12pt'
			}
		];

		const css = generateEditorStyles(styles);
		expect(css).toContain('.ProseMirror [data-style-name="test-style"] {');
		expect(css).toContain('font-family: "Arial";');
		expect(css).toContain('font-size: 12pt;');
	});

	it('should handle complex style properties', () => {
		const styles: BlockStyle[] = [
			{
				id: 'complex',
				name: 'Complex',
				marginTop: '10px',
				marginBottom: '20px',
				fontWeight: 'bold',
				textTransform: 'uppercase'
			}
		];

		const css = generateEditorStyles(styles);
		expect(css).toContain('margin-top: 10px;');
		expect(css).toContain('margin-bottom: 20px;');
		expect(css).toContain('font-weight: bold;');
		expect(css).toContain('text-transform: uppercase;');
	});

	it('should generate page break pseudo-elements', () => {
		const styles: BlockStyle[] = [
			{
				id: 'break-before',
				name: 'Break Before',
				breakBefore: 'page'
			},
			{
				id: 'break-after',
				name: 'Break After',
				breakAfter: 'page'
			}
		];

		const css = generateEditorStyles(styles);

		// Break Before
		expect(css).toContain('break-before: page;');
		expect(css).toContain('.ProseMirror [data-style-name="break-before"]::before {');
		expect(css).toContain("content: 'Page Break';");

		// Break After
		expect(css).toContain('break-after: page;');
		expect(css).toContain('.ProseMirror [data-style-name="break-after"]::after {');
	});

	it('should generate mobile styles', () => {
		const styles: BlockStyle[] = [
			{
				id: 'mobile',
				name: 'Mobile',
				mobileMarginLeft: '10px'
			}
		];

		const css = generateEditorStyles(styles);
		expect(css).toContain('@media (max-width: 600px) {');
		expect(css).toContain('.ProseMirror [data-style-name="mobile"] {');
		expect(css).toContain('margin-left: 10px;');
	});

	it('should include static global styles for hr.page-break', () => {
		const css = generateEditorStyles([]);
		expect(css).toContain('hr.page-break {');
		expect(css).toContain('hr.page-break::after {');
	});
});
