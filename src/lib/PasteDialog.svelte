<script lang="ts">
    import { X, FileText, Layout, Code } from "lucide-svelte";

    let { isOpen = false, onSelect, onClose } = $props();

    function handleOption(option: "plain" | "structure" | "dirty") {
        onSelect(option);
        onClose();
    }
</script>

{#if isOpen}
    <div
        class="overlay"
        role="button"
        tabindex="0"
        onclick={onClose}
        onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") onClose();
        }}
    >
        <div
            class="modal"
            role="dialog"
            aria-modal="true"
            tabindex="-1"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
        >
            <header>
                <h2>Paste Options</h2>
                <button class="close-btn" onclick={onClose}>
                    <X size={20} />
                </button>
            </header>

            <div class="options-grid">
                <button
                    class="option-card"
                    onclick={() => handleOption("plain")}
                >
                    <div class="icon-wrapper">
                        <FileText size={32} />
                    </div>
                    <div class="info">
                        <h3>Plain Text</h3>
                        <p>
                            Paste text only, removing all formatting and links.
                        </p>
                    </div>
                </button>

                <button
                    class="option-card"
                    onclick={() => handleOption("structure")}
                >
                    <div class="icon-wrapper">
                        <Layout size={32} />
                    </div>
                    <div class="info">
                        <h3>Preserve Structure</h3>
                        <p>
                            Keep headings, lists, and links, but remove fonts
                            and colors.
                        </p>
                    </div>
                </button>

                <button
                    class="option-card"
                    onclick={() => handleOption("dirty")}
                >
                    <div class="icon-wrapper">
                        <Code size={32} />
                    </div>
                    <div class="info">
                        <h3>Dirty Paste</h3>
                        <p>
                            Paste as-is, keeping as much original formatting as
                            possible.
                        </p>
                    </div>
                </button>
            </div>

            <div class="footer">
                <button class="cancel-btn" onclick={onClose}>Cancel</button>
            </div>
        </div>
    </div>
{/if}

<style>
    .overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 2000;
        backdrop-filter: blur(2px);
    }

    .modal {
        background: var(--header-bg);
        border-radius: 12px;
        width: 100%;
        max-width: 500px;
        box-shadow: var(--shadow-md);
        display: flex;
        flex-direction: column;
        color: var(--text-color);
        border: 1px solid var(--border-color);
        overflow: hidden;
    }

    header {
        padding: 16px 20px;
        border-bottom: 1px solid var(--border-color);
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    h2 {
        margin: 0;
        font-size: 1.1rem;
        font-weight: 600;
    }

    .close-btn {
        background: none;
        border: none;
        color: var(--icon-color);
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
    }

    .close-btn:hover {
        background: var(--hover-bg);
        color: var(--text-color);
    }

    .options-grid {
        padding: 20px;
        display: grid;
        gap: 12px;
    }

    .option-card {
        display: flex;
        align-items: flex-start;
        gap: 16px;
        background: var(--bg-color);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        padding: 16px;
        text-align: left;
        cursor: pointer;
        transition: all 0.2s;
        width: 100%;
    }

    .option-card:hover {
        border-color: var(--primary-color);
        background: var(--hover-bg);
        transform: translateY(-1px);
        box-shadow: var(--shadow-sm);
    }

    .icon-wrapper {
        color: var(--primary-color);
        background: var(--header-bg); /* Or a lighter shade of primary */
        padding: 8px;
        border-radius: 8px;
        border: 1px solid var(--border-color);
    }

    .info h3 {
        margin: 0 0 4px 0;
        font-size: 1rem;
        font-weight: 600;
        color: var(--text-color);
    }

    .info p {
        margin: 0;
        font-size: 0.85rem;
        color: var(--icon-color);
        line-height: 1.4;
    }

    .footer {
        padding: 16px 20px;
        border-top: 1px solid var(--border-color);
        display: flex;
        justify-content: flex-end;
        background: var(--bg-color);
    }

    .cancel-btn {
        padding: 8px 16px;
        border-radius: 6px;
        font-weight: 500;
        cursor: pointer;
        background: var(--header-bg);
        border: 1px solid var(--border-color);
        color: var(--text-color);
    }

    .cancel-btn:hover {
        background: var(--hover-bg);
    }
</style>
