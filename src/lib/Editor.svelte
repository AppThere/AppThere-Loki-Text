<script lang="ts">
    import StarterKit from "@tiptap/starter-kit";
    import Underline from "@tiptap/extension-underline";
    import Link from "@tiptap/extension-link";
    import Superscript from "@tiptap/extension-superscript";
    import Subscript from "@tiptap/extension-subscript";
    import Image from "@tiptap/extension-image";
    import { Table } from "@tiptap/extension-table";
    import TableRow from "@tiptap/extension-table-row";
    import TableCell from "@tiptap/extension-table-cell";
    import TableHeader from "@tiptap/extension-table-header";
    import BulletList from "@tiptap/extension-bullet-list";
    import OrderedList from "@tiptap/extension-ordered-list";
    import ListItem from "@tiptap/extension-list-item";
    import Blockquote from "@tiptap/extension-blockquote";
    import TextAlign from "@tiptap/extension-text-align";

    import { createEditor, EditorContent, BubbleMenu } from "svelte-tiptap";
    import { NamedSpanStyle, NamedBlockStyle } from "./extensions/NamedStyles";
    import { NextParagraphStyle } from "./extensions/NextParagraphStyle";
    import { PageBreak } from "./extensions/PageBreak";
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

    import { open } from "@tauri-apps/plugin-dialog";
    import { readFile } from "@tauri-apps/plugin-fs";
    import { convertFileSrc } from "@tauri-apps/api/core";

    import PasteDialog from "./PasteDialog.svelte";
    import StyleDialog from "./StyleDialog.svelte";
    import { styleRegistry, resolveStyle } from "./styleStore";

    let {
        status = $bindable("Ready"),
        currentStyleId = $bindable("Normal Text"),
        metadata = $bindable({
            identifier: "",
            title: "",
            language: "en",
            description: "",
            subject: "",
            creator: "",
            creationDate: "",
            generator: "AppThere Loki",
        }),
        onChange,
    } = $props();

    let baseStyles = $derived(
        $styleRegistry
            .map((style) => {
                const s = resolveStyle(style.id, $styleRegistry);
                const rules = [];
                if (s.fontFamily) {
                    const font = s.fontFamily.includes(",")
                        ? s.fontFamily
                        : `"${s.fontFamily}"`;
                    rules.push(`font-family: ${font};`);
                }
                if (s.fontSize) rules.push(`font-size: ${s.fontSize};`);
                if (s.fontWeight) rules.push(`font-weight: ${s.fontWeight};`);
                if (s.lineHeight) rules.push(`line-height: ${s.lineHeight};`);
                if (s.marginTop) rules.push(`margin-top: ${s.marginTop};`);
                if (s.marginBottom)
                    rules.push(`margin-bottom: ${s.marginBottom};`);
                if (s.marginLeft) rules.push(`margin-left: ${s.marginLeft};`);
                if (s.marginRight)
                    rules.push(`margin-right: ${s.marginRight};`);
                if (s.textIndent) rules.push(`text-indent: ${s.textIndent};`);
                if (s.textAlign) rules.push(`text-align: ${s.textAlign};`);
                if (s.textTransform)
                    rules.push(`text-transform: ${s.textTransform};`);

                if (s.breakBefore === "page") {
                    rules.push("break-before: page;");
                    rules.push("margin-top: 3rem;");
                    rules.push("position: relative;");
                }
                if (s.breakAfter === "page") {
                    rules.push("break-after: page;");
                    rules.push("margin-bottom: 3rem;");
                    rules.push("position: relative;");
                }

                let css = `.ProseMirror [data-style-name="${style.id}"] {\n  ${rules.join("\n  ")}\n}`;

                if (s.breakBefore === "page") {
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

                if (s.breakAfter === "page") {
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
            .join("\n"),
    );

    let mobileStyles = $derived(
        $styleRegistry
            .map((style) => {
                const s = resolveStyle(style.id, $styleRegistry);
                const rules = [];
                if (s.mobileMarginLeft)
                    rules.push(`margin-left: ${s.mobileMarginLeft};`);
                if (s.mobileMarginRight)
                    rules.push(`margin-right: ${s.mobileMarginRight};`);

                if (rules.length > 0) {
                    return `.ProseMirror [data-style-name="${style.id}"] {\n  ${rules.join("\n  ")}\n}`;
                }
                return "";
            })
            .filter((css) => css !== "")
            .join("\n"),
    );

    let dynamicStyles = $derived(`
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
`);

    const editor = createEditor({
        editorProps: {
            handleKeyDown: (view, event) => {
                return handleKeyDown(event);
            },
        },
        extensions: [
            StarterKit.configure({
                bulletList: false, // We import separately to configure if needed, or just use StarterKit's
                orderedList: false,
                listItem: false,
                blockquote: false,
            }),
            // Underline,
            Superscript,
            Subscript,
            // Link.configure({
            //     openOnClick: false,
            //     HTMLAttributes: {
            //         class: "text-blue-600 underline",
            //     },
            // }),
            Image,
            Table.configure({
                resizable: true,
            }),
            TableRow,
            TableHeader,
            TableCell,
            BulletList,
            OrderedList,
            ListItem,
            Blockquote,
            TextAlign.configure({
                types: ["heading", "paragraph"],
            }),
            NamedBlockStyle,
            NextParagraphStyle,
            PageBreak,
        ],
        onCreate({ editor }) {
            // Register getNextStyle helper for the extension
            (window as any).__getNextStyle = (currentStyleId: string) => {
                const style = $styleRegistry.find(
                    (s) => s.id === currentStyleId,
                );
                return style?.next;
            };

            // Initial index build
            buildAutocompleteIndex();

            // Apply default style to initial content if needed
            // This ensures the first paragraph has "Normal Text" style
            if (editor.isEmpty) {
                editor
                    .chain()
                    .updateAttributes("paragraph", { styleName: "Normal Text" })
                    .run();
            }
        },
        onUpdate({ editor }) {
            if (onChange) onChange();
            checkAutocomplete();
        },
        onSelectionUpdate({ editor }) {
            if (!editor) return;
            const { selection } = editor.state;
            const { $from: fromPos } = selection;
            const node = fromPos.node(fromPos.depth);

            if (
                node.type.name === "paragraph" ||
                node.type.name === "heading"
            ) {
                const styleName = node.attrs.styleName;
                if (styleName) {
                    currentStyleId = styleName;
                } else {
                    currentStyleId = "Normal Text";
                }
            }
            checkAutocomplete();
        },
        onDestroy() {
            (window as any).__getNextStyle = undefined;
        },
    });

    // ...

    export const insertImage = async () => {
        try {
            const selected = await open({
                multiple: false,
                filters: [
                    {
                        name: "Images",
                        extensions: ["png", "jpg", "jpeg", "gif", "webp"],
                    },
                ],
            });
            if (selected) {
                const path = Array.isArray(selected) ? selected[0] : selected;

                try {
                    // Strategy: Read file directly and use Data URI to bypass asset:// protocol issues
                    const contents = await readFile(path);

                    // Determine mime type from extension
                    const ext = path.split(".").pop()?.toLowerCase() || "png";
                    const mimeType =
                        ext === "jpg" || ext === "jpeg"
                            ? "image/jpeg"
                            : ext === "gif"
                              ? "image/gif"
                              : ext === "webp"
                                ? "image/webp"
                                : "image/png";

                    // distinct approach for buffer conversion to avoid stack overflow on large files
                    const blob = new Blob([contents], { type: mimeType });
                    const reader = new FileReader();

                    reader.onload = (e) => {
                        const src = e.target?.result as string;
                        console.log("Image loaded as Data URI", {
                            path,
                            length: src.length,
                        });
                        $editor
                            ?.chain()
                            .focus()
                            .setImage({ src, alt: path })
                            .run();
                    };

                    reader.readAsDataURL(blob);
                } catch (readErr) {
                    console.error("Failed to read image file", readErr);
                    // Fallback to old method just in case
                    const src = `asset://localhost${encodeURI(path)}`;
                    $editor?.chain().focus().setImage({ src, alt: path }).run();
                }
            }
        } catch (e) {
            console.error("Failed to insert image", e);
        }
    };

    export const insertTable = () => {
        const rows = prompt("Rows:", "3");
        const cols = prompt("Columns:", "3");
        if (rows && cols) {
            $editor
                ?.chain()
                .focus()
                .insertTable({
                    rows: parseInt(rows),
                    cols: parseInt(cols),
                    withHeaderRow: true,
                })
                .run();
        }
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
        if ($editor?.can().sinkListItem("listItem")) {
            $editor.chain().focus().sinkListItem("listItem").run();
        } else {
            // Fallback: Custom indent implementation?
            // Let's just implement Indent for lists for now as it's standard.
            // Paragraph indent is harder without extension.
        }
    };

    export const outdent = () => {
        if ($editor?.can().liftListItem("listItem")) {
            $editor.chain().focus().liftListItem("listItem").run();
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
            if (s.textTransform)
                attributes["fo:text-transform"] = s.textTransform;
            if (s.hyphenate !== undefined)
                attributes["fo:hyphenate"] = String(s.hyphenate);
            if (s.orphans !== undefined)
                attributes["fo:orphans"] = String(s.orphans);
            if (s.widows !== undefined)
                attributes["fo:widows"] = String(s.widows);
            if (s.breakBefore === "page")
                attributes["fo:break-before"] = "page";
            if (s.breakAfter === "page") attributes["fo:break-after"] = "page";
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
                tiptapJson: JSON.stringify(json),
                styles: getStyleDefinitions(),
                metadata,
            });
            // status = "Saved";
        } catch (e) {
            console.error("Sync failed", e);
            status = "Error";
        }
    }

    import { addDebugLog } from "$lib/debugStore";
    import { writeFile } from "@tauri-apps/plugin-fs"; // Import writeFile

    export const saveWithStyles = async (path: string) => {
        if (!$editor) return;
        status = "Saving...";
        addDebugLog(`Editor.svelte: invoking save_document to ${path}`);
        try {
            // Update type definition if possible, but for now treating result as any to check for array
            const result = await invoke<number[] | null>("save_document", {
                path,
                tiptapJson: JSON.stringify($editor.getJSON()),
                styles: getStyleDefinitions(),
                metadata,
            });

            if (result && Array.isArray(result)) {
                addDebugLog(
                    `Editor.svelte: Received ${result.length} bytes from backend. Writing via plugin-fs...`,
                );
                const data = new Uint8Array(result);
                await writeFile(path, data);
                addDebugLog("Editor.svelte: plugin-fs write success.");
            } else {
                addDebugLog(
                    "Editor.svelte: invoke success (backend handled write)",
                );
            }

            status = "Saved";
        } catch (e) {
            status = "Error saving";
            console.error(e);
            addDebugLog(`Editor.svelte invoke error: ${JSON.stringify(e)}`);
            throw e; // Re-throw so parent knows
        }
    };

    export const loadWithStyles = (data: {
        content: any;
        styles: Record<string, any>;
        metadata: any;
    }) => {
        if (!$editor) return;

        // Convert styles correctly regardless of source (ODF backend vs direct frontend objects)
        const styles: any[] = Object.values(data.styles).map((s: any) => {
            const isOdf = !!s.attributes;
            const attr = isOdf ? s.attributes : s;

            return {
                id: s.id || s.name,
                name: s.name,
                description: s.description || "",
                fontFamily: isOdf ? attr["fo:font-family"] : s.fontFamily,
                fontSize: isOdf ? attr["fo:font-size"] : s.fontSize,
                fontWeight: isOdf ? attr["fo:font-weight"] : s.fontWeight,
                lineHeight: isOdf ? attr["fo:line-height"] : s.lineHeight,
                marginTop: isOdf
                    ? attr["fo:margin-top"] || attr["fo:margin"]
                    : s.marginTop,
                marginBottom: isOdf
                    ? attr["fo:margin-bottom"] || attr["fo:margin"]
                    : s.marginBottom,
                marginLeft: isOdf
                    ? attr["fo:margin-left"] || attr["fo:margin"]
                    : s.marginLeft,
                marginRight: isOdf
                    ? attr["fo:margin-right"] || attr["fo:margin"]
                    : s.marginRight,
                textIndent: isOdf ? attr["fo:text-indent"] : s.textIndent,
                textAlign: isOdf ? attr["fo:text-align"] : s.textAlign,
                textTransform: isOdf
                    ? attr["fo:text-transform"]
                    : s.textTransform,
                breakBefore:
                    (isOdf ? attr["fo:break-before"] : s.breakBefore) || "auto",
                breakAfter:
                    (isOdf ? attr["fo:break-after"] : s.breakAfter) || "auto",
                hyphenate: isOdf
                    ? attr["fo:hyphenate"] === "true"
                    : s.hyphenate,
                orphans: isOdf
                    ? attr["fo:orphans"]
                        ? parseInt(attr["fo:orphans"])
                        : undefined
                    : s.orphans,
                widows: isOdf
                    ? attr["fo:widows"]
                        ? parseInt(attr["fo:widows"])
                        : undefined
                    : s.widows,
                basedOn: isOdf ? s.parent : s.basedOn,
                next: s.next,
                displayName: s.displayName,
            };
        });

        styleRegistry.setStyles(styles);
        metadata = data.metadata;
        $editor.commands.setContent(data.content);

        // Rebuild index after loading content
        setTimeout(() => buildAutocompleteIndex(), 100);
    };

    let isStyleDialogOpen = $state(false);
    let isPasteDialogOpen = $state(false);
    let pendingPasteHtml = $state("");
    let pendingPasteText = $state("");

    export const applyStyle = (styleName: string) => {
        if (!$editor) return;
        if (styleName === "Emphasis") {
            $editor
                .chain()
                .focus()
                .toggleMark("namedSpanStyle", { styleName })
                .run();
        } else {
            $editor
                .chain()
                .focus()
                .updateAttributes("paragraph", { styleName })
                .updateAttributes("heading", { styleName })
                .run();
        }
    };

    export const paste = async () => {
        try {
            const clipboardItems = await navigator.clipboard.read();
            for (const item of clipboardItems) {
                if (item.types.includes("text/html")) {
                    const blob = await item.getType("text/html");
                    pendingPasteHtml = await blob.text();
                    // Also get text version for fallback/plain option
                    if (item.types.includes("text/plain")) {
                        const textBlob = await item.getType("text/plain");
                        pendingPasteText = await textBlob.text();
                    } else {
                        // Fallback: strip tags roughly or use innerText if we parsed it
                        pendingPasteText = pendingPasteHtml.replace(
                            /<[^>]*>?/gm,
                            "",
                        );
                    }
                    isPasteDialogOpen = true;
                    return;
                }
                if (item.types.includes("text/plain")) {
                    const blob = await item.getType("text/plain");
                    const text = await blob.text();
                    $editor?.commands.insertContent(text);
                    return;
                }
            }
        } catch (err) {
            console.error("Failed to read clipboard:", err);
            // Fallback to simple paste if permission denied or API unavailable
            // We can try execCommand, or just alert user
            try {
                const text = await navigator.clipboard.readText();
                if (text) $editor?.commands.insertContent(text);
            } catch (e) {
                console.error("Clipboard API failed completely", e);
            }
        }
    };

    function handlePasteOption(option: "plain" | "structure" | "dirty") {
        if (!$editor) return;

        if (option === "plain") {
            $editor.commands.insertContent(pendingPasteText);
        } else if (option === "dirty") {
            $editor.commands.insertContent(pendingPasteHtml);
        } else if (option === "structure") {
            // Structure: Remove style attributes, classes, and generic divs/spans but keep semantic structure
            const parser = new DOMParser();
            const doc = parser.parseFromString(pendingPasteHtml, "text/html");

            // Remove style attributes
            const elements = doc.body.getElementsByTagName("*");
            for (let i = 0; i < elements.length; i++) {
                elements[i].removeAttribute("style");
                elements[i].removeAttribute("class");
                // We could also unwrap non-semantic tags like span/div here if we wanted strict structure
            }

            $editor.commands.insertContent(doc.body.innerHTML);
        }

        pendingPasteHtml = "";
        pendingPasteText = "";
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

    // Autocomplete Logic
    let autocompleteIndex = $state<Record<string, Set<string>>>({});
    let suggestions = $state<string[]>([]);
    let selectedSuggestionIndex = $state(0);
    let showSuggestions = $state(false);
    let suggestionPosition = $state({ top: 0, left: 0 });
    let suggestionQuery = $state("");

    // Build index from current document content
    function buildAutocompleteIndex() {
        if (!$editor) return;
        const index: Record<string, Set<string>> = {};

        $editor.state.doc.descendants((node) => {
            if (
                node.type.name === "paragraph" ||
                node.type.name === "heading"
            ) {
                const styleName = node.attrs.styleName || "Normal Text";
                const style = resolveStyle(styleName, $styleRegistry);

                if (style.autocomplete) {
                    if (!index[styleName]) index[styleName] = new Set();
                    const text = node.textContent.trim();
                    if (text) index[styleName].add(text);
                }
            }
            return true;
        });
        autocompleteIndex = index;
    }

    // Refresh index periodically or on significant changes?
    // For now, let's refresh when we load content and maybe debounce on updates.

    // Check for suggestions on update
    function checkAutocomplete() {
        if (!$editor) return;
        const { selection } = $editor.state;
        const { $from: fromPos } = selection;
        const node = fromPos.node(fromPos.depth);

        const styleName = node.attrs.styleName || "Normal Text";
        const style = resolveStyle(styleName, $styleRegistry);

        if (!style.autocomplete) {
            showSuggestions = false;
            return;
        }

        // Get text of current block up to cursor
        const textBefore = fromPos.parent.textBetween(
            0,
            fromPos.parentOffset,
            "\n",
            "\uFFFC",
        );
        // Simple strategy: suggest if we have typed at least 1 char and it matches start of known entries
        // Note: This matches the *whole line* prefix. Screenplay Characters/Sluglines are usually short whole lines.

        const query = textBefore.trim();
        suggestionQuery = query;

        if (query.length < 1) {
            showSuggestions = false;
            return;
        }

        const candidates = autocompleteIndex[styleName] || new Set();
        // Filter candidates that start with query but are not exact match (don't suggest if already typed)
        const matches = Array.from(candidates)
            .filter(
                (c) =>
                    c.toLowerCase().startsWith(query.toLowerCase()) &&
                    c.toLowerCase() !== query.toLowerCase(),
            )
            .sort()
            .slice(0, 5); // Limit to 5

        if (matches.length > 0) {
            suggestions = matches;
            selectedSuggestionIndex = 0;

            // Calculate position
            const coords = $editor.view.coordsAtPos(fromPos.pos);
            const editorRect = $editor.view.dom.getBoundingClientRect();

            // Relative to window, but we might want it relative to editor wrapper if we used absolute.
            // Using fixed position for simplicity relative to viewport
            suggestionPosition = {
                top: coords.bottom + 5, // 5px gap
                left: coords.left,
            };
            showSuggestions = true;
        } else {
            showSuggestions = false;
        }
    }

    function acceptSuggestion(suggestion: string) {
        if (!$editor) return;
        const { selection } = $editor.state;
        const { $from: fromPos } = selection;
        // Replace current node text with suggestion? Or just append suffix?
        // Usually trigger-based autocomplete appends.
        // But here we are matching the *whole block content*.
        // So we should probably replace the current text or append the remainder.

        // Let's assume we want to complete the line.
        // Current text: "INT."
        // Suggestion: "INT. OFFICE - DAY"
        // Suffix: " OFFICE - DAY"

        // We can just set the node content to the suggestion.
        const node = fromPos.node(fromPos.depth);
        const startPos = fromPos.start();
        $editor
            .chain()
            .deleteRange({ from: startPos, to: startPos + node.content.size })
            .insertContentAt(startPos, suggestion)
            .run();

        showSuggestions = false;
    }

    // Keyboard handling for suggestions
    function handleKeyDown(event: KeyboardEvent) {
        if (!showSuggestions) return false;

        if (event.key === "ArrowDown") {
            event.preventDefault();
            selectedSuggestionIndex =
                (selectedSuggestionIndex + 1) % suggestions.length;
            return true;
        }
        if (event.key === "ArrowUp") {
            event.preventDefault();
            selectedSuggestionIndex =
                (selectedSuggestionIndex - 1 + suggestions.length) %
                suggestions.length;
            return true;
        }
        if (event.key === "Enter" || event.key === "Tab") {
            event.preventDefault();
            acceptSuggestion(suggestions[selectedSuggestionIndex]);
            return true;
        }
        if (event.key === "Escape") {
            showSuggestions = false;
            return true;
        }
        return false;
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

            {#if showSuggestions}
                <div
                    class="suggestion-menu"
                    style="top: {suggestionPosition.top}px; left: {suggestionPosition.left}px"
                >
                    {#each suggestions as suggestion, i}
                        <div
                            class="suggestion-item"
                            class:selected={i === selectedSuggestionIndex}
                            onmousedown={() => acceptSuggestion(suggestion)}
                            role="button"
                            tabindex="-1"
                        >
                            <span class="suggestion-match"
                                >{suggestionQuery}</span
                            >{suggestion.substring(suggestionQuery.length)}
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
        padding: 20px 24px 60px 24px; /* Reduced top padding, ensure bottom padding */
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
