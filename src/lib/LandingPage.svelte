<script lang="ts">
    import { onMount } from "svelte";
    import {
        FileText,
        Plus,
        FolderOpen,
        Clock,
        File,
        Search,
        Trash2,
    } from "lucide-svelte";
    import { recentDocs, type RecentDoc } from "./recentDocs";
    import { TEMPLATES } from "./templates";
    import { ask, message } from "@tauri-apps/plugin-dialog";
    import { addDebugLog } from "$lib/debugStore";

    let { onNew, onOpen, onOpenRecent } = $props();

    let recents: RecentDoc[] = $state([]);
    let selectedTemplateId = $state("basic");
    let selectedRecentPath = $state<string | null>(null);

    onMount(async () => {
        try {
            recents = await recentDocs.get();
        } catch (e) {
            console.error("Failed to load recent docs:", e);
        }
    });

    async function confirmRemove(event: MouseEvent, doc: RecentDoc) {
        event.stopPropagation();

        const removeOption = await ask(
            `What would you like to do with "${doc.title}"?\n\n- Remove from list: Only removes the entry from the editor's home screen.\n- Delete from device: Permanently removes the file from your disk.`,
            {
                title: "Manage Recent Document",
                kind: "info",
                okLabel: "Delete from Device",
                cancelLabel: "Remove from List",
            },
        );

        // Tauri ask() returns true for primary button (Delete from Device)
        // False for secondary button (Remove from List)
        // If they click outside or cancel, it might return null/undefined depending on implementation,
        // but Tauri's ask() usually returns a boolean.

        try {
            if (removeOption) {
                // Delete from Device
                const confirmed = await ask(
                    `Are you ABSOLUTELY sure you want to delete "${doc.title}" from disk?\n\nPath: ${doc.path}`,
                    {
                        title: "Confirm Permanent Deletion",
                        kind: "warning",
                    },
                );
                if (confirmed) {
                    recents = await recentDocs.deleteFile(doc.path);
                    if (selectedRecentPath === doc.path)
                        selectedRecentPath = null;
                }
            } else {
                // Remove from List (this happens if they press the cancelLabel)
                // Wait, Tauri's ask() returns false for cancel button.
                // Let's check the behavior. If it was "cancelled", we remove from list.
                recents = await recentDocs.remove(doc.path);
                if (selectedRecentPath === doc.path) selectedRecentPath = null;
            }
        } catch (e) {
            console.error("Failed to manage document removal:", e);
        }
    }

    function formatDate(iso: string) {
        try {
            return new Date(iso).toLocaleDateString(undefined, {
                month: "short",
                day: "numeric",
                hour: "numeric",
                minute: "numeric",
            });
        } catch {
            return iso;
        }
    }
</script>

