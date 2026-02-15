<script lang="ts">
    import { styleRegistry, type BlockStyle } from "./styleStore";

    let { isOpen = false, onSelect, onClose } = $props();

    let editingStyle: BlockStyle | null = $state(null);
    let newName = $state("");
    let newDesc = $state("");

    // Styling properties
    let fontFamily = $state("");
    let fontSize = $state("");
    let fontWeight = $state("");
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

    function startEdit(style: BlockStyle) {
        editingStyle = style;
        newName = style.name;
        newDesc = style.description;
        fontFamily = style.fontFamily || "";
        fontSize = style.fontSize || "";
        fontWeight = style.fontWeight || "";
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

    function saveEdit() {
        if (editingStyle) {
            styleRegistry.updateStyle(editingStyle.id, {
                name: newName,
                description: newDesc,
                fontFamily,
                fontSize,
                fontWeight,
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
            });
            editingStyle = null;
        }
    }

    function addStyle() {
        const id = `Style-${Date.now()}`;
        styleRegistry.addStyle({
            id,
            name: newName || "New Style",
            description: newDesc,
            fontFamily,
            fontSize,
            fontWeight,
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
        });
        resetForm();
    }

    function resetForm() {
        newName = "";
        newDesc = "";
        fontFamily = "";
        fontSize = "";
        fontWeight = "";
        lineHeight = "";
        marginLeft = "";
        marginRight = "";
        textIndent = "";
        marginTop = "";
        marginBottom = "";
        textAlign = "left";
        hyphenate = false;
        orphans = undefined;
        widows = undefined;
        basedOn = undefined;
        nextStyle = undefined;
    }

    function deleteStyle(id: string) {
        styleRegistry.removeStyle(id);
    }
</script>

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
                <h2 id="modal-title">Manage Styles</h2>
                <button
                    class="close-btn"
                    onclick={onClose}
                    aria-label="Close dialog">×</button
                >
            </header>

            <div class="style-management">
                <div class="style-list">
                    {#each $styleRegistry as style}
                        <div class="style-row">
                            <button
                                class="style-item"
                                onclick={() => {
                                    onSelect(style.id);
                                    onClose();
                                }}
                            >
                                <span class="style-name">{style.name}</span>
                                <span class="style-desc"
                                    >{style.description}</span
                                >
                            </button>
                            <div class="actions">
                                <button
                                    class="icon-btn"
                                    onclick={() => startEdit(style)}>✎</button
                                >
                                <button
                                    class="icon-btn delete"
                                    onclick={() => deleteStyle(style.id)}
                                    >×</button
                                >
                            </div>
                        </div>
                    {/each}
                </div>

                <div class="add-section">
                    <h3>{editingStyle ? "Edit Style" : "Add New Style"}</h3>
                    <div class="form-group">
                        <label for="name">Name</label>
                        <input
                            id="name"
                            bind:value={newName}
                            placeholder="e.g. Normal Text"
                        />
                    </div>
                    <div class="form-group">
                        <label for="desc">Description</label>
                        <input
                            id="desc"
                            bind:value={newDesc}
                            placeholder="Description"
                        />
                    </div>

                    <div class="form-group">
                        <label for="basedOn">Based On Style</label>
                        <select id="basedOn" bind:value={basedOn}>
                            <option value={undefined}>None</option>
                            {#each $styleRegistry as s}
                                {#if !editingStyle || s.id !== editingStyle.id}
                                    <option value={s.id}>{s.name}</option>
                                {/if}
                            {/each}
                        </select>
                    </div>

                    <div class="form-group">
                        <label for="nextStyle">Next Paragraph Style</label>
                        <select id="nextStyle" bind:value={nextStyle}>
                            <option value={undefined}>Same style</option>
                            {#each $styleRegistry as s}
                                <option value={s.id}>{s.name}</option>
                            {/each}
                        </select>
                    </div>

                    <div class="form-grid">
                        <div class="form-column">
                            <h4>Typography</h4>
                            <div class="form-group">
                                <label for="fontFamily">Font Family</label>
                                <input
                                    id="fontFamily"
                                    bind:value={fontFamily}
                                    placeholder="'Liberation Serif', serif"
                                />
                            </div>
                            <div class="form-group">
                                <label for="fontSize">Size</label>
                                <input
                                    id="fontSize"
                                    bind:value={fontSize}
                                    placeholder="12pt"
                                />
                            </div>
                            <div class="form-group">
                                <label for="fontWeight">Weight</label>
                                <input
                                    id="fontWeight"
                                    bind:value={fontWeight}
                                    placeholder="bold, 400"
                                />
                            </div>
                            <div class="form-group">
                                <label for="textAlign">Alignment</label>
                                <select id="textAlign" bind:value={textAlign}>
                                    <option value="left">Left</option>
                                    <option value="center">Center</option>
                                    <option value="right">Right</option>
                                    <option value="justify">Justify</option>
                                </select>
                            </div>
                        </div>

                        <div class="form-column">
                            <h4>Spacing & Indent</h4>
                            <div class="form-group">
                                <label for="lineHeight">Line Spacing</label>
                                <input
                                    id="lineHeight"
                                    bind:value={lineHeight}
                                    placeholder="1.15"
                                />
                            </div>
                            <div class="form-group">
                                <label for="marginTop">Margin Top</label>
                                <input
                                    id="marginTop"
                                    bind:value={marginTop}
                                    placeholder="6pt"
                                />
                            </div>
                            <div class="form-group">
                                <label for="marginBottom">Margin Bottom</label>
                                <input
                                    id="marginBottom"
                                    bind:value={marginBottom}
                                    placeholder="6pt"
                                />
                            </div>
                            <div class="form-group">
                                <label for="textIndent">Text Indent</label>
                                <input
                                    id="textIndent"
                                    bind:value={textIndent}
                                    placeholder="0.5in"
                                />
                            </div>
                        </div>
                    </div>

                    <div class="form-group checkbox">
                        <label>
                            <input type="checkbox" bind:checked={hyphenate} />
                            Enable Hyphenation
                        </label>
                    </div>

                    <div class="form-actions">
                        {#if editingStyle}
                            <button class="primary" onclick={saveEdit}
                                >Save Changes</button
                            >
                            <button
                                onclick={() => {
                                    editingStyle = null;
                                    resetForm();
                                }}>Cancel</button
                            >
                        {:else}
                            <button class="primary" onclick={addStyle}
                                >Add Style</button
                            >
                        {/if}
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .modal-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.4);
        display: flex;
        align-items: flex-end;
        justify-content: center;
        z-index: 1000;
    }

    .modal-content {
        background: var(--header-bg);
        width: 100%;
        max-width: 600px;
        border-radius: 16px 16px 0 0;
        padding: 20px;
        box-shadow: 0 -4px 6px -1px rgba(0, 0, 0, 0.1);
        animation: slide-up 0.3s ease-out;
        color: var(--text-color);
    }

    @keyframes slide-up {
        from {
            transform: translateY(100%);
        }
        to {
            transform: translateY(0);
        }
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 20px;
    }

    .modal-header h2 {
        margin: 0;
        font-size: 1.25rem;
        color: var(--text-color);
    }

    .close-btn {
        background: none;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
        color: var(--icon-color);
    }

    .close-btn:hover {
        color: var(--text-color);
    }

    .style-list {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .style-row {
        display: flex;
        gap: 8px;
        align-items: center;
    }

    .style-item {
        flex: 1;
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        padding: 12px;
        border: 1px solid var(--border-color);
        border-radius: 12px;
        background: var(--bg-color);
        cursor: pointer;
        text-align: left;
        transition: all 0.2s;
    }

    .style-item:hover {
        border-color: var(--primary-color);
        background: var(--hover-bg);
    }

    .style-name {
        font-weight: 600;
        color: var(--text-color);
    }

    .style-desc {
        font-size: 0.75rem;
        color: var(--icon-color);
    }

    .actions {
        display: flex;
        gap: 4px;
    }

    .icon-btn {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 1.2rem;
        color: var(--icon-color);
        padding: 4px;
    }

    .icon-btn:hover {
        color: var(--text-color);
    }

    .style-management {
        display: flex;
        flex-direction: column;
        gap: 24px;
        max-height: 70vh;
        overflow-y: auto;
        padding-bottom: 20px;
    }

    .add-section h3 {
        margin: 0 0 16px 0;
        font-size: 1.1rem;
        color: var(--text-color);
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 12px;
    }

    .form-group label {
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--icon-color);
        text-transform: uppercase;
    }

    .form-group input,
    .form-group select {
        padding: 8px 12px;
        border: 1px solid var(--border-color);
        border-radius: 8px;
        font-size: 0.9rem;
        outline: none;
        background: var(--bg-color);
        color: var(--text-color);
    }

    .form-group input:focus,
    .form-group select:focus {
        border-color: var(--primary-color);
        box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
    }

    .form-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 20px;
        margin: 16px 0;
        padding-top: 16px;
        border-top: 1px solid var(--border-color);
    }

    .form-column h4 {
        margin: 0 0 12px 0;
        font-size: 0.85rem;
        color: var(--text-color);
        font-weight: 700;
    }

    .form-actions {
        display: flex;
        gap: 8px;
        margin-top: 24px;
    }

    .form-group.checkbox {
        flex-direction: row;
        align-items: center;
        gap: 8px;
        margin-top: 8px;
    }

    .form-group.checkbox label {
        text-transform: none;
        font-weight: 500;
        color: var(--text-color);
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 8px;
    }

    button.primary {
        background: var(--primary-color);
        color: white;
        border: none;
        padding: 10px 20px;
        border-radius: 8px;
        font-weight: 600;
        cursor: pointer;
    }

    button.primary:hover {
        opacity: 0.9;
    }

    .delete:hover {
        color: #ef4444;
    }
</style>
