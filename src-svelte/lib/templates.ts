import type { BlockStyle } from './styleStore';

export interface Template {
	id: string;
	name: string;
	description: string;
	styles: BlockStyle[];
}

export const TEMPLATES: Template[] = [
	{
		id: 'basic',
		name: 'Basic Document',
		description: 'Standard document with headings and paragraphs.',
		styles: [
			// Basic uses the default built-in styles, so we can essentially return an empty list
			// if we assume Editor starts with defaults.
			// However, to be explicit, let's include the standard set or allow the Editor to merge.
			// For now, let's keep it empty to use defaults, or we can redefine them here if we want specific overrides.
			// The prompt implies "Basic template that's the H1-H6 + Normal Text option", which matches our defaults.
		]
	},
	{
		id: 'screenplay',
		name: 'Screenplay',
		description: 'Hollywood standard screenplay format.',
		styles: [
			{
				id: 'Script Title',
				name: 'Script Title',
				description: 'Title of the script',
				next: 'Scene Heading',
				outlineLevel: 1,
				fontFamily: '"Courier Prime", "Courier New", Courier, monospace',
				fontSize: '12pt',
				fontWeight: 'bold',
				textAlign: 'center',
				marginTop: '1.5in',
				marginBottom: '0.5in',
				textTransform: 'uppercase'
			},
			{
				id: 'Scene Heading',
				name: 'Scene Heading',
				displayName: 'Slugline',
				description: 'Scene Location and Time (INT./EXT.)',
				next: 'Action',
				outlineLevel: 2,
				fontFamily: '"Courier Prime", "Courier New", Courier, monospace',
				fontSize: '12pt',
				fontWeight: 'bold',
				textAlign: 'left',
				marginTop: '12pt',
				marginBottom: '12pt',
				textTransform: 'uppercase',
				marginLeft: '0in',
				marginRight: '0in',
				textIndent: '0in',
				autocomplete: true
			},
			{
				id: 'Action',
				name: 'Action',
				description: 'Scene description',
				next: 'Action',
				fontFamily: '"Courier Prime", "Courier New", Courier, monospace',
				fontSize: '12pt',
				fontWeight: 'normal',
				textAlign: 'left',
				marginTop: '12pt',
				marginBottom: '12pt',
				marginLeft: '0in',
				marginRight: '0in'
			},
			{
				id: 'Character',
				name: 'Character',
				description: 'Character Name',
				next: 'Dialogue',
				outlineLevel: 3,
				fontFamily: '"Courier Prime", "Courier New", Courier, monospace',
				fontSize: '12pt',
				fontWeight: 'normal',
				textAlign: 'left',
				marginTop: '12pt',
				marginBottom: '0pt',
				marginLeft: '2.0in',
				marginRight: '0in',
				textTransform: 'uppercase',
				mobileMarginLeft: '1.5in',
				autocomplete: true
			},
			{
				id: 'Dialogue',
				name: 'Dialogue',
				description: 'Character Dialogue',
				next: 'Character',
				fontFamily: '"Courier Prime", "Courier New", Courier, monospace',
				fontSize: '12pt',
				fontWeight: 'normal',
				textAlign: 'left',
				marginTop: '0pt',
				marginBottom: '12pt',
				marginLeft: '1.0in',
				marginRight: '1.5in',
				mobileMarginLeft: '0.5in',
				mobileMarginRight: '0.5in'
			},
			{
				id: 'Parenthetical',
				name: 'Parenthetical',
				description: 'Action within dialogue',
				next: 'Dialogue',
				fontFamily: '"Courier Prime", "Courier New", Courier, monospace',
				fontSize: '12pt',
				fontWeight: 'normal',
				textAlign: 'left',
				marginTop: '0pt',
				marginBottom: '0pt',
				marginLeft: '1.5in',
				marginRight: '2.0in',
				mobileMarginLeft: '1.0in',
				mobileMarginRight: '1.0in'
			},
			{
				id: 'Transition',
				name: 'Transition',
				description: 'Cut to, Fade in, etc.',
				next: 'Scene Heading',
				fontFamily: '"Courier Prime", "Courier New", Courier, monospace',
				fontSize: '12pt',
				fontWeight: 'normal',
				textAlign: 'right',
				marginTop: '12pt',
				marginBottom: '12pt',
				marginRight: '0in',
				textTransform: 'uppercase'
			}
		]
	}
];
