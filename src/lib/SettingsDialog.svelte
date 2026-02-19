<script lang="ts">
	import { X, Moon, Sun, Monitor } from 'lucide-svelte';
	import { fade, scale } from 'svelte/transition';
	import { settingsStore, type Theme } from './settingsStore';

	let { isOpen = $bindable(), onClose = () => {} } = $props();

	function close() {
		isOpen = false;
		onClose?.();
	}

	function setTheme(theme: Theme) {
		settingsStore.updateSetting('theme', theme);
	}
</script>

{#if isOpen}
	<div
		class="modal-backdrop"
		transition:fade={{ duration: 200 }}
		onclick={close}
		onkeydown={(e) => {
			if (e.key === 'Escape' || e.key === 'Enter' || e.key === ' ') {
				close();
			}
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
				<h2>Settings</h2>
				<button class="close-btn" onclick={close} aria-label="Close">
					<X size={20} />
				</button>
			</div>

			<div class="modal-body">
				<div class="setting-group">
					<h3 class="group-title">Appearance</h3>

					<div class="theme-selector">
						<button
							class="theme-btn"
							class:active={$settingsStore.theme === 'light'}
							onclick={() => setTheme('light')}
						>
							<Sun size={20} />
							<span>Light</span>
						</button>
						<button
							class="theme-btn"
							class:active={$settingsStore.theme === 'dark'}
							onclick={() => setTheme('dark')}
						>
							<Moon size={20} />
							<span>Dark</span>
						</button>
						<button
							class="theme-btn"
							class:active={$settingsStore.theme === 'system'}
							onclick={() => setTheme('system')}
						>
							<Monitor size={20} />
							<span>System</span>
						</button>
					</div>
				</div>

				<div class="setting-item">
					<div class="setting-details">
						<div class="setting-title">Editor Defaults</div>
						<div class="setting-description">Configure default font and styles (Coming soon).</div>
					</div>
				</div>
			</div>

			<div class="modal-footer">
				<button class="primary-btn" onclick={close}>Done</button>
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
		max-width: 450px;
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

	.setting-group {
		margin-bottom: 24px;
	}

	.group-title {
		font-size: 0.75rem;
		font-weight: 700;
		text-transform: uppercase;
		color: var(--icon-color);
		letter-spacing: 0.05em;
		margin-bottom: 12px;
	}

	.theme-selector {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: 12px;
	}

	.theme-btn {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 12px;
		background: var(--hover-bg);
		border: 1px solid var(--border-color);
		border-radius: 8px;
		color: var(--text-color);
		cursor: pointer;
		transition: all 0.2s;
	}

	.theme-btn span {
		font-size: 0.8rem;
		font-weight: 500;
	}

	.theme-btn:hover {
		border-color: var(--primary-color);
	}

	.theme-btn.active {
		background: var(--primary-color);
		border-color: var(--primary-color);
		color: white;
	}

	.modal-body {
		padding: 24px;
		overflow-y: auto;
		color: var(--text-color, #f5f5f4);
	}

	.setting-item {
		display: flex;
		align-items: center;
		padding: 12px 0;
		border-bottom: 1px solid var(--border-color, #292524);
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
