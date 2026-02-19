export interface Metadata {
	identifier?: string;
	title?: string;
	language?: string;
	description?: string;
	subject?: string;
	creator?: string;
	creationDate?: string;
	generator?: string;
}

export interface StyleDefinition {
	name: string;
	family: 'paragraph' | 'text';
	parent?: string;
	next?: string;
	displayName?: string;
	attributes: Record<string, string>;
	textTransform?: string;
	outlineLevel?: number;
	autocomplete?: boolean;
}

export interface TiptapNode {
	type: string;
	content?: TiptapNode[];
	text?: string;
	attrs?: Record<string, any>;
	marks?: any[];
}

export interface TiptapResponse {
	content: TiptapNode;
	styles: Record<string, StyleDefinition>;
	metadata: Metadata;
}
