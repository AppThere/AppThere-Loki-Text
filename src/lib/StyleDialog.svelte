<script lang="ts">
    import { styleRegistry, type BlockStyle } from "./styleStore";
    import {
        Plus,
        Trash2,
        X,
        CaseSensitive,
        AlignJustify,
        Layers,
    } from "lucide-svelte";
    import { fade, scale } from "svelte/transition";

    let { isOpen = false, onSelect, onClose } = $props();

    let selectedStyleId = $state("Normal Text");
    let isCreating = $state(false);
    let showDeleteConfirm = $state(false);
    let activeTab = $state("general");

    // Form fields
    let newName = $state("");
    let newDesc = $state("");
    let fontFamily = $state("");
    let fontSize = $state("");
    let fontWeight = $state(400); // Numeric weight
    let lineHeight = $state("");
    let marginLeft = $state("");
    let marginRight = $state("");
    let textIndent = $state("");
    let marginTop = $state("");
    let marginBottom = $state("");
    let textAlign = $state<BlockStyle["textAlign"]>("left");
    let hyphenate = $state(false);
    let orphans = $state<number | undefined>(undefined);
    let widows = $state<number | undefined>(undefined);
    let basedOn = $state<string | undefined>(undefined);
    let nextStyle = $state<string | undefined>(undefined);
    let outlineLevel = $state<number | undefined>(undefined);

    // Common Google Fonts & Web Safe Fonts
    const COMMON_FONTS = [
        "Arial, sans-serif",
        "Helvetica, sans-serif",
        "Times New Roman, serif",
        "Courier New, monospace",
        "Verdana, sans-serif",
        "Georgia, serif",
        "Palatino, serif",
        "Garamond, serif",
        "Bookman, serif",
        "Comic Sans MS, cursive",
        "Trebuchet MS, sans-serif",
        "Arial Black, sans-serif",
        "Impact, sans-serif",
        "Roboto, sans-serif",
        "Open Sans, sans-serif",
        "Lato, sans-serif",
        "Montserrat, sans-serif",
        "Oswald, sans-serif",
        "Source Sans Pro, sans-serif",
        "Slabo 27px, serif",
        "Raleway, sans-serif",
        "PT Sans, sans-serif",
        "Merriweather, serif",
        "Atkinson Hyperlegible Next, sans-serif",
    ];

    $effect(() => {
        if (isOpen && !isCreating) {
            // Load selected style data
            const style = $styleRegistry.find((s) => s.id === selectedStyleId);
            if (style) {
                loadStyle(style);
            } else {
                // Fallback if selected style was deleted
                selectedStyleId = "Normal Text";
            }
        }
    });

    function loadStyle(style: BlockStyle) {
        newName = style.name;
        newDesc = style.description;
        fontFamily = style.fontFamily || "";
        fontSize = style.fontSize || "";
        const weight = parseInt(style.fontWeight || "400");
        fontWeight = isNaN(weight)
            ? style.fontWeight === "bold"
                ? 700
                : 400
            : weight;
        lineHeight = style.lineHeight || "";
        marginLeft = style.marginLeft || "";
        marginRight = style.marginRight || "";
        textIndent = style.textIndent || "";
        marginTop = style.marginTop || "";
        marginBottom = style.marginBottom || "";
        textAlign = style.textAlign || "left";
        hyphenate = style.hyphenate || false;
        orphans = style.orphans;
        widows = style.widows;
        basedOn = style.basedOn;
        nextStyle = style.next;
        outlineLevel = style.outlineLevel;
    }
    function startCreate() {
        isCreating = true;
        newName = "New Style";
        newDesc = "";
        fontFamily = "Atkinson Hyperlegible Next, sans-serif";
        fontSize = "12pt";
        fontWeight = 400;
        lineHeight = "1.2";
        marginLeft = "0pt";
        marginRight = "0pt";
        textIndent = "0pt";
        marginTop = "0pt";
        marginBottom = "10pt";
        textAlign = "left";
        hyphenate = false;
        orphans = 2;
        widows = 2;
        basedOn = "Normal Text";
        nextStyle = undefined;
        outlineLevel = undefined;
        activeTab = "general";
    }

    function cancelCreate() {
        isCreating = false;
        const style = $styleRegistry.find((s) => s.id === selectedStyleId);
        if (style) loadStyle(style);
    }

    function saveStyle() {
        const styleData: Partial<BlockStyle> = {
            name: newName,
            description: newDesc,
            fontFamily,
            fontSize,
            fontWeight: fontWeight.toString(),
            lineHeight,
            marginLeft,
            marginRight,
            textIndent,
            marginTop,
            marginBottom,
            textAlign,
            hyphenate,
            orphans,
            widows,
            basedOn,
            next: nextStyle,
            outlineLevel,
        };

        if (isCreating) {
            const id = `Style-${Date.now()}`;
            styleRegistry.addStyle({
                id,
                ...styleData,
            } as BlockStyle);
            isCreating = false;
            selectedStyleId = id; // Select new style
        } else {
            styleRegistry.updateStyle(selectedStyleId, styleData);
        }
    }

    function requestDelete() {
        if (selectedStyleId === "Normal Text") return;
        showDeleteConfirm = true;
    }

    function confirmDelete() {
        styleRegistry.removeStyle(selectedStyleId);
        selectedStyleId = "Normal Text";
        showDeleteConfirm = false;
    }

    let previewStyle = $derived({
        fontFamily: fontFamily,
        fontSize: fontSize,
        fontWeight: fontWeight,
        lineHeight: lineHeight,
        textAlign: textAlign,
    });

    // Google Fonts Loader Logic
    const GOOGLE_FONTS = new Set([
        "Roboto",
        "Open Sans",
        "Lato",
        "Montserrat",
        "Oswald",
        "Source Sans Pro",
        "Slabo 27px",
        "Raleway",
        "PT Sans",
        "Merriweather",
    ]);

    let googleFontUrl = $derived.by(() => {
        if (!fontFamily) return null;
        const firstFamily = fontFamily
            .split(",")[0]
            .trim()
            .replace(/['"]/g, "");

        if (GOOGLE_FONTS.has(firstFamily)) {
            const weights = [400];
            if (fontWeight !== 400)
                weights.push(
                    typeof fontWeight === "string"
                        ? parseInt(fontWeight)
                        : (fontWeight as number),
                );
            const uniqueWeights = [...new Set(weights)].join(";");
            return `https://fonts.googleapis.com/css2?family=${firstFamily.replace(/ /g, "+")}:wght@${uniqueWeights}&display=swap`;
        }
        return null;
    });

    function normalizeUnit(
        value: string,
        type: "length" | "lineHeight" = "length",
    ): string {
        const trimmed = value.trim();
        if (!trimmed) return "";
        if (!isNaN(Number(trimmed))) {
            const num = Number(trimmed);
            if (type === "lineHeight") {
                return num < 4 ? String(num) : `${num}pt`;
            } else {
                return `${num}pt`;
            }
        }
        return trimmed;
    }
</script>

<svelte:head>
    {#if googleFontUrl}
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
            rel="preconnect"
            href="https://fonts.gstatic.com"
            crossorigin="anonymous"
        />
        <link href={googleFontUrl} rel="stylesheet" />
    {/if}
</svelte:head>

{#if isOpen}
    <div
        class="modal-overlay"
        onclick={onClose}
        onkeydown={(e) => {
            if (e.key === "Escape" || e.key === "Enter" || e.key === " ") {
                onClose();
            }
        }}
        transition:fade={{ duration: 150 }}
        role="button"
        tabindex="0"
        aria-label="Close dialog"
    >
        <div
            class="modal-content"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-title"
            tabindex="-1"
            transition:scale={{ duration: 200, start: 0.95 }}
        >
            <header class="modal-header">
                <div class="header-main">
                    <h2 id="modal-title">Styles</h2>
                    <div class="style-selector-container">
                        {#if isCreating}
                            <div class="creating-badge">New Style</div>
                        {:else}
                            <div class="selector-wrapper">
                                <select
                                    bind:value={selectedStyleId}
                                    class="style-select-header"
                                >
                                    {#each $styleRegistry as style}
                                        <option value={style.id}
                                            >{style.displayName ||
                                                style.name}</option
                                        >
                                    {/each}
                                </select>
                                <div class="header-actions">
                                    <button
                                        class="icon-btn-header"
                                        onclick={startCreate}
                                        title="New Style"
                                    >
                                        <Plus size={18} />
                                    </button>
                                    <button
                                        class="icon-btn-header delete"
                                        onclick={requestDelete}
                                        disabled={selectedStyleId ===
                                            "Normal Text"}
                                        title="Delete Style"
                                    >
                                        <Trash2 size={18} />
                                    </button>
                                </div>
                            </div>
                        {/if}
                    </div>
                </div>
                <button
                    class="header-close-btn"
                    onclick={onClose}
                    aria-label="Close"
                >
                    <X size={20} />
                </button>
            </header>

            <div class="tabs-nav">
                <button
                    class="tab-btn"
                    class:active={activeTab === "general"}
                    onclick={() => (activeTab = "general")}
                >
                    <Layers size={16} />
                    <span>General</span>
                </button>
                <button
                    class="tab-btn"
                    class:active={activeTab === "typography"}
                    onclick={() => (activeTab = "typography")}
                >
                    <CaseSensitive size={16} />
                    <span>Type</span>
                </button>
                <button
                    class="tab-btn"
                    class:active={activeTab === "paragraph"}
                    onclick={() => (activeTab = "paragraph")}
                >
                    <AlignJustify size={16} />
                    <span>Paragraph</span>
                </button>
            </div>

            <div class="main-body">
                {#if isCreating && activeTab === "general"}
                    <div class="create-notice">
                        <button class="cancel-link" onclick={cancelCreate}
                            >Cancel new style</button
                        >
                    </div>
                {/if}

                <div class="form-container">
                    {#if activeTab === "general"}
                        <div class="form-group">
                            <label for="name">Style Name</label>
                            <input
                                id="name"
                                bind:value={newName}
                                placeholder="e.g., Block Quote"
                            />
                        </div>
                        <div class="form-row">
                            <div class="form-group grow">
                                <label for="basedOn">Inherit From</label>
                                <select id="basedOn" bind:value={basedOn}>
                                    <option value={undefined}>None</option>
                                    {#each $styleRegistry as s}
                                        {#if isCreating || s.id !== selectedStyleId}
                                            <option value={s.id}
                                                >{s.displayName ||
                                                    s.name}</option
                                            >
                                        {/if}
                                    {/each}
                                </select>
                            </div>
                            <div class="form-group grow">
                                <label for="nextStyle">Next Paragraph</label>
                                <select id="nextStyle" bind:value={nextStyle}>
                                    <option value={undefined}>Same Style</option
                                    >
                                    {#each $styleRegistry as s}
                                        <option value={s.id}
                                            >{s.displayName || s.name}</option
                                        >
                                    {/each}
                                </select>
                            </div>
                        </div>
                        <div class="form-group">
                            <label for="desc">Description (Optional)</label>
                            <input
                                id="desc"
                                bind:value={newDesc}
                                placeholder="Purpose of this style..."
                            />
                        </div>
                        <div class="form-group">
                            <label for="outlineLevel"
                                >Semantic Level (Outline Level)</label
                            >
                            <select id="outlineLevel" bind:value={outlineLevel}>
                                <option value={undefined}
                                    >None (Body Text)</option
                                >
                                <option value={1}
                                    >Level 1 (Heading 1 / Title)</option
                                >
                                <option value={2}
                                    >Level 2 (Heading 2 / Slugline)</option
                                >
                                <option value={3}
                                    >Level 3 (Heading 3 / Character)</option
                                >
                                <option value={4}>Level 4 (Heading 4)</option>
                                <option value={5}>Level 5 (Heading 5)</option>
                                <option value={6}>Level 6 (Heading 6)</option>
                                <option value={7}>Level 7</option>
                                <option value={8}>Level 8</option>
                                <option value={9}>Level 9</option>
                                <option value={10}>Level 10</option>
                            </select>
                            <p class="field-help">
                                Used for document navigation and Table of
                                Contents.
                            </p>
                        </div>
                    {:else if activeTab === "typography"}
                        <div class="form-group">
                            <label for="fontFamily">Font Family</label>
                            <input
                                list="fonts"
                                id="fontFamily"
                                bind:value={fontFamily}
                            />
                            <datalist id="fonts">
                                {#each COMMON_FONTS as font}
                                    <option value={font}></option>
                                {/each}
                            </datalist>
                        </div>
                        <div class="form-row">
                            <div class="form-group grow">
                                <label for="fontSize">Size</label>
                                <input
                                    id="fontSize"
                                    bind:value={fontSize}
                                    onblur={() =>
                                        (fontSize = normalizeUnit(fontSize))}
                                />
                            </div>
                            <div class="form-group grow">
                                <label for="textAlign">Alignment</label>
                                <select id="textAlign" bind:value={textAlign}>
                                    <option value="left">Left</option>
                                    <option value="center">Center</option>
                                    <option value="right">Right</option>
                                    <option value="justify">Justified</option>
                                </select>
                            </div>
                        </div>
                        <div class="form-group">
                            <label for="fontWeight">Weight: {fontWeight}</label>
                            <input
                                type="range"
                                id="fontWeight"
                                min="100"
                                max="900"
                                step="100"
                                bind:value={fontWeight}
                            />
                            <div class="range-labels">
                                <span>Light</span>
                                <span>Normal</span>
                                <span>Bold</span>
                            </div>
                        </div>
                    {:else if activeTab === "paragraph"}
                        <div class="form-row">
                            <div class="form-group grow">
                                <label for="lineHeight">Line Spacing</label>
                                <input
                                    id="lineHeight"
                                    bind:value={lineHeight}
                                    onblur={() =>
                                        (lineHeight = normalizeUnit(
                                            lineHeight,
                                            "lineHeight",
                                        ))}
                                />
                            </div>
                            <div class="form-group grow">
                                <label for="textIndent">First Line Indent</label
                                >
                                <input
                                    id="textIndent"
                                    bind:value={textIndent}
                                    onblur={() =>
                                        (textIndent =
                                            normalizeUnit(textIndent))}
                                />
                            </div>
                        </div>
                        <div class="form-row">
                            <div class="form-group grow">
                                <label for="marginTop">Space Above</label>
                                <input
                                    id="marginTop"
                                    bind:value={marginTop}
                                    onblur={() =>
                                        (marginTop = normalizeUnit(marginTop))}
                                />
                            </div>
                            <div class="form-group grow">
                                <label for="marginBottom">Space Below</label>
                                <input
                                    id="marginBottom"
                                    bind:value={marginBottom}
                                    onblur={() =>
                                        (marginBottom =
                                            normalizeUnit(marginBottom))}
                                />
                            </div>
                        </div>
                        <div class="form-row">
                            <div class="form-group grow">
                                <label for="orphans">Orphans</label>
                                <input
                                    type="number"
                                    id="orphans"
                                    bind:value={orphans}
                                    min="1"
                                    max="10"
                                />
                            </div>
                            <div class="form-group grow">
                                <label for="widows">Widows</label>
                                <input
                                    type="number"
                                    id="widows"
                                    bind:value={widows}
                                    min="1"
                                    max="10"
                                />
                            </div>
                        </div>
                        <div class="checkbox-group">
                            <input
                                type="checkbox"
                                id="hyphenate"
                                bind:checked={hyphenate}
                            />
                            <label for="hyphenate">Enable hyphenation</label>
                        </div>
                    {/if}
                </div>
            </div>

            <footer class="modal-footer">
                <div class="preview-area">
                    <span class="preview-tag">Preview</span>
                    <div
                        class="preview-box"
                        style:font-family={previewStyle.fontFamily}
                        style:font-size={previewStyle.fontSize}
                        style:font-weight={previewStyle.fontWeight}
                        style:line-height={previewStyle.lineHeight}
                        style:text-align={previewStyle.textAlign}
                    >
                        The quick brown fox jumps over the lazy dog.
                    </div>
                </div>
                <button class="save-btn" onclick={saveStyle}>
                    {isCreating ? "Create Style" : "Save Changes"}
                </button>
            </footer>
        </div>
    </div>

    {#if showDeleteConfirm}
        <div
            class="modal-overlay confirm-overlay"
            onclick={() => (showDeleteConfirm = false)}
            onkeydown={(e) => {
                if (e.key === "Escape" || e.key === "Enter" || e.key === " ") {
                    showDeleteConfirm = false;
                }
            }}
            role="button"
            tabindex="0"
            aria-label="Cancel delete"
        >
            <div
                class="confirm-box"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
                role="alertdialog"
                tabindex="-1"
            >
                <h3>Delete Style?</h3>
                <p>Are you sure you want to delete "{newName}"?</p>
                <div class="confirm-actions">
                    <button
                        class="cancel-btn"
                        onclick={() => (showDeleteConfirm = false)}
                        >Cancel</button
                    >
                    <button class="delete-btn" onclick={confirmDelete}
                        >Delete</button
                    >
                </div>
            </div>
        </div>
    {/if}
{/if}

<style>
    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
        backdrop-filter: blur(4px);
        padding: 20px;
    }

    .modal-content {
        background: var(--header-bg);
        width: 100%;
        max-width: 520px;
        border-radius: 16px;
        box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2);
        display: flex;
        flex-direction: column;
        max-height: calc(100vh - 40px);
        overflow: hidden;
        color: var(--text-color);
        border: 1px solid var(--border-color);
    }

    /* Header Styling */
    .modal-header {
        padding: 12px 16px;
        border-bottom: 1px solid var(--border-color);
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: var(--bg-color);
    }

    .header-main {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 16px;
        min-width: 0;
    }

    .header-main h2 {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 700;
        color: var(--text-color);
        white-space: nowrap;
    }

    .style-selector-container {
        flex: 1;
        min-width: 0;
    }

    .selector-wrapper {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .style-select-header {
        flex: 1;
        min-width: 0;
        padding: 6px 10px;
        border-radius: 8px;
        border: 1px solid var(--border-color);
        background: var(--header-bg);
        color: var(--text-color);
        font-size: 0.9rem;
        cursor: pointer;
    }

    .header-actions {
        display: flex;
        gap: 4px;
    }

    .icon-btn-header {
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--border-color);
        background: var(--header-bg);
        border-radius: 8px;
        cursor: pointer;
        color: var(--icon-color);
        transition: all 0.2s;
    }

    .icon-btn-header:hover:not(:disabled) {
        background: var(--hover-bg);
        color: var(--text-color);
        border-color: var(--icon-color);
    }

    .icon-btn-header.delete:hover:not(:disabled) {
        color: #ef4444;
        border-color: #ef4444;
    }

    .icon-btn-header:disabled {
        opacity: 0.3;
        cursor: not-allowed;
    }

    .header-close-btn {
        background: none;
        border: none;
        color: var(--icon-color);
        cursor: pointer;
        padding: 4px;
        border-radius: 6px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .header-close-btn:hover {
        background: var(--hover-bg);
        color: var(--text-color);
    }

    .creating-badge {
        background: var(--primary-color);
        color: white;
        padding: 4px 12px;
        border-radius: 12px;
        font-size: 0.75rem;
        font-weight: 700;
        text-transform: uppercase;
        display: inline-block;
    }

    /* Tabs Styling */
    .tabs-nav {
        display: flex;
        padding: 4px;
        background: var(--bg-color);
        border-bottom: 1px solid var(--border-color);
    }

    .tab-btn {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        padding: 10px;
        background: transparent;
        border: none;
        border-radius: 8px;
        color: var(--icon-color);
        font-size: 0.85rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }

    .tab-btn:hover {
        background: var(--hover-bg);
        color: var(--text-color);
    }

    .tab-btn.active {
        background: var(--header-bg);
        color: var(--primary-color);
        box-shadow: var(--shadow-sm);
    }

    /* Body Styling */
    .main-body {
        flex: 1;
        padding: 20px;
        overflow-y: auto;
        min-height: 0;
        background: var(--header-bg);
    }

    .create-notice {
        margin-bottom: 16px;
        text-align: right;
    }

    .cancel-link {
        background: none;
        border: none;
        color: #ef4444;
        font-size: 0.85rem;
        text-decoration: underline;
        cursor: pointer;
    }

    .form-container {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .form-row {
        display: flex;
        gap: 16px;
    }

    .grow {
        flex: 1;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .form-group label {
        font-size: 0.75rem;
        font-weight: 700;
        color: var(--icon-color);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .form-group input,
    .form-group select {
        padding: 10px 12px;
        border-radius: 8px;
        border: 1px solid var(--border-color);
        background: var(--bg-color);
        color: var(--text-color);
        font-size: 0.95rem;
        transition: border-color 0.2s;
    }

    .form-group input:focus,
    .form-group select:focus {
        outline: none;
        border-color: var(--primary-color);
    }

    .checkbox-group {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 4px 0;
    }

    .checkbox-group input {
        width: 18px;
        height: 18px;
        cursor: pointer;
    }

    .checkbox-group label {
        font-size: 0.9rem;
        cursor: pointer;
    }

    .range-labels {
        display: flex;
        justify-content: space-between;
        font-size: 0.7rem;
        color: var(--icon-color);
        margin-top: -4px;
    }

    .field-help {
        font-size: 0.75rem;
        color: var(--icon-color);
        margin-top: 4px;
        line-height: 1.4;
    }

    input[type="range"] {
        height: 6px;
        border-radius: 3px;
        background: var(--border-color);
        appearance: none;
        margin: 10px 0;
    }

    input[type="range"]::-webkit-slider-thumb {
        appearance: none;
        width: 18px;
        height: 18px;
        border-radius: 50%;
        background: var(--primary-color);
        cursor: pointer;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    }

    /* Footer / Preview Styling */
    .modal-footer {
        padding: 16px 20px;
        background: var(--bg-color);
        border-top: 1px solid var(--border-color);
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .preview-area {
        position: relative;
        background: var(--header-bg);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 12px;
        min-height: 60px;
        display: flex;
        align-items: center;
    }

    .preview-tag {
        position: absolute;
        top: -8px;
        left: 10px;
        background: var(--bg-color);
        padding: 0 4px;
        font-size: 0.65rem;
        font-weight: 800;
        color: var(--icon-color);
        text-transform: uppercase;
    }

    .preview-box {
        width: 100%;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        line-height: normal; /* Override default */
    }

    .save-btn {
        background: var(--primary-color);
        color: white;
        border: none;
        padding: 12px;
        border-radius: 10px;
        font-weight: 700;
        font-size: 1rem;
        cursor: pointer;
        transition:
            transform 0.1s,
            filter 0.2s;
    }

    .save-btn:hover {
        filter: brightness(1.1);
    }

    .save-btn:active {
        transform: scale(0.98);
    }

    /* Confirm Box */
    .confirm-box {
        background: var(--bg-color);
        padding: 24px;
        border-radius: 16px;
        width: 300px;
        box-shadow: var(--shadow-lg);
        border: 1px solid var(--border-color);
    }

    .confirm-box h3 {
        margin: 0 0 12px 0;
        font-size: 1.1rem;
    }

    .confirm-actions {
        display: flex;
        justify-content: flex-end;
        gap: 12px;
        margin-top: 24px;
    }

    .cancel-btn {
        background: none;
        border: 1px solid var(--border-color);
        padding: 8px 16px;
        border-radius: 8px;
        cursor: pointer;
        color: var(--text-color);
    }

    .delete-btn {
        background: #ef4444;
        color: white;
        border: none;
        padding: 8px 16px;
        border-radius: 8px;
        cursor: pointer;
        font-weight: 600;
    }

    @media (max-width: 480px) {
        .modal-overlay {
            padding: 10px;
        }

        .modal-content {
            max-height: calc(100vh - 20px);
            border-radius: 12px;
        }

        .header-main h2 {
            display: none; /* Hide 'Styles' title on very small screens to save space */
        }

        .form-row {
            flex-direction: column;
            gap: 12px;
        }

        .tab-btn span {
            display: none;
        }

        .tab-btn {
            padding: 12px;
        }
    }
</style>
