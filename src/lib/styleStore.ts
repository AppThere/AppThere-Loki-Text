import { writable } from 'svelte/store';

export interface BlockStyle {
	id: string;
	name: string;
	displayName?: string;
	description: string;
	// Inheritance
	basedOn?: string; // Parent style to inherit from
	next?: string; // Style to apply to next paragraph
	// Font information
	fontFamily?: string;
	fontSize?: string;
	fontWeight?: string;
	// Line spacing
	lineHeight?: string;
	// Indentation
	marginLeft?: string;
	marginRight?: string;
	textIndent?: string;
	// Paragraph spacing
	marginTop?: string;
	marginBottom?: string;
	// Alignment
	textAlign?: 'left' | 'center' | 'right' | 'justify';
	// Text Transformation
	textTransform?: 'uppercase' | 'lowercase' | 'capitalize' | 'none';
	// ODF Meta (not visible in editor)
	hyphenate?: boolean;
	orphans?: number;
	widows?: number;
	outlineLevel?: number; // 1-10

	// Responsive Mobile Overrides (Loki specific)
	mobileMarginLeft?: string;
	mobileMarginRight?: string;

	// Autocomplete
	autocomplete?: boolean;

	// Page Breaks
	breakBefore?: 'auto' | 'page';
	breakAfter?: 'auto' | 'page';
}

const DEFAULT_STYLES: BlockStyle[] = [
	{
		id: 'Normal Text',
		name: 'Normal Text',
		description: 'Standard body text',
		fontFamily: 'Arial, sans-serif',
		fontSize: '11pt',
		lineHeight: '1.15',
		marginTop: '0pt',
		marginBottom: '8pt',
		textAlign: 'left'
	},
	{
		id: 'Title',
		name: 'Title',
		description: 'Document title',
		next: 'Normal Text',
		fontFamily: 'Arial, sans-serif',
		fontSize: '26pt',
		fontWeight: 'normal',
		lineHeight: '1.15',
		marginTop: '0pt',
		marginBottom: '3pt',
		textAlign: 'left'
	},
	{
		id: 'Subtitle',
		name: 'Subtitle',
		description: 'Document subtitle',
		next: 'Normal Text',
		fontFamily: 'Arial, sans-serif',
		fontSize: '15pt',
		fontWeight: 'normal',
		lineHeight: '1.15',
		marginTop: '0pt',
		marginBottom: '10pt',
		textAlign: 'left'
	},
	{
		id: 'Heading 1',
		name: 'Heading 1',
		description: 'Level 1 heading',
		next: 'Normal Text',
		outlineLevel: 1,
		fontFamily: 'Arial, sans-serif',
		fontSize: '20pt',
		fontWeight: 'normal',
		lineHeight: '1.15',
		marginTop: '20pt',
		marginBottom: '6pt',
		textAlign: 'left'
	},
	{
		id: 'Heading 2',
		name: 'Heading 2',
		description: 'Level 2 heading',
		next: 'Normal Text',
		outlineLevel: 2,
		fontFamily: 'Arial, sans-serif',
		fontSize: '16pt',
		fontWeight: 'bold',
		lineHeight: '1.15',
		marginTop: '18pt',
		marginBottom: '6pt',
		textAlign: 'left'
	},
	{
		id: 'Heading 3',
		name: 'Heading 3',
		description: 'Level 3 heading',
		next: 'Normal Text',
		outlineLevel: 3,
		fontFamily: 'Arial, sans-serif',
		fontSize: '14pt',
		fontWeight: 'bold',
		lineHeight: '1.15',
		marginTop: '16pt',
		marginBottom: '4pt',
		textAlign: 'left'
	},
	{
		id: 'Heading 4',
		name: 'Heading 4',
		description: 'Level 4 heading',
		next: 'Normal Text',
		outlineLevel: 4,
		fontFamily: 'Arial, sans-serif',
		fontSize: '12pt',
		fontWeight: 'bold',
		lineHeight: '1.15',
		marginTop: '14pt',
		marginBottom: '4pt',
		textAlign: 'left'
	},
	{
		id: 'Heading 5',
		name: 'Heading 5',
		description: 'Level 5 heading',
		next: 'Normal Text',
		outlineLevel: 5,
		fontFamily: 'Arial, sans-serif',
		fontSize: '11pt',
		fontWeight: 'bold',
		lineHeight: '1.15',
		marginTop: '12pt',
		marginBottom: '4pt',
		textAlign: 'left'
	},
	{
		id: 'Heading 6',
		name: 'Heading 6',
		description: 'Level 6 heading',
		next: 'Normal Text',
		outlineLevel: 6,
		fontFamily: 'Arial, sans-serif',
		fontSize: '11pt',
		fontWeight: 'bold',
		lineHeight: '1.15',
		marginTop: '10pt',
		marginBottom: '4pt',
		textAlign: 'left'
	}
];

// Helper function to filter only defined properties
function filterDefinedProperties(style: BlockStyle): Partial<BlockStyle> {
	const result: any = {};
	const metaKeys = ['id', 'name', 'description', 'basedOn', 'next'];

	for (const [key, value] of Object.entries(style)) {
		if (value !== undefined && !metaKeys.includes(key)) {
			result[key] = value;
		}
	}
	return result;
}

// Resolve style with inheritance
export function resolveStyle(
	styleId: string,
	allStyles: BlockStyle[],
	visited = new Set<string>()
): BlockStyle {
	// Prevent circular references
	if (visited.has(styleId)) {
		console.warn(`Circular style inheritance detected for: ${styleId}`);
		return allStyles.find((s) => s.id === styleId) || getDefaultStyle();
	}

	const style = allStyles.find((s) => s.id === styleId);
	if (!style) {
		return getDefaultStyle();
	}

	// No inheritance - return as is
	if (!style.basedOn) {
		return style;
	}

	// Mark as visited and resolve parent
	visited.add(styleId);
	const baseStyle = resolveStyle(style.basedOn, allStyles, visited);

	// Merge: base properties + current style's overrides
	const merged = {
		...baseStyle,
		...filterDefinedProperties(style),
		id: style.id,
		name: style.name,
		displayName: style.displayName,
		description: style.description,
		basedOn: style.basedOn,
		next: style.next
	};

	return merged;
}

function getDefaultStyle(): BlockStyle {
	return {
		id: 'Normal Text',
		name: 'Normal Text',
		description: 'Standard body text',
		fontFamily: 'Arial, sans-serif',
		fontSize: '11pt',
		lineHeight: '1.15',
		marginTop: '0pt',
		marginBottom: '8pt',
		textAlign: 'left'
	};
}

function createStyleRegistry() {
	const { subscribe, update, set } = writable<BlockStyle[]>(DEFAULT_STYLES);

	return {
		subscribe,
		getStyles: () => {
			let val: BlockStyle[] = [];
			subscribe((s) => (val = s))();
			return val;
		},
		setStyles: (styles: BlockStyle[]) => set(styles),
		addStyle: (style: BlockStyle) => update((styles) => [...styles, style]),
		removeStyle: (id: string) => update((styles) => styles.filter((s) => s.id !== id)),
		updateStyle: (id: string, updated: Partial<BlockStyle>) =>
			update((styles) => styles.map((s) => (s.id === id ? { ...s, ...updated } : s))),
		reset: () => set(DEFAULT_STYLES)
	};
}

export const styleRegistry = createStyleRegistry();
