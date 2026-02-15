<script lang="ts">
  import Editor from "$lib/Editor.svelte";
  import StyleSelect from "$lib/StyleSelect.svelte";
  import InsertMenu from "$lib/InsertMenu.svelte";
  import MetadataDialog from "$lib/MetadataDialog.svelte";
  import { save, open } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    FolderOpen,
    Save,
    Info,
    Undo,
    Redo,
    Clipboard,
    Image,
    Table,
    List,
    ListOrdered,
    IndentIncrease,
    IndentDecrease,
    Quote,
  } from "lucide-svelte";

  let editorComponent: any = $state();
  let syncStatus = $state("Ready");
  let currentStyleId = $state("Normal Text");
  let currentPath = $state<string | null>(null);

  let metadata = $state({
    title: "",
    description: "",
    subject: "",
    creator: "",
    creationDate: "",
    generator: "",
  });
  let isMetadataOpen = $state(false);
  let isDirty = $state(false);

  // Derived title for the banner
  let displayTitle = $derived(metadata.title || "Untitled Document");

  $effect(() => {
    const title = metadata.title
      ? `${metadata.title} - AppThere Loki Text`
      : "AppThere Loki Text";
    getCurrentWindow().setTitle(title);
  });

  async function handleSave() {
    if (!editorComponent) return;

    let path = currentPath;
    if (!path) {
      path = await save({
        filters: [
          {
            name: "Flat ODT",
            extensions: ["fodt"],
          },
        ],
      });
      if (!path) return; // User cancelled
      currentPath = path;
    }

    syncStatus = "Saving...";
    try {
      await editorComponent.saveWithStyles(path);
      syncStatus = "Saved to disk";
    } catch (e) {
      console.error("Save failed", e);
      syncStatus = "Save Error";
    }
  }

  async function handleOpen() {
    if (!editorComponent) return;

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Flat ODT",
          extensions: ["fodt"],
        },
      ],
    });

    if (!selected) return;

    const path = Array.isArray(selected) ? selected[0] : selected;

    syncStatus = "Opening...";
    try {
      const response = await invoke("open_document", { path });
      editorComponent.loadWithStyles(response);
      currentPath = path;
      isDirty = false; // Reset dirty state on open
      syncStatus = "Opened";
    } catch (e) {
      console.error("Open failed", e);
      syncStatus = "Open Error";
    }
  }
</script>

