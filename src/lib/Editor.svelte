<script lang="ts">
	import { createEditor, EditorContent, BubbleMenu } from 'svelte-tiptap';
	import { getEditorExtensions } from './editor/config';
	import { NamedSpanStyle, NamedBlockStyle } from './extensions/NamedStyles';
	import { NextParagraphStyle } from './extensions/NextParagraphStyle';
	import { PageBreak } from './extensions/PageBreak';
	import { AutocompleteController } from './editor/autocomplete.svelte';

	const autocomplete = new AutocompleteController();
	import {
		Bold,
		Italic,
		Underline as UnderlineIcon,
		Strikethrough,
		Link as LinkIcon,
		Scissors,
		Copy,
		Superscript as SuperscriptIcon,
		Subscript as SubscriptIcon,
		Check,
		X
	} from 'lucide-svelte';

	// readFile removed, replaced by fileService
	import { fileService } from './services/FileService';

	import PasteDialog from './PasteDialog.svelte';
	import StyleDialog from './StyleDialog.svelte';
	import { styleRegistry, resolveStyle } from './styleStore';

	let {
		status = $bindable('Ready'),
		currentStyleId = $bindable('Normal Text'),
		metadata = $bindable({
			identifier: '',
			title: '',
			language: 'en',
			description: '',
			subject: '',
			creator: '',
			creationDate: '',
			generator: 'AppThere Loki'
		}),
		onChange
	} = $props();

	import { generateEditorStyles } from './editor/styles';

	let dynamicStyles = $derived(generateEditorStyles($styleRegistry));

	const editor = createEditor({
		editorProps: {
			handleKeyDown: (view, event) => {
				return handleKeyDown(event);
			}
		},
		extensions: getEditorExtensions(),
		onCreate({ editor }) {
			// Register getNextStyle helper for the extension
			(window as any).__getNextStyle = (currentStyleId: string) => {
				const style = $styleRegistry.find((s) => s.id === currentStyleId);
				return style?.next;
			};

			// Initial index build
			autocomplete.buildIndex(editor, $styleRegistry);

			if (pendingLoadData) {
				console.log('Editor: applying pendingLoadData in onCreate');
				applyLoadData(pendingLoadData, editor);
				pendingLoadData = null;
			} else if (editor.isEmpty) {
				// Apply default style to initial content if needed
				// This ensures the first paragraph has "Normal Text" style
				editor.chain().updateAttributes('paragraph', { styleName: 'Normal Text' }).run();
			}
		},
		onUpdate({ editor }) {
			if (onChange) onChange();
			autocomplete.check(editor, $styleRegistry);
		},
		onSelectionUpdate({ editor }) {
			if (!editor) return;
			const { selection } = editor.state;
			const { $from: fromPos } = selection;
			const node = fromPos.node(fromPos.depth);

			if (node.type.name === 'paragraph' || node.type.name === 'heading') {
				const styleName = node.attrs.styleName;
				if (styleName) {
					currentStyleId = styleName;
				} else {
					currentStyleId = 'Normal Text';
				}
			}
			autocomplete.check(editor, $styleRegistry);
		},
		onDestroy() {
			(window as any).__getNextStyle = undefined;
		}
	});

	// ...

	import { insertImageAction, insertTableAction } from './editor/actions';

	export const insertImage = async () => {
		if (!$editor) return;
		await insertImageAction($editor, fileService);
	};

	export const insertTable = () => {
		if (!$editor) return;
		insertTableAction($editor);
	};

	export const toggleBulletList = () => {
		$editor?.chain().focus().toggleBulletList().run();
	};

	export const toggleOrderedList = () => {
		$editor?.chain().focus().toggleOrderedList().run();
	};

	export const toggleBlockquote = () => {
		$editor?.chain().focus().toggleBlockquote().run();
	};

	export const insertPageBreak = () => {
		$editor?.chain().focus().setPageBreak().run();
	};

	export const insertHardBreak = () => {
		$editor?.chain().focus().setHardBreak().run();
	};

	export const indent = () => {
		// Tiptap doesn't have default indent. Standard is usually sinkListItem for lists.
		// For paragraphs, we might need a custom indentation extension or just margin-left.
		// Let's try sinkListItem first as it handles lists which is user's likely intent with these buttons context.
		// But for paragraphs?
		if ($editor?.can().sinkListItem('listItem')) {
			$editor.chain().focus().sinkListItem('listItem').run();
		} else {
			// Fallback: Custom indent implementation?
			// Let's just implement Indent for lists for now as it's standard.
			// Paragraph indent is harder without extension.
		}
	};

	export const outdent = () => {
		if ($editor?.can().liftListItem('listItem')) {
			$editor.chain().focus().liftListItem('listItem').run();
		}
	};

	export const undo = () => {
		$editor?.chain().focus().undo().run();
	};

	export const redo = () => {
		$editor?.chain().focus().redo().run();
	};

	function getStyleDefinitions() {
		let styles: Record<string, any> = {};
		const allStyles = styleRegistry.getStyles();
		allStyles.forEach((s) => {
			let attributes: Record<string, string> = {};
			if (s.fontFamily) attributes['fo:font-family'] = s.fontFamily;
			if (s.fontSize) attributes['fo:font-size'] = s.fontSize;
			if (s.fontWeight) attributes['fo:font-weight'] = s.fontWeight;
			if (s.lineHeight) attributes['fo:line-height'] = s.lineHeight;
			if (s.marginLeft) attributes['fo:margin-left'] = s.marginLeft;
			if (s.marginRight) attributes['fo:margin-right'] = s.marginRight;
			if (s.marginTop) attributes['fo:margin-top'] = s.marginTop;
			if (s.marginBottom) attributes['fo:margin-bottom'] = s.marginBottom;
			if (s.textIndent) attributes['fo:text-indent'] = s.textIndent;
			if (s.textAlign) attributes['fo:text-align'] = s.textAlign;
			if (s.textTransform) attributes['fo:text-transform'] = s.textTransform;
			if (s.hyphenate !== undefined) attributes['fo:hyphenate'] = String(s.hyphenate);
			if (s.orphans !== undefined) attributes['fo:orphans'] = String(s.orphans);
			if (s.widows !== undefined) attributes['fo:widows'] = String(s.widows);
			if (s.breakBefore === 'page') attributes['fo:break-before'] = 'page';
			if (s.breakAfter === 'page') attributes['fo:break-after'] = 'page';
			if (s.basedOn) attributes['style:parent-style-name'] = s.basedOn;
			if (s.next) attributes['style:next-style-name'] = s.next;

			styles[s.id] = {
				name: s.id,
				family: 'Paragraph',
				attributes
			};
		});
		return styles;
	}

	async function syncDocument(json: any) {
		// status = "Syncing..."; // Don't show confusing status to user
		try {
			await fileService.syncDocument(JSON.stringify(json), getStyleDefinitions(), metadata);
			// status = "Saved";
		} catch (e) {
			console.error('Sync failed', e);
			status = 'Error';
		}
	}

	import { addDebugLog } from '$lib/debugStore';
	// writeFile import removed

	export const saveWithStyles = async (path: string) => {
		if (!$editor) return;
		status = 'Saving...';
		addDebugLog(`Editor.svelte: delegating save to FileService: ${path}`);
		try {
			await fileService.saveDocument(
				path,
				JSON.stringify($editor.getJSON()),
				getStyleDefinitions(),
				metadata // Note: passed as is
			);
			status = 'Saved';
		} catch (e) {
			status = 'Error saving';
			console.error(e);
			addDebugLog(`Editor.svelte save error: ${JSON.stringify(e)}`);
			throw e; // Re-throw so parent knows
		}
	};

	let pendingLoadData = $state<{
		content: any;
		styles: Record<string, any>;
		metadata: any;
	} | null>(null);

	export const loadWithStyles = (data: {
		content: any;
		styles: Record<string, any>;
		metadata: any;
	}) => {
		if (!$editor) {
			console.log('Editor not ready, queuing loadWithStyles data');
			pendingLoadData = data;
			return;
		}

		applyLoadData(data);
	};

	function applyLoadData(
		data: {
			content: any;
			styles: Record<string, any>;
			metadata: any;
		},
		explicitEditor?: any
	) {
		console.log('Editor: applyLoadData called', {
			hasContent: !!data.content,
			styleCount: Object.keys(data.styles).length,
			hasExplicitEditor: !!explicitEditor
		});

		const e = explicitEditor || $editor;

		if (!e) {
			console.warn('Editor: applyLoadData failed - no editor instance available');
			return;
		}

		// Convert styles correctly regardless of source (ODF backend vs direct frontend objects)
		const styles: any[] = Object.values(data.styles).map((s: any) => {
			const isOdf = !!s.attributes;
			const attr = isOdf ? s.attributes : s;

			return {
				id: s.id || s.name,
				name: s.name,
				description: s.description || '',
				fontFamily: isOdf ? attr['fo:font-family'] : s.fontFamily,
				fontSize: isOdf ? attr['fo:font-size'] : s.fontSize,
				fontWeight: isOdf ? attr['fo:font-weight'] : s.fontWeight,
				lineHeight: isOdf ? attr['fo:line-height'] : s.lineHeight,
				marginTop: isOdf ? attr['fo:margin-top'] || attr['fo:margin'] : s.marginTop,
				marginBottom: isOdf ? attr['fo:margin-bottom'] || attr['fo:margin'] : s.marginBottom,
				marginLeft: isOdf ? attr['fo:margin-left'] || attr['fo:margin'] : s.marginLeft,
				marginRight: isOdf ? attr['fo:margin-right'] || attr['fo:margin'] : s.marginRight,
				textIndent: isOdf ? attr['fo:text-indent'] : s.textIndent,
				textAlign: isOdf ? attr['fo:text-align'] : s.textAlign,
				textTransform: isOdf ? attr['fo:text-transform'] : s.textTransform,
				breakBefore: (isOdf ? attr['fo:break-before'] : s.breakBefore) || 'auto',
				breakAfter: (isOdf ? attr['fo:break-after'] : s.breakAfter) || 'auto',
				hyphenate: isOdf ? attr['fo:hyphenate'] === 'true' : s.hyphenate,
				orphans: isOdf
					? attr['fo:orphans']
						? parseInt(attr['fo:orphans'])
						: undefined
					: s.orphans,
				widows: isOdf ? (attr['fo:widows'] ? parseInt(attr['fo:widows']) : undefined) : s.widows,
				basedOn: isOdf ? s.parent : s.basedOn,
				next: s.next,
				displayName: s.displayName
			};
		});

		styleRegistry.setStyles(styles);
		metadata = data.metadata;

		console.log('Editor: setting content', {
			nodeCount: data.content?.content?.length
		});
		e.commands.setContent(data.content);

		// Rebuild index after loading content
		setTimeout(() => {
			autocomplete.buildIndex(e, $styleRegistry);
		}, 100);
	}

	$effect(() => {
		const e = $editor;
		if (e && pendingLoadData) {
			console.log('Editor: $editor store updated, applying pending load data');
			applyLoadData(pendingLoadData, e);
			pendingLoadData = null;
		}
	});

	let isStyleDialogOpen = $state(false);
	let isPasteDialogOpen = $state(false);
	let pendingPasteHtml = $state('');
	let pendingPasteText = $state('');

	export const applyStyle = (styleName: string) => {
		if (!$editor) return;
		if (styleName === 'Emphasis') {
			$editor.chain().focus().toggleMark('namedSpanStyle', { styleName }).run();
		} else {
			$editor
				.chain()
				.focus()
				.updateAttributes('paragraph', { styleName })
				.updateAttributes('heading', { styleName })
				.run();
		}
	};

	export const paste = async () => {
		try {
			const clipboardItems = await navigator.clipboard.read();
			for (const item of clipboardItems) {
				if (item.types.includes('text/html')) {
					const blob = await item.getType('text/html');
					pendingPasteHtml = await blob.text();
					// Also get text version for fallback/plain option
					if (item.types.includes('text/plain')) {
						const textBlob = await item.getType('text/plain');
						pendingPasteText = await textBlob.text();
					} else {
						// Fallback: strip tags roughly or use innerText if we parsed it
						pendingPasteText = pendingPasteHtml.replace(/<[^>]*>?/gm, '');
					}
					isPasteDialogOpen = true;
					return;
				}
				if (item.types.includes('text/plain')) {
					const blob = await item.getType('text/plain');
					const text = await blob.text();
					$editor?.commands.insertContent(text);
					return;
				}
			}
		} catch (err) {
			console.error('Failed to read clipboard:', err);
			// Fallback to simple paste if permission denied or API unavailable
			// We can try execCommand, or just alert user
			try {
				const text = await navigator.clipboard.readText();
				if (text) $editor?.commands.insertContent(text);
			} catch (e) {
				console.error('Clipboard API failed completely', e);
			}
		}
	};

	function handlePasteOption(option: 'plain' | 'structure' | 'dirty') {
		if (!$editor) return;

		if (option === 'plain') {
			$editor.commands.insertContent(pendingPasteText);
		} else if (option === 'dirty') {
			$editor.commands.insertContent(pendingPasteHtml);
		} else if (option === 'structure') {
			// Structure: Remove style attributes, classes, and generic divs/spans but keep semantic structure
			const parser = new DOMParser();
			const doc = parser.parseFromString(pendingPasteHtml, 'text/html');

			// Remove style attributes
			const elements = doc.body.getElementsByTagName('*');
			for (let i = 0; i < elements.length; i++) {
				elements[i].removeAttribute('style');
				elements[i].removeAttribute('class');
				// We could also unwrap non-semantic tags like span/div here if we wanted strict structure
			}

			$editor.commands.insertContent(doc.body.innerHTML);
		}

		pendingPasteHtml = '';
		pendingPasteText = '';
	}

	export const setContent = (content: any) => {
		$editor?.commands.setContent(content);
	};

	export const getJSON = () => {
		return $editor?.getJSON();
	};

	export const openStyles = () => {
		isStyleDialogOpen = true;
	};

	// Link handling
	let linkUrl = $state('');
	let isLinkMode = $state(false);

	function setLink() {
		if (!$editor) return;
		if (linkUrl === '') {
			$editor.chain().focus().extendMarkRange('link').unsetLink().run();
		} else {
			$editor.chain().focus().extendMarkRange('link').setLink({ href: linkUrl }).run();
		}
		isLinkMode = false;
		linkUrl = '';
	}

	function toggleLinkMode() {
		if (!$editor) return;
		isLinkMode = !isLinkMode;
		if (isLinkMode) {
			linkUrl = $editor.getAttributes('link').href || '';
		}
	}

	// Autocomplete Logic

	// Keyboard handling for suggestions
	function handleKeyDown(event: KeyboardEvent) {
		if (!$editor) return false;
		return autocomplete.handleKeyDown(event, $editor);
	}
