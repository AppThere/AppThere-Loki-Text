<script lang="ts">
    import { styleRegistry } from "./styleStore";

    let {
        currentStyleId = $bindable("Normal Text"),
        onSelect,
        onEdit,
    } = $props();

    let isOpen = $state(false);

    function handleSelect(id: string) {
        onSelect(id);
        isOpen = false;
    }
</script>

<div class="style-select">
    <button class="select-trigger" onclick={() => (isOpen = !isOpen)}>
        <span class="current-style"
            >{$styleRegistry.find((s) => s.id === currentStyleId)?.name ||
                currentStyleId}</span
        >
        <span class="chevron">▼</span>
    </button>

    {#if isOpen}
        <div class="select-dropdown">
            <div class="dropdown-header">
                <span>Block Style</span>
                <button
                    class="edit-btn"
                    onclick={() => {
                        onEdit();
                        isOpen = false;
                    }}>Manage</button
                >
            </div>
            <div class="options">
                {#each $styleRegistry as style}
                    <button
                        class="option"
                        class:active={style.id === currentStyleId}
                        onclick={() => handleSelect(style.id)}
                    >
                        <span class="style-name">{style.name}</span>
                    </button>
                {/each}
            </div>
        </div>
    {/if}
</div>

{#if isOpen}
    <div
        class="overlay"
        onclick={() => (isOpen = false)}
        aria-hidden="true"
    ></div>
{/if}

<style>
    .style-select {
        position: relative;
        user-select: none;
    }

    .select-trigger {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        background: var(--bg-color);
        border: 1px solid var(--border-color);
        border-radius: 8px;
        font-size: 0.9rem;
        font-weight: 500;
        color: var(--text-color);
        cursor: pointer;
        min-width: 140px;
        justify-content: space-between;
    }

    .select-trigger:hover {
        background: var(--hover-bg);
        border-color: var(--icon-color);
    }

    .current-style {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .chevron {
        font-size: 0.7rem;
        color: var(--icon-color);
    }

    .select-dropdown {
        position: absolute;
        bottom: calc(100% + 8px);
        left: 0;
        width: 200px;
        background: var(--header-bg);
        border: 1px solid var(--border-color);
        border-radius: 12px;
        box-shadow: var(--shadow-md);
        z-index: 1001;
        overflow: hidden;
        animation: slide-up 0.15s cubic-bezier(0, 0, 0.2, 1);
    }

    @keyframes slide-up {
        from {
            transform: translateY(10px);
            opacity: 0;
        }
        to {
            transform: translateY(0);
            opacity: 1;
        }
    }

    .dropdown-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 10px 12px;
        background: var(--bg-color);
        border-bottom: 1px solid var(--border-color);
        font-size: 0.75rem;
        font-weight: 600;
        color: var(--icon-color);
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    .edit-btn {
        background: none;
        border: none;
        color: var(--primary-color);
        font-weight: 700;
        cursor: pointer;
        padding: 2px 4px;
    }

    .options {
        max-height: 240px;
        overflow-y: auto;
        padding: 4px;
    }

    .option {
        width: 100%;
        display: flex;
        align-items: center;
        padding: 10px 12px;
        background: none;
        border: none;
        border-radius: 6px;
        font-size: 0.9rem;
        color: var(--text-color);
        cursor: pointer;
        text-align: left;
        transition: background 0.1s;
    }

    .option:hover {
        background: var(--hover-bg);
        color: var(--text-color);
    }

    .option.active {
        background: var(--hover-bg);
        color: var(--primary-color);
        font-weight: 600;
    }

    .overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: 1000;
    }
</style>