<main class="app-container">
  <header class="app-header">
    <div class="brand">
      <span class="logo" title={displayTitle}>{displayTitle}</span>
    </div>
    <div class="header-actions">
      <button
        class="meta-btn"
        onclick={() => (isMetadataOpen = true)}
        title="Document Properties"
        aria-label="Document Properties"
      >
        <Info size={20} />
      </button>
      <button
        class="open-btn"
        onclick={handleOpen}
        title="Open document"
        aria-label="Open document"
      >
        <FolderOpen size={20} />
      </button>
      <button
        class="save-btn"
        onclick={handleSave}
        title="Save document"
        aria-label="Save document"
      >
        <Save size={20} />
      </button>
    </div>
  </header>

  <div class="content-view">
    <Editor
      bind:this={editorComponent}
      bind:status={syncStatus}
      bind:currentStyleId
      bind:metadata
      onChange={() => (isDirty = true)}
    />
  </div>

  <MetadataDialog
    isOpen={isMetadataOpen}
    bind:metadata
    onClose={() => (isMetadataOpen = false)}
  />

  <div class="bottom-toolbar">
    <div class="toolbar-scroll-container">
      <div class="toolbar-controls">
        <div class="history-controls">
          <button
            class="icon-btn"
            onclick={() => editorComponent?.undo()}
            title="Undo (Ctrl+Z)"
            aria-label="Undo"
          >
            <Undo size={18} />
          </button>
          <button
            class="icon-btn"
            onclick={() => editorComponent?.redo()}
            title="Redo (Ctrl+Shift+Z)"
            aria-label="Redo"
          >
            <Redo size={18} />
          </button>
        </div>
        <div class="divider"></div>
        <button
          class="icon-btn"
          onclick={() => editorComponent?.paste()}
          title="Paste (Ctrl+V)"
          aria-label="Paste"
        >
          <Clipboard size={18} />
        </button>

        <StyleSelect
          bind:currentStyleId
          onSelect={(id: string) => editorComponent?.applyStyle(id)}
          onEdit={() => editorComponent?.openStyles()}
        />

        <div class="divider"></div>

        <InsertMenu
          onInsertImage={() => editorComponent?.insertImage()}
          onInsertTable={() => editorComponent?.insertTable()}
        />

        <div class="group-spacer"></div>

        <button
          class="icon-btn"
          onclick={() => editorComponent?.toggleBulletList()}
          title="Bulleted List"
        >
          <List size={18} />
        </button>
        <button
          class="icon-btn"
          onclick={() => editorComponent?.toggleOrderedList()}
          title="Numbered List"
        >
          <ListOrdered size={18} />
        </button>
        <!-- Blockquote removed as requested -->

        <div class="mini-divider"></div>

        <button
          class="icon-btn"
          onclick={() => editorComponent?.indent()}
          title="Increase Indent"
        >
          <IndentIncrease size={18} />
        </button>
        <button
          class="icon-btn"
          onclick={() => editorComponent?.outdent()}
          title="Decrease Indent"
        >
          <IndentDecrease size={18} />
        </button>
      </div>
    </div>
  </div>

  <footer class="app-footer">
    <div class="footer-left">
      <span class="status-indicator">
        {currentPath ? currentPath.split("/").pop() : "New File"}
        {#if isDirty}
          <span class="dirty-indicator">•</span>
        {/if}
      </span>
    </div>
    <div class="footer-right">
      <span>v0.1.0</span>
    </div>
  </footer>
</main>

<style>
  :root {
    --primary-color: #3b82f6;
    --bg-color: #f3f4f6;
    --text-color: #1f2937;
    --header-bg: #ffffff;
    --border-color: #e5e7eb;
    --icon-color: #4b5563;
    --hover-bg: #f9fafb;

    --header-height: 56px;
    --footer-height: 28px;
    --toolbar-height: 64px;
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --primary-color: #60a5fa;
      --bg-color: #1c1917; /* Stone-900 */
      --text-color: #f5f5f4; /* Stone-100 */
      --header-bg: #292524; /* Stone-800 */
      --border-color: #44403c; /* Stone-700 */
      --icon-color: #a8a29e; /* Stone-400 */
      --hover-bg: #44403c; /* Stone-700 */
      --document-bg: #000000; /* Pure black */
      --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.3);
      --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.4);
    }
  }

  :global(body) {
    margin: 0;
    padding: 0;
    background-color: var(--bg-color);
    color: var(--text-color);
    font-family:
      "Inter",
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      Roboto,
      sans-serif;
    overflow: hidden; /* Prevent body scroll, use content-view scroll */
  }

  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .app-header {
    height: var(--header-height);
    display: flex;
    align-items: center;
    padding: 0 24px;
    background: var(--header-bg);
    border-bottom: 1px solid var(--border-color);
    justify-content: space-between;
    z-index: 10;
  }

  .brand {
    flex: 1; /* Allow brand to take available space */
    min-width: 0; /* Enable truncation for flex child */
    margin-right: 16px;
  }

  .logo {
    font-weight: 800;
    font-size: 1.1rem;
    letter-spacing: -0.025em;
    color: var(--text-color);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block; /* Required for truncation */
  }

  .content-view {
    flex: 1;
    overflow-y: auto;
    padding: 40px 0;
    display: flex;
    justify-content: center;
    background-color: var(--bg-color);
  }

  @media (prefers-color-scheme: dark) {
    .content-view {
      background-color: var(--document-bg);
    }
  }

  .bottom-toolbar {
    height: var(--toolbar-height);
    background: var(--header-bg);
    border-top: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    padding: 0 24px;
    z-index: 10;
    box-shadow: 0 -1px 3px rgba(0, 0, 0, 0.05);
  }

  .toolbar-scroll-container {
    width: 100%;
    overflow-x: auto;
    /* Hide scrollbar for Chrome, Safari and Opera */
    &::-webkit-scrollbar {
      display: none;
    }
    /* Hide scrollbar for IE, Edge and Firefox */
    -ms-overflow-style: none; /* IE and Edge */
    scrollbar-width: none; /* Firefox */
  }

  .toolbar-controls {
    display: flex;
    align-items: center;
    gap: 8px; /* Slightly tighter gap */
    max-width: 800px;
    margin: 0 auto;
    width: max-content; /* Allow growing beyond container width */
    min-width: 100%; /* But take full width if smaller */
    padding: 0 4px; /* Minimal padding */
  }

  /* Adjust gap/margin specifically for larger screens to center better if needed */
  @media (min-width: 850px) {
    .toolbar-controls {
      width: 100%;
      justify-content: center;
    }
  }

  .group-spacer {
    display: inline-block;
    width: 12px;
  }

  .mini-divider {
    width: 1px;
    height: 16px;
    background: var(--border-color);
    margin: 0 4px;
  }

  .divider {
    width: 1px;
    height: 24px;
    background: var(--border-color);
  }

  .app-footer {
    height: var(--footer-height);
    background: var(--header-bg);
    border-top: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px;
    font-size: 0.8rem;
    color: var(--icon-color);
  }

  .status-indicator {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
  }

  .dirty-indicator {
    color: var(
      --text-color
    ); /* Or a specific color like orange/red if preferred */
    font-size: 1.2rem;
    line-height: 0.5;
  }

  /* Shared button styles */
  .save-btn,
  .open-btn,
  .meta-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    padding: 0;
    background: var(--header-bg);
    color: var(--icon-color);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .save-btn:hover,
  .open-btn:hover,
  .meta-btn:hover {
    background: var(--hover-bg);
    border-color: var(--icon-color);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }

  .save-btn:active,
  .open-btn:active,
  .meta-btn:active {
    transform: translateY(0);
  }

  /* Specific save button styles removed to match other buttons */
  /* .save-btn inherits from shared block above */

  .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  @media (max-width: 640px) {
    .save-btn,
    .open-btn,
    .meta-btn {
      width: 36px;
      height: 36px;
    }

    .content-view {
      padding: 0;
    }
    .app-header {
      padding: 0 16px;
    }
    .bottom-toolbar {
      padding: 0 12px;
    }
  }

  .history-controls {
    display: flex;
    gap: 4px;
    align-items: center;
  }

  .icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    color: var(--icon-color);
    cursor: pointer;
    transition: all 0.2s;
  }

  .icon-btn:hover {
    background: var(--hover-bg);
    color: var(--text-color);
  }

  .icon-btn:active {
    background: var(--border-color);
  }
</style>
