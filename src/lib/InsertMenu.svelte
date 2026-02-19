<script lang="ts">
	import { Plus, Image, Table, FilePlus, CornerDownLeft } from 'lucide-svelte';
	import { onMount } from 'svelte';

	let { onInsertImage, onInsertTable, onInsertPageBreak, onInsertLineBreak } = $props();

	let isOpen = $state(false);
	let triggerRef: HTMLButtonElement;
	let dropdownStyle = $state('');

	function updatePosition() {
		if (triggerRef) {
			const rect = triggerRef.getBoundingClientRect();
			// Position above the button
			dropdownStyle = `
                position: fixed;
                bottom: ${window.innerHeight - rect.top + 8}px;
                left: ${rect.left}px;
            `;
		}
	}

	$effect(() => {
		if (isOpen) {
			window.addEventListener('scroll', updatePosition, true);
			window.addEventListener('resize', updatePosition);
			return () => {
				window.removeEventListener('scroll', updatePosition, true);
				window.removeEventListener('resize', updatePosition);
			};
		}
	});

	function toggleOpen() {
		if (!isOpen) {
			updatePosition();
		}
		isOpen = !isOpen;
	}

	function handleSelect(action: () => void) {
		action();
		isOpen = false;
	}
</script>

<div class="insert-menu">
	<button
		class="insert-trigger"
		class:active={isOpen}
		bind:this={triggerRef}
		onclick={toggleOpen}
		title="Insert..."
		aria-label="Insert"
		aria-haspopup="true"
		aria-expanded={isOpen}
	>
		<Plus size={18} />
		<span class="label">Insert</span>
	</button>

	{#if isOpen}
		<div class="menu-dropdown" style={dropdownStyle}>
			<button onclick={() => handleSelect(onInsertImage)} class="menu-item">
				<Image size={18} />
				<span>Image</span>
			</button>
			<button onclick={() => handleSelect(onInsertTable)} class="menu-item">
				<Table size={18} />
				<span>Table</span>
			</button>
			<div class="menu-divider"></div>
			<button onclick={() => handleSelect(onInsertPageBreak)} class="menu-item">
				<FilePlus size={18} />
				<span>Page Break</span>
			</button>
			<button onclick={() => handleSelect(onInsertLineBreak)} class="menu-item">
				<CornerDownLeft size={18} />
				<span>Line Break</span>
			</button>
		</div>
		<div class="overlay" onclick={() => (isOpen = false)} aria-hidden="true"></div>
	{/if}
</div>

<style>
	.insert-menu {
		position: relative;
		user-select: none;
	}

	.insert-trigger {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 12px;
		height: 32px;
		background: transparent;
		border: 1px solid transparent;
		border-radius: 6px;
		color: var(--icon-color);
		font-size: 0.9rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
	}

	.insert-trigger:hover,
	.insert-trigger.active {
		background: var(--hover-bg);
		color: var(--text-color);
	}

	.menu-dropdown {
		/* position: fixed is applied inline */
		min-width: 140px;
		background: var(--header-bg);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		box-shadow: var(--shadow-md);
		z-index: 1001;
		overflow: hidden;
		animation: slide-up 0.15s cubic-bezier(0, 0, 0.2, 1);
		padding: 4px;
		display: flex;
		flex-direction: column;
		gap: 2px;
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

	.menu-item {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 12px;
		width: 100%;
		text-align: left;
		background: transparent;
		border: none;
		border-radius: 4px;
		color: var(--text-color);
		font-size: 0.9rem;
		cursor: pointer;
		transition: background 0.1s;
	}

	.menu-item:hover {
		background: var(--hover-bg);
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
