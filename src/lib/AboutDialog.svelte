<script lang="ts">
    import { X } from "lucide-svelte";
    import { fade, scale } from "svelte/transition";

    let { isOpen = $bindable(), onClose = () => {} } = $props();

    function close() {
        isOpen = false;
        onClose?.();
    }
</script>

{#if isOpen}
    <div
        class="modal-backdrop"
        transition:fade={{ duration: 200 }}
        onclick={close}
        onkeydown={(e) => e.key === "Escape" && close()}
        role="button"
        tabindex="-1"
    >
        <div
            class="modal-content"
            transition:scale={{ duration: 200, start: 0.95 }}
            onclick={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
        >
            <div class="modal-header">
                <h2>About AppThere Loki Text</h2>
                <button class="close-btn" onclick={close} aria-label="Close">
                    <X size={20} />
                </button>
            </div>

            <div class="modal-body">
                <div class="app-info">
                    <img
                        src="/icons/128x128.png"
                        alt="Loki Logo"
                        class="app-logo"
                    />
                    <h3>AppThere Loki Text</h3>
                    <p class="version">Version 0.1.0</p>
                    <p class="description">
                        A distraction-free ODT editor for Android and Desktop.
                    </p>
                </div>

                <div class="licenses">
                    <p class="license-info">
                        Released under the <strong>Apache License 2.0</strong>.
                    </p>
                    <h4>Open Source Licenses</h4>
                    <p>
                        This application uses the following open source
                        software:
                    </p>
                    <ul>
                        <li><strong>Tauri</strong> - MIT License</li>
                        <li><strong>Svelte</strong> - MIT License</li>
                        <li><strong>Tiptap</strong> - MIT License</li>
                        <li><strong>Lucide Params</strong> - ISC License</li>
                        <!-- Add more as needed -->
                    </ul>
                    <p class="legal">
                        Copyright © 2026 AppThere. All rights reserved.
                    </p>
                </div>
            </div>

            <div class="modal-footer">
                <button class="primary-btn" onclick={close}>Close</button>
            </div>
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
        justify-content: center;
        align-items: center;
        z-index: 1000;
        backdrop-filter: blur(2px);
    }

    .modal-content {
        background: var(--bg-color, #1c1917);
        border: 1px solid var(--border-color, #44403c);
        border-radius: 12px;
        width: 90%;
        max-width: 400px;
        max-height: 85vh;
        display: flex;
        flex-direction: column;
        box-shadow:
            0 20px 25px -5px rgba(0, 0, 0, 0.1),
            0 10px 10px -5px rgba(0, 0, 0, 0.04);
    }

    .modal-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px 20px;
        border-bottom: 1px solid var(--border-color, #44403c);
    }

    h2 {
        margin: 0;
        font-size: 1.25rem;
        font-weight: 600;
        color: var(--text-color, #f5f5f4);
    }

    .close-btn {
        background: transparent;
        border: none;
        color: var(--icon-color, #a8a29e);
        cursor: pointer;
        padding: 4px;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .close-btn:hover {
        background: var(--hover-bg, #292524);
        color: var(--text-color, #f5f5f4);
    }

    .modal-body {
        padding: 20px;
        overflow-y: auto;
        color: var(--text-color, #f5f5f4);
    }

    .app-info {
        text-align: center;
        margin-bottom: 24px;
    }

    .app-logo {
        width: 64px;
        height: 64px;
        margin-bottom: 12px;
    }

    h3 {
        margin: 0 0 4px 0;
        font-size: 1.5rem;
    }

    .version {
        margin: 0 0 12px 0;
        color: var(--icon-color, #a8a29e);
        font-size: 0.9rem;
    }

    .description {
        font-size: 0.95rem;
        line-height: 1.4;
        color: var(--text-color, #d6d3d1);
    }

    .licenses {
        text-align: left;
        font-size: 0.85rem;
    }

    .license-info {
        margin: 0 0 16px 0;
        font-size: 0.9rem;
        color: var(--text-color, #d6d3d1);
    }

    .licenses h4 {
        margin: 0 0 8px 0;
        font-size: 1rem;
        border-bottom: 1px solid var(--border-color, #44403c);
        padding-bottom: 4px;
    }

    .licenses ul {
        list-style-type: disc;
        padding-left: 20px;
        margin: 0 0 16px 0;
        color: var(--text-color, #d6d3d1);
    }

    .licenses li {
        margin-bottom: 4px;
    }

    .legal {
        font-size: 0.75rem;
        color: var(--icon-color, #78716c);
        text-align: center;
        margin-top: 20px;
    }

    .modal-footer {
        padding: 16px 20px;
        border-top: 1px solid var(--border-color, #44403c);
        display: flex;
        justify-content: flex-end;
    }

    .primary-btn {
        background: #e11d48; /* Rose-600 */
        color: white;
        border: none;
        padding: 8px 16px;
        border-radius: 6px;
        font-weight: 500;
        cursor: pointer;
        font-size: 0.9rem;
        transition: background-color 0.2s;
    }

    .primary-btn:hover {
        background: #be123c; /* Rose-700 */
    }
</style>
