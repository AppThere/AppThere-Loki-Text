<script lang="ts">
	import { debugLogs, clearDebugLogs } from './debugStore';
	import { X, Trash2, Bug } from 'lucide-svelte';

	let isOpen = $state(false);

	function toggle() {
		isOpen = !isOpen;
	}
</script>

<div class="debug-overlay-container">
	{#if !isOpen}
		<button class="debug-toggle" onclick={toggle} title="Open Debug Console">
			<Bug size={20} />
		</button>
	{:else}
		<div class="debug-panel">
			<div class="debug-header">
				<h3>Debug Console</h3>
				<div class="actions">
					<button onclick={clearDebugLogs} title="Clear Logs">
						<Trash2 size={16} />
					</button>
					<button onclick={toggle} title="Close">
						<X size={16} />
					</button>
				</div>
			</div>
			<div class="debug-content">
				{#if $debugLogs.length === 0}
					<div class="empty">No logs yet...</div>
				{:else}
					{#each $debugLogs as log}
						<div class="log-entry">{log}</div>
					{/each}
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.debug-overlay-container {
		position: fixed;
		bottom: 20px;
		right: 20px;
		z-index: 9999;
		font-family: monospace;
	}

	.debug-toggle {
		width: 48px;
		height: 48px;
		border-radius: 50%;
		background: rgba(0, 0, 0, 0.8);
		color: #0f0;
		border: 2px solid #0f0;
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
	}

	.debug-panel {
		width: 90vw;
		max-width: 400px;
		height: 300px;
		background: rgba(0, 0, 0, 0.9);
		border: 1px solid #333;
		border-radius: 8px;
		display: flex;
		flex-direction: column;
		color: #eee;
		box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.5);
	}

	.debug-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px;
		border-bottom: 1px solid #333;
		background: #111;
		border-radius: 8px 8px 0 0;
	}

	.debug-header h3 {
		margin: 0;
		font-size: 0.9rem;
		color: #0f0;
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.actions button {
		background: transparent;
		border: none;
		color: #aaa;
		cursor: pointer;
		padding: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.actions button:hover {
		color: white;
	}

	.debug-content {
		flex: 1;
		overflow-y: auto;
		padding: 8px;
		font-size: 0.8rem;
		word-break: break-all;
	}

	.log-entry {
		margin-bottom: 4px;
		border-bottom: 1px solid #222;
		padding-bottom: 2px;
	}

	.empty {
		color: #555;
		text-align: center;
		padding-top: 20px;
	}
</style>
