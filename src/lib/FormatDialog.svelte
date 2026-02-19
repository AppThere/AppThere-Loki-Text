<script lang="ts">
	import { X, FileText, FileCode2 } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';

	let {
		isOpen = $bindable(),
		onSelect = (format: 'odt' | 'fodt') => {},
		onCancel = () => {}
	} = $props();

	function close() {
		isOpen = false;
		onCancel();
	}

	function select(format: 'odt' | 'fodt') {
		isOpen = false;
		onSelect(format);
	}
</script>

{#if isOpen}
	<div
		class="modal-backdrop"
		transition:fade={{ duration: 200 }}
		onclick={close}
		onkeydown={(e) => {
			if (e.key === 'Escape') close();
		}}
		role="button"
		tabindex="0"
		aria-label="Close dialog"
	>
		<div
			class="modal-content"
			transition:scale={{ duration: 200, start: 0.95 }}
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<div class="modal-header">
				<h2>Select Document Format</h2>
				<button class="close-btn" onclick={close} aria-label="Close">
					<X size={20} />
				</button>
			</div>

			<div class="modal-body">
				<p class="description">
					Choose the file format to save your document. Both formats are compatible with LibreOffice
					and feature the same content and styling.
				</p>

				<div class="format-options">
					<button class="format-card" onclick={() => select('odt')}>
						<div class="icon-wrapper">
							<FileText size={32} />
						</div>
						<div class="card-content">
							<h3>ODT (Compressed)</h3>
							<p>
								Standard OpenDocument Text format. A zipped archive containing XML and media files.
								Smaller file size.
							</p>
						</div>
					</button>

					<button class="format-card" onclick={() => select('fodt')}>
						<div class="icon-wrapper">
							<FileCode2 size={32} />
						</div>
						<div class="card-content">
							<h3>FODT (Flat XML)</h3>
							<p>
								Single-file XML format. Media is base64 encoded. Great for version control systems
								like Git.
							</p>
						</div>
					</button>
				</div>
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
		max-width: 480px;
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
		color: var(--text-color, #f5f5f4);
	}

	.description {
		margin: 0 0 20px 0;
		color: var(--icon-color, #a8a29e);
		font-size: 0.95rem;
		line-height: 1.5;
	}

	.format-options {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.format-card {
		display: flex;
		align-items: center;
		gap: 16px;
		background: var(--surface-bg, #292524);
		border: 1px solid var(--border-color, #44403c);
		border-radius: 8px;
		padding: 16px;
		cursor: pointer;
		text-align: left;
		transition: all 0.2s;
		color: inherit;
	}

	.format-card:hover {
		background: var(--hover-bg, #44403c);
		border-color: var(--icon-color, #a8a29e);
		transform: translateY(-2px);
	}

	.icon-wrapper {
		color: var(--text-color, #f5f5f4);
		background: rgba(255, 255, 255, 0.1);
		padding: 12px;
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.card-content h3 {
		margin: 0 0 4px 0;
		font-size: 1.1rem;
		font-weight: 600;
	}

	.card-content p {
		margin: 0;
		font-size: 0.85rem;
		color: var(--icon-color, #a8a29e);
		line-height: 1.4;
	}
</style>