</script>

<div class="editor-container">
	<div class="editor-wrapper">
		{#if $editor}
			<BubbleMenu editor={$editor}>
				{#if !$editor.state.selection.empty}
					<div class="bubble-menu">
						{#if !isLinkMode}
							<div class="menu-group">
								<button
									onclick={() => $editor?.chain().focus().toggleBold().run()}
									class:active={$editor.isActive('bold')}
									title="Strong"
									aria-label="Strong"
								>
									<Bold size={16} />
								</button>
								<button
									onclick={() => $editor?.chain().focus().toggleItalic().run()}
									class:active={$editor.isActive('italic')}
									title="Emphasis"
									aria-label="Emphasis"
								>
									<Italic size={16} />
								</button>
								<button
									onclick={() => $editor?.chain().focus().toggleUnderline().run()}
									class:active={$editor.isActive('underline')}
									aria-label="Underline"
								>
									<UnderlineIcon size={16} />
								</button>
								<button
									onclick={() => $editor?.chain().focus().toggleStrike().run()}
									class:active={$editor.isActive('strike')}
									aria-label="Strike"
								>
									<Strikethrough size={16} />
								</button>
							</div>

							<div class="menu-divider"></div>

							<div class="menu-group">
								<button
									onclick={() =>
										$editor?.chain().focus().toggleSuperscript().unsetSubscript().run()}
									class:active={$editor.isActive('superscript')}
									aria-label="Superscript"
								>
									<SuperscriptIcon size={16} />
								</button>
								<button
									onclick={() =>
										$editor?.chain().focus().toggleSubscript().unsetSuperscript().run()}
									class:active={$editor.isActive('subscript')}
									aria-label="Subscript"
								>
									<SubscriptIcon size={16} />
								</button>
							</div>

							<div class="menu-divider"></div>

							<div class="menu-group">
								<button
									onclick={toggleLinkMode}
									class:active={$editor.isActive('link')}
									aria-label="Link"
								>
									<LinkIcon size={16} />
								</button>
							</div>

							<div class="menu-divider"></div>

							<div class="menu-group">
								<button
									onclick={() => {
										document.execCommand('cut');
										$editor?.chain().focus().run();
									}}
									aria-label="Cut"
								>
									<Scissors size={16} />
								</button>
								<button
									onclick={() => {
										document.execCommand('copy');
										$editor?.chain().focus().run();
									}}
									aria-label="Copy"
								>
									<Copy size={16} />
								</button>
							</div>
						{:else}
							<div class="link-input-container">
								<input
									type="text"
									bind:value={linkUrl}
									placeholder="https://..."
									onkeydown={(e) => e.key === 'Enter' && setLink()}
								/>
								<button onclick={setLink} class="link-confirm"><Check size={14} /></button>
								<button onclick={() => (isLinkMode = false)} class="link-cancel"
									><X size={14} /></button
								>
							</div>
						{/if}
					</div>
				{/if}
			</BubbleMenu>
			<EditorContent editor={$editor} />
			{@html `<style>${dynamicStyles}</style>`}

			{#if autocomplete.showSuggestions}
				<div
					class="suggestion-menu"
					style="top: {autocomplete.suggestionPosition.top}px; left: {autocomplete
						.suggestionPosition.left}px"
				>
					{#each autocomplete.suggestions as suggestion, i}
						<div
							class="suggestion-item"
							class:selected={i === autocomplete.selectedSuggestionIndex}
							onmousedown={() => autocomplete.accept($editor!)}
							role="button"
							tabindex="-1"
						>
							<span class="suggestion-match">{autocomplete.suggestionQuery}</span
							>{suggestion.substring(autocomplete.suggestionQuery.length)}
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

<StyleDialog
	isOpen={isStyleDialogOpen}
	onSelect={applyStyle}
	onClose={() => (isStyleDialogOpen = false)}
/>

<PasteDialog
	isOpen={isPasteDialogOpen}
	onSelect={handlePasteOption}
	onClose={() => (isPasteDialogOpen = false)}
/>

<style>
	.editor-container {
		width: 100%;
		display: flex;
		justify-content: center;
		padding-bottom: 100px; /* Space for bottom toolbar */
	}

	.editor-wrapper {
		width: 100%;
		max-width: 800px;
		background: transparent; /* Remove paper background */
		padding: 20px 24px 60px 24px; /* Reduced top padding, ensure large bottom margin */
		min-height: 200px;
		height: auto;
		box-shadow: none; /* Remove paper shadow */
		border-radius: 0;
		text-align: left;
		box-sizing: border-box; /* Crucial for padding to not exceed 100% width */
	}

	/* Remove the specific dark mode override for paper since it's now transparent */
	@media (prefers-color-scheme: dark) {
		.editor-wrapper {
			background: transparent;
			color: #e5e7eb;
			box-shadow: none;
		}
		:global(.ProseMirror) {
			color: #e5e7eb !important;
		}
	}

	:global(.ProseMirror) {
		outline: none;
		min-height: 100px;
		font-family: 'Liberation Serif', 'Times New Roman', serif;
		line-height: 1.5; /* Improve readability for continuous flow */
		color: var(--text-color); /* Use variable */
	}

	/* Page Break / Horizontal Rule Styling */
	:global(.ProseMirror hr) {
		border: none;
		border-top: 2px dashed var(--border-color);
		margin: 2rem 0;
		position: relative;
	}

	/* Optional: Add a "Page Break" label or icon if desired, but dashed line is what was asked */
	:global(.ProseMirror hr::after) {
		content: '';
		display: block;
		/* potential for visual indicator */
	}

	:global(.ProseMirror p) {
		margin-top: 0;
		margin-bottom: 0;
		line-height: 1;
	}

	:global(.ProseMirror h1, .ProseMirror h2, .ProseMirror h3) {
		font-family: 'Liberation Sans', 'Arial', sans-serif;
		margin-top: 1rem;
		margin-bottom: 0.5rem;
		line-height: 1.2;
	}

	:global(.ProseMirror [data-style-name='Emphasis']),
	:global(.ProseMirror em) {
		font-style: italic !important;
		color: inherit !important;
	}

	:global(.ProseMirror strong),
	:global(.ProseMirror b) {
		font-weight: bold !important;
	}

	:global(.ProseMirror img) {
		max-width: 100%;
		height: auto;
		display: block; /* optional, but good for layout */
	}

	.bubble-menu {
		display: flex;
		background-color: var(--header-bg);
		padding: 4px;
		border-radius: 8px;
		box-shadow: var(--shadow-md);
		border: 1px solid var(--border-color);
		gap: 2px;
		animation: bubble-fade-in 0.2s ease-out;
	}

	/* Autocomplete Styles */
	.suggestion-menu {
		position: fixed; /* Fixed relative to viewport, coordsAtPos is viewport relative */
		background: var(--bg-color);
		border: 1px solid var(--border-color);
		border-radius: 6px;
		box-shadow: var(--shadow-md);
		z-index: 1000;
		min-width: 200px;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.suggestion-item {
		padding: 6px 12px;
		cursor: pointer;
		font-family: inherit;
		font-size: 0.9em;
		color: var(--text-color);
	}

	.suggestion-item.selected {
		background: var(--primary-color);
		color: white;
	}

	.suggestion-match {
		font-weight: bold;
	}

	@keyframes bubble-fade-in {
		from {
			opacity: 0;
			transform: scale(0.95) translateY(10px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	.menu-group {
		display: flex;
		gap: 2px;
	}

	.menu-divider {
		width: 1px;
		background-color: var(--border-color);
		margin: 4px 2px;
	}

	.bubble-menu button {
		background: transparent;
		border: none;
		color: var(--icon-color);
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 4px;
		cursor: pointer;
		font-family: inherit;
		font-size: 0.9rem;
		transition: all 0.1s;
	}

	.bubble-menu button:hover {
		background-color: var(--hover-bg);
		color: var(--text-color);
	}

	.bubble-menu button.active {
		background-color: var(--primary-color);
		color: white;
	}

	.link-input-container {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
	}

	.link-input-container input {
		background: var(--bg-color);
		border: 1px solid var(--border-color);
		color: var(--text-color);
		padding: 4px 8px;
		border-radius: 4px;
		outline: none;
		font-size: 0.85rem;
		width: 180px;
	}

	.link-input-container input:focus {
		border-color: var(--primary-color);
	}

	.link-confirm {
		background: var(--primary-color) !important;
		width: auto !important;
		padding: 0 12px !important;
		font-size: 0.8rem !important;
	}

	.link-cancel {
		width: 24px !important;
		height: 24px !important;
		font-size: 0.75rem !important;
	}

	:global(.ProseMirror a) {
		color: var(--primary-color);
		text-decoration: underline;
		cursor: pointer;
	}
</style>
