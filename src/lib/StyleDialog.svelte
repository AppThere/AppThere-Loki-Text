<script lang="ts">
    import { styleRegistry, type BlockStyle } from "./styleStore";

    let { isOpen = false, onSelect, onClose } = $props();

    let selectedStyleId = $state("Normal Text");
    let isCreating = $state(false);
    let showDeleteConfirm = $state(false);

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
        // Popular Google Fonts (assuming likely available or linked elsewhere, or just semantic names)
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
        // Parse weight to number if possible, default 400
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
    }

    function startCreate() {
        isCreating = true;
        newName = "New Style";
        newDesc = "";
        fontFamily = "Arial, sans-serif";
        fontSize = "12pt";
        fontWeight = 400;
        lineHeight = "1.15";
        marginLeft = "0pt";
        marginRight = "0pt";
        textIndent = "0pt";
        marginTop = "0pt";
        marginBottom = "0pt";
        textAlign = "left";
        hyphenate = false;
        orphans = 2;
        widows = 2;
        basedOn = "Normal Text";
        nextStyle = undefined;
    }

    function cancelCreate() {
        isCreating = false;
        // Re-load the currently selected style
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

    // Derived Preview Style
    let previewStyle = $derived({
        fontFamily: fontFamily,
        fontSize: fontSize,
        fontWeight: fontWeight,
        lineHeight: lineHeight,
        textAlign: textAlign,
        // Margin/Indent logic might be complex to preview in a small box,
        // but we can try basic padding/margin if units are CSS compatible.
        // For simplicity, we preview typography primarily.
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
        // Extract first family name, remove quotes
        const firstFamily = fontFamily
            .split(",")[0]
            .trim()
            .replace(/['"]/g, "");

        if (GOOGLE_FONTS.has(firstFamily)) {
            // Construct URL: https://fonts.googleapis.com/css2?family=Roboto:wght@400;700&display=swap
            // We load the specific weight plus a regular fallback (400)
            const weights = [400];
            if (fontWeight !== 400)
                weights.push(
                    typeof fontWeight === "string"
                        ? parseInt(fontWeight)
                        : fontWeight,
                );
            // Dedupe
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

        // If it's just a number
        if (!isNaN(Number(trimmed))) {
            const num = Number(trimmed);
            if (type === "lineHeight") {
                // Heuristic: Small numbers (< 4) are multipliers, large are likely fixed points
                return num < 4 ? String(num) : `${num}pt`;
            } else {
                // Default lengths to pt
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
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
        class="modal-overlay"
        onclick={onClose}
        onkeydown={(e) => {
            if (e.key === "Escape") onClose();
        }}
        aria-hidden="true"
    >
        <div
            class="modal-content"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            tabindex="-1"
            aria-labelledby="modal-title"
        >
            <header class="modal-header">
                <div>
                    <h2 id="modal-title">Manage Styles</h2>
                    <p class="subtitle">
                        Create, edit, and reorganize block styles.
                    </p>
                </div>
                <button
                    class="close-btn"
                    onclick={onClose}
                    aria-label="Close dialog">×</button
                >
            </header>

            <div class="top-bar">
                {#if isCreating}
                    <div class="creating-badge">Creating New Style</div>
                    <button class="secondary" onclick={cancelCreate}
                        >Cancel</button
                    >
                {:else}
                    <div class="style-selector-group">
                        <select
                            bind:value={selectedStyleId}
                            class="style-select"
                        >
                            {#each $styleRegistry as style}
                                <option value={style.id}
                                    >{style.displayName || style.name}</option
                                >
                            {/each}
                        </select>
                        <button
                            class="icon-btn add"
                            onclick={startCreate}
                            aria-label="New Style"
                            title="Create New Style">+</button
                        >
                        <button
                            class="icon-btn delete"
                            onclick={requestDelete}
                            disabled={selectedStyleId === "Normal Text"}
                            aria-label="Delete Style"
                            title="Delete Selected Style">🗑️</button
                        >
                    </div>
                {/if}
            </div>

            <div class="main-body">
                <!-- Preview Section -->
                <div class="preview-box">
                    <span class="preview-label">Preview</span>
                    <div
                        class="preview-content"
                        style:font-family={previewStyle.fontFamily}
                        style:font-size={previewStyle.fontSize}
                        style:font-weight={previewStyle.fontWeight}
                        style:line-height={previewStyle.lineHeight}
                        style:text-align={previewStyle.textAlign}
                    >
                        The quick brown fox jumps over the lazy dog.
                    </div>
                </div>

                <div class="form-container">
                    <div class="form-row">
                        <div class="form-group grow">
                            <label for="name">Name</label>
                            <input
                                id="name"
                                bind:value={newName}
                                placeholder="Style Name"
                            />
                        </div>
                        <div class="form-group grow">
                            <label for="basedOn">Based On</label>
                            <select id="basedOn" bind:value={basedOn}>
                                <option value={undefined}>None</option>
                                {#each $styleRegistry as s}
                                    {#if isCreating || s.id !== selectedStyleId}
                                        <option value={s.id}
                                            >{s.displayName || s.name}</option
                                        >
                                    {/if}
                                {/each}
                            </select>
                        </div>
                    </div>
                    <div class="form-row">
                        <div class="form-group grow">
                            <label for="nextStyle">Next Style</label>
                            <select id="nextStyle" bind:value={nextStyle}>
                                <option value={undefined}>Same style</option>
                                {#each $styleRegistry as s}
                                    <option value={s.id}
                                        >{s.displayName || s.name}</option
                                    >
                                {/each}
                            </select>
                        </div>
                    </div>

                    <div class="separator">Typography</div>

                    <div class="form-row">
                        <div class="form-group grow">
                            <label for="fontFamily">Font</label>
                            <input
                                list="fonts"
                                id="fontFamily"
                                bind:value={fontFamily}
                                placeholder="Select or type font..."
                            />
                            <datalist id="fonts">
                                {#each COMMON_FONTS as font}
                                    <option value={font}></option>
                                {/each}
                            </datalist>
                        </div>
                        <div class="form-group w-short">
                            <label for="fontSize">Size</label>
                            <input
                                id="fontSize"
                                bind:value={fontSize}
                                onblur={() =>
                                    (fontSize = normalizeUnit(fontSize))}
                                placeholder="12pt"
                            />
                        </div>
                    </div>

                    <div class="form-row">
                        <div class="form-group grow">
                            <label for="fontWeight">Weight ({fontWeight})</label
                            >
                            <input
                                type="range"
                                id="fontWeight"
                                min="100"
                                max="900"
                                step="100"
                                bind:value={fontWeight}
                            />
                        </div>
                        <div class="form-group w-short">
                            <label for="textAlign">Align</label>
                            <select id="textAlign" bind:value={textAlign}>
                                <option value="left">Left</option>
                                <option value="center">Center</option>
                                <option value="right">Right</option>
                                <option value="justify">Justify</option>
                            </select>
                        </div>
                    </div>

                    <div class="separator">Spacing & Indents</div>

                    <div class="form-row">
                        <div class="form-group">
                            <label for="lineHeight">Line Height</label>
                            <input
                                id="lineHeight"
                                bind:value={lineHeight}
                                onblur={() =>
                                    (lineHeight = normalizeUnit(
                                        lineHeight,
                                        "lineHeight",
                                    ))}
                                placeholder="1.15"
                            />
                        </div>
                        <div class="form-group">
                            <label for="textIndent">First Line Indent</label>
                            <input
                                id="textIndent"
                                bind:value={textIndent}
                                onblur={() =>
                                    (textIndent = normalizeUnit(textIndent))}
                                placeholder="0pt"
                            />
                        </div>
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label for="marginTop">Margin Top</label>
                            <input
                                id="marginTop"
                                bind:value={marginTop}
                                onblur={() =>
                                    (marginTop = normalizeUnit(marginTop))}
                                placeholder="0pt"
                            />
                        </div>
                        <div class="form-group">
                            <label for="marginBottom">Margin Bottom</label>
                            <input
                                id="marginBottom"
                                bind:value={marginBottom}
                                onblur={() =>
                                    (marginBottom =
                                        normalizeUnit(marginBottom))}
                                placeholder="8pt"
                            />
                        </div>
                    </div>

                    <div class="form-actions">
                        <button class="primary" onclick={saveStyle}
                            >Save Style</button
                        >
                    </div>
                </div>
            </div>
        </div>
    </div>

    {#if showDeleteConfirm}
        <div class="modal-overlay confirm-overlay" role="alertdialog">
            <div class="confirm-box">
                <h3>Delete Style?</h3>
                <p>
                    Are you sure you want to delete "{(() => {
                        const s = $styleRegistry.find(
                            (s) => s.id === selectedStyleId,
                        );
                        return s?.displayName || s?.name;
                    })()}"? usage in document may revert to Normal Text.
                </p>
                <div class="confirm-actions">
                    <button onclick={() => (showDeleteConfirm = false)}
                        >Cancel</button
                    >
                    <button class="danger" onclick={confirmDelete}
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
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        justify-content: center;
        align-items: center;
        z-index: 1000;
        backdrop-filter: blur(2px);
    }

    .confirm-overlay {
        z-index: 1100;
    }

    .modal-content {
        background: var(--bg-color);
        width: 100%;
        max-width: 500px;
        border-radius: 12px;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.2);
        display: flex;
        flex-direction: column;
        max-height: 85vh;
        overflow: hidden;
        color: var(--text-color);
    }

    .modal-header {
        padding: 16px 20px;
        border-bottom: 1px solid var(--border-color);
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        background: var(--header-bg);
    }

    .modal-header h2 {
        margin: 0;
        font-size: 1.25rem;
    }
    .subtitle {
        margin: 4px 0 0 0;
        font-size: 0.85rem;
        color: var(--icon-color);
    }

    .close-btn {
        background: none;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
        color: var(--icon-color);
        padding: 0;
        line-height: 1;
    }

    .top-bar {
        padding: 12px 20px;
        background: var(--bg-color);
        border-bottom: 1px solid var(--border-color);
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .style-selector-group {
        display: flex;
        width: 100%;
        gap: 8px;
    }

    .style-select {
        flex: 1;
        padding: 8px;
        border-radius: 6px;
        border: 1px solid var(--border-color);
        background: var(--bg-color);
        color: var(--text-color);
    }

    .icon-btn {
        width: 36px;
        height: 36px;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 1px solid var(--border-color);
        background: var(--bg-color);
        border-radius: 6px;
        cursor: pointer;
        font-size: 1.1rem;
    }
    .icon-btn:hover {
        background: var(--hover-bg);
    }
    .icon-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
    .delete:hover {
        border-color: #ef4444;
        color: #ef4444;
    }

    .main-body {
        padding: 20px;
        overflow-y: auto;
    }

    .preview-box {
        background: var(--bg-color);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 16px;
        margin-bottom: 20px;
        position: relative;
    }
    .preview-label {
        position: absolute;
        top: -8px;
        left: 12px;
        background: var(--bg-color);
        padding: 0 4px;
        font-size: 0.75rem;
        color: var(--icon-color);
        font-weight: 600;
    }
    .preview-content {
        min-height: 48px;
        display: flex;
        align-items: center;
        overflow: hidden;
        white-space: nowrap;
    }

    .form-container {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .form-row {
        display: flex;
        gap: 12px;
    }
    .grow {
        flex: 1;
    }
    .w-short {
        width: 80px;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    .form-group label {
        font-size: 0.75rem;
        color: var(--icon-color);
        font-weight: 600;
    }
    .form-group input,
    .form-group select {
        padding: 8px;
        border: 1px solid var(--border-color);
        border-radius: 6px;
        background: var(--bg-color);
        color: var(--text-color);
    }

    .separator {
        font-size: 0.75rem;
        font-weight: 700;
        color: var(--icon-color);
        text-transform: uppercase;
        margin: 8px 0 4px 0;
        border-bottom: 1px solid var(--border-color);
        padding-bottom: 4px;
    }

    .confirm-box {
        background: var(--header-bg);
        padding: 24px;
        border-radius: 12px;
        width: 320px;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        border: 1px solid var(--border-color);
        color: var(--text-color);
    }
    .confirm-box h3 {
        margin: 0 0 12px 0;
        font-size: 1.1rem;
    }
    .confirm-actions {
        display: flex;
        justify-content: flex-end;
        gap: 12px;
        margin-top: 20px;
    }

    button.primary {
        background: var(--primary-color);
        color: white;
        border: none;
        padding: 10px 20px;
        border-radius: 6px;
        font-weight: 600;
        cursor: pointer;
        width: 100%;
        margin-top: 12px;
    }
    button.danger {
        background: #ef4444;
        color: white;
        border: none;
        padding: 8px 16px;
        border-radius: 6px;
        font-weight: 600;
        cursor: pointer;
    }
    button.secondary {
        background: transparent;
        border: 1px solid var(--border-color);
        color: var(--text-color);
        padding: 6px 12px;
        border-radius: 6px;
        cursor: pointer;
    }
</style>