<div class="landing-container">
    <div class="header-section">
        <div class="header-spacer"></div>
        <img src="/icons/128x128.png" alt="Loki Logo" class="logo" />
        <h1>AppThere Loki Text</h1>
    </div>

    <div class="tiles-container">
        <!-- New Document Tile -->
        <div class="tile new-tile">
            <div class="tile-header">
                <Plus size={20} />
                <h2>New Document</h2>
            </div>
            <div class="tile-content">
                <div class="template-list">
                    {#each TEMPLATES as template}
                        <button
                            class="template-item"
                            class:selected={selectedTemplateId === template.id}
                            onclick={() => (selectedTemplateId = template.id)}
                            ondblclick={() => onNew(template.id)}
                        >
                            <div class="template-icon">
                                <File size={18} />
                            </div>
                            <div class="template-info">
                                <div class="template-name">{template.name}</div>
                                <div class="template-desc">
                                    {template.description}
                                </div>
                            </div>
                        </button>
                    {/each}
                </div>
            </div>
            <div class="tile-footer">
                <button
                    class="action-btn primary"
                    onclick={() => onNew(selectedTemplateId)}
                    disabled={!selectedTemplateId}
                >
                    Create Document
                </button>
            </div>
        </div>

        <!-- Open Document Tile -->
        <div class="tile open-tile">
            <div class="tile-header">
                <FolderOpen size={20} />
                <h2>Open Document</h2>
            </div>
            <div class="tile-content">
                {#if recents.length > 0}
                    <div class="recent-list">
                        {#each recents as doc}
                            <div
                                class="recent-item"
                                class:selected={selectedRecentPath === doc.path}
                                onclick={async () => {
                                    try {
                                        console.log(
                                            "Recent item clicked:",
                                            doc.path,
                                        );
                                        addDebugLog(
                                            "LandingPage: Clicked recent doc: " +
                                                doc.path,
                                        );
                                        if (
                                            typeof onOpenRecent !== "function"
                                        ) {
                                            addDebugLog(
                                                "ERROR: onOpenRecent is not a function!",
                                            );
                                            return;
                                        }
                                        await onOpenRecent(doc.path);
                                    } catch (e) {
                                        addDebugLog("ERROR in onclick: " + e);
                                        console.error(e);
                                    }
                                }}
                                ondblclick={() => {
                                    /* Double click ignored in favor of single click */
                                }}
                                onkeydown={(e) => {
                                    if (e.key === "Enter" || e.key === " ") {
                                        e.preventDefault();
                                        onOpenRecent(doc.path);
                                    }
                                }}
                                role="button"
                                tabindex="0"
                                title={doc.path}
                            >
                                <div class="doc-icon">
                                    <FileText size={18} />
                                </div>
                                <div class="doc-info">
                                    <div class="doc-title">{doc.title}</div>
                                    <div class="doc-meta">
                                        {formatDate(doc.lastOpened)} • {doc.path
                                            .split("/")
                                            .slice(0, -1)
                                            .pop() || "/"}
                                    </div>
                                </div>
                                <button
                                    class="remove-btn"
                                    onclick={(e) => confirmRemove(e, doc)}
                                    title="Manage document"
                                >
                                    <Trash2 size={16} />
                                </button>
                            </div>
                        {/each}
                    </div>
                {:else}
                    <div class="empty-state">
                        <Clock size={32} />
                        <p>No recent documents</p>
                    </div>
                {/if}
            </div>
            <div class="tile-footer">
                <button class="action-btn primary full-width" onclick={onOpen}>
                    <Search size={16} />
                    Browse Storage...
                </button>
            </div>
        </div>
    </div>
</div>

<style>
    /* ... existing styles ... */

    .full-width {
        width: 100%;
        justify-content: center;
    }
    .landing-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        width: 100%;
        max-width: 900px;
        margin: 0 auto;
        padding: 20px;
        height: 100vh;
        overflow-y: hidden; /* Changed to hidden to try to fit without scroll */
    }

    .header-spacer {
        height: env(safe-area-inset-top, 24px);
    }

    .header-section {
        display: flex;
        flex-direction: column;
        align-items: center;
        text-align: center;
        margin-bottom: 24px;
        flex-shrink: 0;
    }

    .logo {
        width: 48px;
        height: 48px;
        margin-bottom: 12px;
    }

    h1 {
        font-size: 1.5rem;
        font-weight: 800;
        margin: 0 0 4px;
        color: var(--text-color);
        letter-spacing: -0.025em;
    }

    p {
        color: var(--icon-color);
        margin: 0;
        font-size: 0.9rem;
    }

    .tiles-container {
        display: flex;
        flex-direction: column;
        gap: 16px;
        width: 100%;
        flex: 1;
        min-height: 0; /* Important for flex children to shrink */
        margin-bottom: 20px;
    }

    @media (min-width: 768px) {
        .tiles-container {
            flex-direction: row;
            height: auto; /* Let flex handle height */
        }
    }

    .tile {
        flex: 1;
        background: var(--header-bg);
        border: 1px solid var(--border-color);
        border-radius: 12px;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        box-shadow: var(--shadow-sm);
        /* Remove min-height to allow shrinking */
    }

    .tile-header {
        padding: 12px 16px;
        border-bottom: 1px solid var(--border-color);
        display: flex;
        align-items: center;
        gap: 12px;
        background: var(--bg-color);
        color: var(--icon-color);
        flex-shrink: 0;
    }

    .tile-header h2 {
        font-size: 0.85rem;
        font-weight: 600;
        margin: 0;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-color);
    }

    .tile-content {
        flex: 1;
        overflow-y: auto;
        padding: 8px; /* Reduced padding */
        background: var(--header-bg);
        min-height: 0; /* Crucial for scrolling within flex item */
    }

    .tile-footer {
        padding: 12px 16px;
        border-top: 1px solid var(--border-color);
        display: flex;
        gap: 12px;
        background: var(--bg-color);
        flex-shrink: 0;
    }

    /* Template List Styles */
    .template-list {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .template-item {
        display: flex;
        align-items: flex-start;
        gap: 12px;
        padding: 8px 12px;
        background: transparent;
        border: 2px solid transparent;
        border-radius: 8px;
        cursor: pointer;
        text-align: left;
        transition: all 0.1s;
    }

    .template-item:hover {
        background: var(--hover-bg);
    }

    .template-item.selected {
        background: var(--hover-bg);
        border-color: var(--primary-color);
    }

    .template-icon {
        color: var(--icon-color);
        margin-top: 2px;
    }

    .template-item.selected .template-icon {
        color: var(--primary-color);
    }

    .template-info {
        flex: 1;
    }

    .template-name {
        font-weight: 600;
        color: var(--text-color);
        margin-bottom: 2px;
        font-size: 0.9rem;
    }

    .template-desc {
        font-size: 0.8rem;
        color: var(--icon-color);
        line-height: 1.3;
    }

    /* Recent List Styles */
    .recent-list {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .recent-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 6px 12px; /* Reduced padding */
        background: transparent;
        border: 2px solid transparent;
        border-radius: 8px;
        cursor: pointer;
        text-align: left;
        transition: all 0.1s;
    }

    .recent-item:hover {
        background: var(--hover-bg);
    }

    .recent-item.selected {
        background: var(--hover-bg);
        border-color: var(--primary-color);
    }

    .doc-icon {
        color: var(--icon-color);
    }

    .recent-item.selected .doc-icon {
        color: var(--primary-color);
    }

    .doc-info {
        flex: 1;
        min-width: 0;
    }

    .doc-title {
        font-weight: 500;
        color: var(--text-color);
        font-size: 0.85rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .doc-meta {
        font-size: 0.7rem;
        color: var(--icon-color);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        opacity: 0.8;
    }

    .remove-btn {
        opacity: 0;
        background: transparent;
        border: none;
        padding: 8px;
        color: var(--icon-color);
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 4px;
        transition: all 0.2s;
        margin-left: 4px;
    }

    .recent-item:hover .remove-btn,
    .recent-item.selected .remove-btn {
        opacity: 0.6;
    }

    .remove-btn:hover {
        opacity: 1 !important;
        background: rgba(255, 0, 0, 0.1);
        color: #ff4444;
    }

    /* For touch devices, show the remove button by default */
    @media (pointer: coarse) {
        .remove-btn {
            opacity: 0.6;
        }
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: var(--icon-color);
        gap: 12px;
        opacity: 0.5;
        font-size: 0.9rem;
    }

    /* Action Buttons */
    .action-btn {
        flex: 1;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 8px;
        height: 32px; /* Smaller height */
        background: var(--header-bg);
        border: 1px solid var(--border-color);
        border-radius: 6px;
        color: var(--text-color);
        cursor: pointer;
        font-weight: 500;
        font-size: 0.85rem; /* Smaller font */
        transition: all 0.1s;
    }

    .action-btn:hover:not(:disabled) {
        border-color: var(--text-color);
        transform: translateY(-1px);
    }

    .action-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .action-btn.primary {
        background: var(--primary-color);
        border-color: var(--primary-color);
        color: white;
    }

    .action-btn.primary:hover:not(:disabled) {
        filter: brightness(1.1);
        border-color: var(--primary-color);
    }

    .action-btn.secondary {
        background-color: transparent;
        border: 1px dashed var(--border-color);
    }

    .action-btn.secondary:hover {
        border-color: var(--text-color);
        background-color: var(--hover-bg);
    }
</style>
