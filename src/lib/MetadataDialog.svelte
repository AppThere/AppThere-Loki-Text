<script lang="ts">
    import { run } from "svelte/legacy";
    import { X } from "lucide-svelte";

    let {
        isOpen = false,
        metadata = $bindable({
            title: "",
            description: "",
            subject: "",
            creator: "",
            creationDate: "",
            generator: "",
        }),
        onClose,
    } = $props();

    let title = $state("");
    let description = $state("");
    let subject = $state("");
    let creator = $state("");

    // Sync local state with prop
    run(() => {
        if (isOpen) {
            title = metadata.title || "";
            description = metadata.description || "";
            subject = metadata.subject || "";
            creator = metadata.creator || "";
        }
    });

    function handleSave() {
        metadata = {
            ...metadata,
            title,
            description,
            subject,
            creator,
        };
        onClose();
    }

    function handleCancel() {
        onClose();
    }
</script>

{#if isOpen}
    <div class="modal-backdrop" onclick={handleCancel} role="presentation">
        <div
            class="modal"
            onclick={(e) => e.stopPropagation()}
            role="presentation"
        >
            <header>
                <h2>Document Properties</h2>
                <button class="close-btn" onclick={handleCancel}>
                    <X size={20} />
                </button>
            </header>

            <div class="modal-body">
                <div class="form-group">
                    <label for="title">Title</label>
                    <input type="text" id="title" bind:value={title} />
                </div>

                <div class="form-group">
                    <label for="subject">Subject (Keywords)</label>
                    <input type="text" id="subject" bind:value={subject} />
                </div>

                <div class="form-group">
                    <label for="creator">Author</label>
                    <input type="text" id="creator" bind:value={creator} />
                </div>

                <div class="form-group">
                    <label for="description">Description</label>
                    <textarea id="description" bind:value={description} rows="4"
                    ></textarea>
                </div>

                <div class="meta-info">
                    {#if metadata.creationDate}
                        <p>
                            <small
                                >Created: {new Date(
                                    metadata.creationDate,
                                ).toLocaleString()}</small
                            >
                        </p>
                    {/if}
                    {#if metadata.generator}
                        <p><small>Generator: {metadata.generator}</small></p>
                    {/if}
                </div>
            </div>

            <footer>
                <button class="cancel-btn" onclick={handleCancel}>Cancel</button
                >
                <button class="save-btn" onclick={handleSave}>OK</button>
            </footer>
        </div>
    </div>
{/if}

<style>
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .modal {
        background: var(--header-bg);
        border-radius: 8px;
        width: 100%;
        max-width: 500px;
        box-shadow: var(--shadow-md);
        display: flex;
        flex-direction: column;
        max-height: 90vh;
        color: var(--text-color);
    }

    header {
        padding: 16px 20px;
        border-bottom: 1px solid var(--border-color);
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    header h2 {
        margin: 0;
        font-size: 1.1rem;
        color: var(--text-color);
    }

    .close-btn {
        background: none;
        border: none;
        font-size: 1.5rem;
        color: var(--icon-color);
        cursor: pointer;
        padding: 0;
        line-height: 1;
        display: flex;
        align-items: center;
    }

    .close-btn:hover {
        color: var(--text-color);
    }

    .modal-body {
        padding: 20px;
        overflow-y: auto;
    }

    .form-group {
        margin-bottom: 16px;
    }

    label {
        display: block;
        margin-bottom: 6px;
        font-size: 0.9rem;
        font-weight: 500;
        color: var(--text-color);
    }

    input,
    textarea {
        width: 100%;
        padding: 8px 12px;
        border: 1px solid var(--border-color);
        border-radius: 6px;
        font-size: 0.95rem;
        transition: border-color 0.2s;
        box-sizing: border-box;
        background: var(--bg-color);
        color: var(--text-color);
    }

    input:focus,
    textarea:focus {
        outline: none;
        border-color: var(--primary-color);
        box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
    }

    .meta-info {
        margin-top: 20px;
        padding-top: 10px;
        border-top: 1px solid var(--border-color);
    }

    .meta-info p {
        margin: 4px 0;
        color: var(--icon-color);
    }

    footer {
        padding: 16px 20px;
        border-top: 1px solid var(--border-color);
        display: flex;
        justify-content: flex-end;
        gap: 12px;
    }

    button {
        padding: 8px 16px;
        border-radius: 6px;
        font-weight: 500;
        font-size: 0.9rem;
        cursor: pointer;
        transition: all 0.2s;
    }

    .cancel-btn {
        background: var(--header-bg);
        border: 1px solid var(--border-color);
        color: var(--text-color);
    }

    .cancel-btn:hover {
        background: var(--hover-bg);
    }

    .save-btn {
        background: var(--primary-color);
        border: 1px solid var(--primary-color);
        color: white;
    }

    .save-btn:hover {
        opacity: 0.9;
    }
</style>
