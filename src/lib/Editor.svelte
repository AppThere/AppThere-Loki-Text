<script lang="ts">
    import StarterKit from "@tiptap/starter-kit";
    import Underline from "@tiptap/extension-underline";
    import Link from "@tiptap/extension-link";
    import Superscript from "@tiptap/extension-superscript";
    import Subscript from "@tiptap/extension-subscript";
    import { createEditor, EditorContent, BubbleMenu } from "svelte-tiptap";
    import { NamedSpanStyle, NamedBlockStyle } from "./extensions/NamedStyles";
    import { NextParagraphStyle } from "./extensions/NextParagraphStyle";
    import { invoke } from "@tauri-apps/api/core";
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
        X,
    } from "lucide-svelte";

    import StyleDialog from "./StyleDialog.svelte";
    import { styleRegistry, resolveStyle } from "./styleStore";

    let {
        status = $bindable("Ready"),
        currentStyleId = $bindable("Normal Text"),
        metadata = $bindable({
            title: "",
            description: "",
            subject: "",
            creator: "",
            creationDate: "",
            generator: "",
        }),
        onChange,
    } = $props();

    // Dynamic style generation
    let dynamicStyles = $state("");
    styleRegistry.subscribe((styles) => {
        let css = "";
        styles.forEach((style) => {
            // Use resolved style to apply inheritance
            const resolved = resolveStyle(style.id, styles);
            css += `[data-style-name="${style.id}"] {`;
            if (resolved.fontFamily)
                css += `font-family: ${resolved.fontFamily} !important;`;
            if (resolved.fontSize)
                css += `font-size: ${resolved.fontSize} !important;`;
            if (resolved.fontWeight)
                css += `font-weight: ${resolved.fontWeight} !important;`;
            if (resolved.lineHeight)
                css += `line-height: ${resolved.lineHeight} !important;`;
            if (resolved.marginLeft)
                css += `margin-left: ${resolved.marginLeft} !important;`;
            if (resolved.marginRight)
                css += `margin-right: ${resolved.marginRight} !important;`;
            if (resolved.marginTop)
                css += `margin-top: ${resolved.marginTop} !important;`;
            if (resolved.marginBottom)
                css += `margin-bottom: ${resolved.marginBottom} !important;`;
            if (resolved.textIndent)
                css += `text-indent: ${resolved.textIndent} !important;`;
            if (resolved.textAlign)
                css += `text-align: ${resolved.textAlign} !important;`;
            if (resolved.hyphenate !== undefined)
                css += `hyphens: ${resolved.hyphenate ? "auto" : "none"} !important;`;
            // orphans/widows are handled by browser/print styles usually, but we can try
            if (resolved.orphans !== undefined)
                css += `orphans: ${resolved.orphans} !important;`;
            if (resolved.widows !== undefined)
                css += `widows: ${resolved.widows} !important;`;

            css += `}`;
        });
        dynamicStyles = css;
    });

    // Helper function for NextParagraphStyle extension
    (window as any).__getNextStyle = (currentStyleId: string) => {
        const styles = styleRegistry.getStyles();
        const currentStyle = styles.find((s) => s.id === currentStyleId);
        return currentStyle?.next;
    };

    const editor = createEditor({
        extensions: [
            StarterKit,
            Underline,
            Superscript,
            Subscript,
            Link.configure({
                openOnClick: false,
                HTMLAttributes: {
                    class: "text-blue-600 underline",
                },
            }),
            NamedSpanStyle,
            NamedBlockStyle,
            NextParagraphStyle,
        ],
        content: "",
        onUpdate({ editor }) {
            syncDocument(editor.getJSON());
            if (onChange) onChange();
        },
        onSelectionUpdate({ editor }) {
            // Find the parent block and its styleName attribute
            const selectionFrom = editor.state.selection.$from;
            const node = selectionFrom.node(selectionFrom.depth);
            currentStyleId = node.attrs.styleName || "Normal Text";
        },
        editorProps: {
            attributes: {
                class: "focus:outline-none focus:ring-0 mx-auto",
            },
        },
    });

    let isStyleDialogOpen = $state(false);

    export const applyStyle = (styleName: string) => {
        if (!$editor) return;
        if (styleName === "Emphasis") {
            $editor
                .chain()
                .focus()
                .toggleMark("namedSpanStyle", { styleName })
                .run();
        } else {
            // Apply styleName to all selected blocks (paragraphs and headings)
            $editor
                .chain()
                .focus()
                .updateAttributes("paragraph", { styleName })
                .updateAttributes("heading", { styleName })
                .run();
        }
    };

    export const setContent = (content: any) => {
        $editor?.commands.setContent(content);
    };

    export const getJSON = () => {
        return $editor?.getJSON();
    };

    export const openStyles = () => {
        isStyleDialogOpen = true;
    };

    function getStyleDefinitions() {
        let styles: Record<string, any> = {};
        const allStyles = styleRegistry.getStyles();
        allStyles.forEach((s) => {
            let attributes: Record<string, string> = {};
            if (s.fontFamily) attributes["fo:font-family"] = s.fontFamily;
            if (s.fontSize) attributes["fo:font-size"] = s.fontSize;
            if (s.fontWeight) attributes["fo:font-weight"] = s.fontWeight;
            if (s.lineHeight) attributes["fo:line-height"] = s.lineHeight;
            if (s.marginLeft) attributes["fo:margin-left"] = s.marginLeft;
            if (s.marginRight) attributes["fo:margin-right"] = s.marginRight;
            if (s.marginTop) attributes["fo:margin-top"] = s.marginTop;
            if (s.marginBottom) attributes["fo:margin-bottom"] = s.marginBottom;
            if (s.textIndent) attributes["fo:text-indent"] = s.textIndent;
            if (s.textAlign) attributes["fo:text-align"] = s.textAlign;
            if (s.hyphenate !== undefined)
                attributes["fo:hyphenate"] = String(s.hyphenate);
            if (s.orphans !== undefined)
                attributes["fo:orphans"] = String(s.orphans);
            if (s.widows !== undefined)
                attributes["fo:widows"] = String(s.widows);
            if (s.basedOn) attributes["style:parent-style-name"] = s.basedOn;
            if (s.next) attributes["style:next-style-name"] = s.next;

            styles[s.id] = {
                name: s.id,
                family: "Paragraph",
                attributes,
            };
        });
        return styles;
    }

    async function syncDocument(json: any) {
        // status = "Syncing..."; // Don't show confusing status to user
        try {
            await invoke("sync_document", {
                tiptapJson: json,
                styles: getStyleDefinitions(),
                metadata,
            });
            // status = "Saved";
        } catch (e) {
            console.error("Sync failed", e);
            status = "Error";
        }
    }

    export const saveWithStyles = async (path: string) => {
        if (!$editor) return;
        status = "Saving...";
        try {
            await invoke("save_document", {
                path,
                tiptapJson: $editor.getJSON(),
                styles: getStyleDefinitions(),
                metadata,
            });
            status = "Saved";
        } catch (e) {
            status = "Error saving";
            console.error(e);
        }
    };

    export const loadWithStyles = (data: {
        content: any;
        styles: Record<string, any>;
        metadata: any;
    }) => {
        if (!$editor) return;
        // Convert ODF attributes back to BlockStyle
        const styles: any[] = Object.values(data.styles).map((s: any) => {
            const attr = s.attributes;
            return {
                id: s.name,
                name: s.name,
                description: "",
                fontFamily: attr["fo:font-family"],
                fontSize: attr["fo:font-size"],
                fontWeight: attr["fo:font-weight"],
                lineHeight: attr["fo:line-height"],
                marginLeft: attr["fo:margin-left"],
                marginRight: attr["fo:margin-right"],
                marginTop: attr["fo:margin-top"],
                marginBottom: attr["fo:margin-bottom"],
                textIndent: attr["fo:text-indent"],
                textAlign: attr["fo:text-align"],
                hyphenate: attr["fo:hyphenate"] === "true",
                orphans: attr["fo:orphans"]
                    ? parseInt(attr["fo:orphans"])
                    : undefined,
                widows: attr["fo:widows"]
                    ? parseInt(attr["fo:widows"])
                    : undefined,
                basedOn: attr["style:parent-style-name"],
                next: attr["style:next-style-name"],
            };
        });
        styleRegistry.setStyles(styles);
        metadata = data.metadata;
        $editor.commands.setContent(data.content);
    };

    // Link handling
    let linkUrl = $state("");
    let isLinkMode = $state(false);

    function setLink() {
        if (!$editor) return;
        if (linkUrl === "") {
            $editor.chain().focus().extendMarkRange("link").unsetLink().run();
        } else {
            $editor
                .chain()
                .focus()
                .extendMarkRange("link")
                .setLink({ href: linkUrl })
                .run();
        }
        isLinkMode = false;
        linkUrl = "";
    }

    function toggleLinkMode() {
        if (!$editor) return;
        isLinkMode = !isLinkMode;
        if (isLinkMode) {
            linkUrl = $editor.getAttributes("link").href || "";
        }
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
                                    onclick={() =>
                                        $editor
                                            ?.chain()
                                            .focus()
                                            .toggleBold()
                                            .run()}
                                    class:active={$editor.isActive("bold")}
                                    title="Strong"
                                    aria-label="Strong"
                                >
                                    <Bold size={16} />
                                </button>
                                <button
                                    onclick={() =>
                                        $editor
                                            ?.chain()
                                            .focus()
                                            .toggleItalic()
                                            .run()}
                                    class:active={$editor.isActive("italic")}
                                    title="Emphasis"
                                    aria-label="Emphasis"
                                >
                                    <Italic size={16} />
                                </button>
                                <button
                                    onclick={() =>
                                        $editor
                                            ?.chain()
                                            .focus()
                                            .toggleUnderline()
                                            .run()}
                                    class:active={$editor.isActive("underline")}
                                    aria-label="Underline"
                                >
                                    <UnderlineIcon size={16} />
                                </button>
                                <button
                                    onclick={() =>
                                        $editor
                                            ?.chain()
                                            .focus()
                                            .toggleStrike()
                                            .run()}
                                    class:active={$editor.isActive("strike")}
                                    aria-label="Strike"
                                >
                                    <Strikethrough size={16} />
                                </button>
                            </div>

                            <div class="menu-divider"></div>

                            <div class="menu-group">
                                <button
                                    onclick={() =>
                                        $editor
                                            ?.chain()
                                            .focus()
                                            .toggleSuperscript()
                                            .unsetSubscript()
                                            .run()}
                                    class:active={$editor.isActive(
                                        "superscript",
                                    )}
                                    aria-label="Superscript"
                                >
                                    <SuperscriptIcon size={16} />
                                </button>
                                <button
                                    onclick={() =>
                                        $editor
                                            ?.chain()
                                            .focus()
                                            .toggleSubscript()
                                            .unsetSuperscript()
                                            .run()}
                                    class:active={$editor.isActive("subscript")}
                                    aria-label="Subscript"
                                >
                                    <SubscriptIcon size={16} />
                                </button>
                            </div>

                            <div class="menu-divider"></div>

                            <div class="menu-group">
                                <button
                                    onclick={toggleLinkMode}
                                    class:active={$editor.isActive("link")}
                                    aria-label="Link"
                                >
                                    <LinkIcon size={16} />
                                </button>
                            </div>

                            <div class="menu-divider"></div>

                            <div class="menu-group">
                                <button
                                    onclick={() => {
                                        document.execCommand("cut");
                                        $editor?.chain().focus().run();
                                    }}
                                    aria-label="Cut"
                                >
                                    <Scissors size={16} />
                                </button>
                                <button
                                    onclick={() => {
                                        document.execCommand("copy");
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
                                    onkeydown={(e) =>
                                        e.key === "Enter" && setLink()}
                                />
                                <button onclick={setLink} class="link-confirm"
                                    ><Check size={14} /></button
                                >
                                <button
                                    onclick={() => (isLinkMode = false)}
                                    class="link-cancel"><X size={14} /></button
                                >
                            </div>
                        {/if}
                    </div>
                {/if}
            </BubbleMenu>
            <EditorContent editor={$editor} />
            {@html `<style>${dynamicStyles}</style>`}
        {/if}
    </div>
</div>

<StyleDialog
    isOpen={isStyleDialogOpen}
    onSelect={applyStyle}
    onClose={() => (isStyleDialogOpen = false)}
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
        padding: 60px 24px; /* Ensure horizontal padding exists on all screens */
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
        font-family: "Liberation Serif", "Times New Roman", serif;
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
        content: "";
        display: block;
        /* potential for visual indicator */
    }

    :global(.ProseMirror p) {
        margin-top: 0;
        margin-bottom: 0;
        line-height: 1;
    }

    :global(.ProseMirror h1, .ProseMirror h2, .ProseMirror h3) {
        font-family: "Liberation Sans", "Arial", sans-serif;
        margin-top: 1rem;
        margin-bottom: 0.5rem;
        line-height: 1.2;
    }

    :global(.ProseMirror [data-style-name="Emphasis"]),
    :global(.ProseMirror em) {
        font-style: italic !important;
        color: inherit !important;
    }

    :global(.ProseMirror strong),
    :global(.ProseMirror b) {
        font-weight: bold !important;
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
