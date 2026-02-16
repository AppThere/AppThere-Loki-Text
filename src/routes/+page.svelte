<script lang="ts">
  import Editor from "$lib/Editor.svelte";
  import StyleSelect from "$lib/StyleSelect.svelte";
  import InsertMenu from "$lib/InsertMenu.svelte";
  import MetadataDialog from "$lib/MetadataDialog.svelte";
  import AboutDialog from "$lib/AboutDialog.svelte";
  import SettingsDialog from "$lib/SettingsDialog.svelte";
  import LandingPage from "$lib/LandingPage.svelte";
  import { recentDocs } from "$lib/recentDocs";
  import { TEMPLATES } from "$lib/templates";
  import { styleRegistry } from "$lib/styleStore";
  import { save, open, ask } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, tick } from "svelte";
  import {
    FolderOpen,
    Save,
    FileDown,
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
    Menu,
    Plus,
    Settings,
    X,
  } from "lucide-svelte";

  let editorComponent: any = $state();
  let syncStatus = $state("Ready");
  let currentStyleId = $state("Normal Text");
  let currentPath = $state<string | null>(null);

  let metadata = $state({
    identifier: crypto.randomUUID(),
    title: "Untitled Document",
    language: "en",
    creator: "Unknown Author",
    creationDate: new Date().toISOString(),
    generator: "AppThere Loki",
    description: "",
    subject: "",
  });

  let isMetadataOpen = $state(false);
  let isAboutOpen = $state(false);
  let isSettingsOpen = $state(false);
  let isMenuOpen = $state(false);

  let view = $state<"landing" | "editor">("landing");
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
            name: "OpenDocument Text",
            extensions: ["odt"],
          },
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

  async function handleExportEpub() {
    console.log("handleExportEpub called");
    if (!editorComponent) {
      console.error("Editor component not ready");
      return;
    }

    const path = await save({
      filters: [
        {
          name: "EPUB",
          extensions: ["epub"],
        },
      ],
    });

    if (!path) {
      console.log("User cancelled EPUB export");
      return; // User cancelled
    }

    console.log("Exporting to:", path);
    syncStatus = "Exporting to EPUB...";

    try {
      // Detect which fonts are used in styles
      const fontPaths: string[] = [];
      const styles = $styleRegistry;
      const fontDir =
        "/Users/kevin/.gemini/antigravity/scratch/appthere-loki/static/fonts";

      const FONT_MAP: Record<string, string[]> = {
        "Courier Prime": [
          "CourierPrime-Regular.ttf",
          "CourierPrime-Bold.ttf",
          "CourierPrime-Italic.ttf",
          "CourierPrime-BoldItalic.ttf",
        ],
        "Atkinson Hyperlegible Next": [
          "AtkinsonHyperlegibleNext-Regular.ttf",
          "AtkinsonHyperlegibleNext-Bold.ttf",
          "AtkinsonHyperlegibleNext-Italic.ttf",
          "AtkinsonHyperlegibleNext-BoldItalic.ttf",
        ],
        "Public Sans": [
          "PublicSans-Variable.ttf",
          "PublicSans-Italic-Variable.ttf",
        ],
        Newsreader: [
          "Newsreader-Variable.ttf",
          "Newsreader-Italic-Variable.ttf",
        ],
        "Cormorant Garamond": [
          "CormorantGaramond-Variable.ttf",
          "CormorantGaramond-Italic-Variable.ttf",
        ],
        Geist: ["Geist-Variable.ttf"],
        "Bodoni Moda": [
          "BodoniModa-Variable.ttf",
          "BodoniModa-Italic-Variable.ttf",
        ],
        Lexend: ["Lexend-Variable.ttf"],
        Caveat: ["Caveat-Variable.ttf"],
        "Roboto Flex": ["RobotoFlex-Variable.ttf"],
        Bitter: ["Bitter-Variable.ttf", "Bitter-Italic-Variable.ttf"],
      };

      Object.entries(FONT_MAP).forEach(([family, files]) => {
        if (styles.some((s) => s.fontFamily && s.fontFamily.includes(family))) {
          files.forEach((f) => fontPaths.push(`${fontDir}/${f}`));
        }
      });

      // Get current document state
      const tiptapJson = editorComponent.getJSON();

      // Convert styles to StyleDefinition format (same as saveWithStyles)
      const stylesMap: Record<string, any> = {};
      styles.forEach((s) => {
        let attributes: Record<string, string> = {};
        if (s.fontFamily) attributes["fo:font-family"] = s.fontFamily;
        if (s.fontSize) attributes["fo:font-size"] = s.fontSize;
        if (s.fontWeight) attributes["fo:font-weight"] = s.fontWeight;
        if (s.lineHeight) attributes["fo:line-height"] = s.lineHeight;
        if (s.marginLeft) attributes["fo:margin-left"] = s.marginLeft;
        if (s.marginRight) attributes["fo:margin-right"] = s.marginRight;
        if (s.marginTop) attributes["fo:margin-top"] = s.marginTop;
        if (s.marginBottom) attributes["fo:margin-bottom"] = s.marginBottom;
        if (s.textIndent) attributes["fo:text-indent"] = s.textIndent;
        if (s.textAlign) attributes["fo:text-align"] = s.textAlign;
        if (s.textTransform) attributes["fo:text-transform"] = s.textTransform;
        if (s.hyphenate !== undefined)
          attributes["fo:hyphenate"] = String(s.hyphenate);
        if (s.orphans !== undefined)
          attributes["fo:orphans"] = String(s.orphans);
        if (s.widows !== undefined) attributes["fo:widows"] = String(s.widows);
        if (s.basedOn) attributes["style:parent-style-name"] = s.basedOn;
        if (s.next) attributes["style:next-style-name"] = s.next;

        stylesMap[s.id] = {
          name: s.id,
          family: "Paragraph",
          parent: s.basedOn || null,
          displayName: s.displayName || null,
          attributes,
          textTransform: s.textTransform || null,
        };
      });

      console.log("Calling save_epub with:", { path, fontPaths, metadata });

      await invoke("save_epub", {
        path,
        tiptapJson,
        styles: stylesMap,
        metadata: metadata,
        fontPaths: fontPaths,
      });

      console.log("EPUB export successful");
      syncStatus = "Exported to EPUB successfully";
      setTimeout(() => {
        syncStatus = "Ready";
      }, 2000);
    } catch (e) {
      console.error("EPUB export failed", e);
      syncStatus = "EPUB Export Error: " + String(e);
    }
  }

  async function handleOpen() {
    console.log("handleOpen called");

    if (isDirty) {
      const confirmed = await ask(
        "You have unsaved changes. Are you sure you want to open another document?",
        {
          title: "Open Document",
          kind: "warning",
        },
      );
      if (!confirmed) return;
    }

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "OpenDocument Text",
          extensions: ["odt", "fodt"],
        },
      ],
    });

    if (!selected) return;

    const path = Array.isArray(selected) ? selected[0] : selected;
    console.log("Opening document:", path);

    syncStatus = "Opening...";
    try {
      const response = await invoke("open_document", { path });
      console.log("open_document response:", response);

      const {
        content,
        styles,
        metadata: loadedMetadata,
      } = response as { content: any; styles: any; metadata: any };

      metadata = loadedMetadata;

      view = "editor";
      await tick();

      if (!editorComponent) {
        console.error("Editor component not ready after switching view");
        syncStatus = "Error: Editor not ready";
        return;
      }

      editorComponent.loadWithStyles({
        content,
        styles,
        metadata: loadedMetadata,
      });

      await recentDocs.add(
        path,
        metadata.title || path.split("/").pop() || "Untitled",
      );

      currentPath = path;
      isDirty = false;
      syncStatus = "Opened";
    } catch (e) {
      console.error("Open failed", e);
      syncStatus = "Open Error";
    }
  }

  async function handleNewDocument(templateId?: string) {
    console.log("handleNewDocument called with:", templateId);
    if (isDirty) {
      const confirmed = await ask(
        "You have unsaved changes. Are you sure you want to create a new document?",
        {
          title: "New Document",
          kind: "warning",
        },
      );
      if (!confirmed) return;
    }

    // Reset state
    currentPath = null;
    isDirty = false;
    syncStatus = "Ready";
    metadata = {
      identifier: crypto.randomUUID(),
      title: "Untitled Document",
      language: "en",
      creator: "Unknown Author",
      creationDate: new Date().toISOString(),
      generator: "AppThere Loki",
      description: "",
      subject: "",
    };

    let initialStyles = [];
    if (templateId) {
      const template = TEMPLATES.find((t) => t.id === templateId);
      if (template && template.styles.length > 0) {
        styleRegistry.setStyles(template.styles);
        initialStyles = template.styles;
      } else {
        styleRegistry.reset();
        initialStyles = $styleRegistry;
      }
    } else {
      styleRegistry.reset();
      initialStyles = $styleRegistry;
    }

    let startStyle = "Normal Text";
    if (templateId === "screenplay") {
      startStyle = "Scene Heading";
    }

    const emptyDoc = {
      type: "doc",
      content: [
        {
          type: "paragraph",
          attrs: { styleName: startStyle },
        },
      ],
    };

    view = "editor";
    await tick();

    editorComponent.loadWithStyles({
      content: emptyDoc,
      styles: initialStyles.reduce((acc: any, s: any) => {
        acc[s.id] = s;
        return acc;
      }, {}),
      metadata: metadata,
    });

    currentStyleId = startStyle;
    isMenuOpen = false;
  }

  async function handleOpenRecent(path: string) {
    if (isDirty) {
      const confirmed = await ask(
        "You have unsaved changes. Are you sure you want to open another document?",
        {
          title: "Open Recent",
          kind: "warning",
        },
      );
      if (!confirmed) return;
    }

    try {
      const result = await invoke("open_document", { path });
      // @ts-ignore
      const { content, styles, metadata: meta } = result;

      metadata = meta;
      view = "editor";
      await tick();

      editorComponent.loadWithStyles({
        content: JSON.parse(content),
        styles: styles,
        metadata: metadata,
      });

      currentPath = path;
      syncStatus = "Ready";
      isDirty = false;
      await recentDocs.add(path, metadata.title);
    } catch (e) {
      console.error("Failed to open recent:", e);
      await ask(
        `Failed to open document: ${path}\nIt may have been moved or deleted.`,
        {
          title: "Error",
          kind: "error",
        },
      );
    }
  }

  async function scrollEditorToTop() {
    await tick();
    const contentView = document.querySelector(".content-view");
    if (contentView) {
      contentView.scrollTop = 0;
    }
  }

  // Session Preservation
  async function saveSession() {
    if (view === "landing") {
      localStorage.removeItem("loki_session");
      return;
    }

    if (!editorComponent) return;

    try {
      const sessionData = {
        view,
        currentPath,
        metadata,
        isDirty,
        content: editorComponent.getJSON(),
        styles: $styleRegistry,
        currentStyleId,
        timestamp: Date.now(),
      };
      localStorage.setItem("loki_session", JSON.stringify(sessionData));
      console.log("Session saved");
    } catch (e) {
      console.error("Failed to save session:", e);
    }
  }

  async function restoreSession() {
    const data = localStorage.getItem("loki_session");
    if (!data) return;

    try {
      const session = JSON.parse(data);
      // Only restore if recently saved (e.g. within 24 hours) or always?
      // Let's go with always for reliable lifecycle handling.

      console.log("Restoring session...");
      currentPath = session.currentPath;
      metadata = session.metadata;
      isDirty = session.isDirty;
      currentStyleId = session.currentStyleId;
      view = session.view;

      if (session.styles) {
        styleRegistry.setStyles(session.styles);
      }

      await tick();
      if (editorComponent && session.content) {
        editorComponent.loadWithStyles({
          content: session.content,
          styles: (session.styles || []).reduce((acc: any, s: any) => {
            acc[s.id] = s;
            return acc;
          }, {}),
          metadata: metadata,
        });
        console.log("Editor content restored");
      }
    } catch (e) {
      console.error("Failed to restore session:", e);
      localStorage.removeItem("loki_session");
    }
  }

  async function closeDocument() {
    if (isDirty) {
      const confirmed = await ask(
        "You have unsaved changes. Are you sure you want to close?",
        {
          title: "Close Document",
          kind: "warning",
        },
      );
      if (!confirmed) return;
    }
    view = "landing";
    currentPath = null;
    isDirty = false;
    isMenuOpen = false;
    localStorage.removeItem("loki_session");
  }

  function toggleMenu() {
    isMenuOpen = !isMenuOpen;
  }

  function closeMenu() {
    isMenuOpen = false;
  }

  onMount(() => {
    restoreSession();

    const handleVisibilityChange = () => {
      if (document.visibilityState === "hidden") {
        saveSession();
      }
    };

    const handleKeyboardShortcuts = (e: KeyboardEvent) => {
      const isMod = e.ctrlKey || e.metaKey;

      if (isMod) {
        switch (e.key.toLowerCase()) {
          case "s":
            e.preventDefault();
            if (e.shiftKey) {
              handleExportEpub();
            } else {
              handleSave();
            }
            break;
          case "o":
            e.preventDefault();
            handleOpen();
            break;
          case "n":
            e.preventDefault();
            handleNewDocument();
            break;
          case "z":
            // Tiptap usually handles this, but we can explicitly trigger it if focus is elsewhere
            // or just let it bubble if we want editor to handle it.
            // However, redo is often Ctrl+Shift+Z or Ctrl+Y.
            if (e.shiftKey) {
              e.preventDefault();
              editorComponent?.redo();
            }
            break;
          case "y":
            e.preventDefault();
            editorComponent?.redo();
            break;
        }
      }
    };

    window.addEventListener("keydown", handleKeyboardShortcuts);
    document.addEventListener("visibilitychange", handleVisibilityChange);

    if (window.visualViewport) {
      const handleResize = () => {
        if (window.visualViewport) {
          document.documentElement.style.setProperty(
            "--viewport-height",
            `${window.visualViewport.height}px`,
          );
        }
      };

      window.visualViewport.addEventListener("resize", handleResize);
      handleResize();

      return () => {
        window.removeEventListener("keydown", handleKeyboardShortcuts);
        document.removeEventListener(
          "visibilitychange",
          handleVisibilityChange,
        );
        window.visualViewport?.removeEventListener("resize", handleResize);
      };
    }

    return () => {
      window.removeEventListener("keydown", handleKeyboardShortcuts);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  });
</script>

{#if view === "landing"}
  <LandingPage
    onNew={handleNewDocument}
    onOpen={handleOpen}
    onOpenRecent={handleOpenRecent}
  />
{:else}
  <main class="app-container">
    <header class="app-header">
      <div class="brand">
        <button
          class="meta-btn"
          onclick={() => (isMetadataOpen = true)}
          title="Document Properties"
          aria-label="Document Properties"
        >
          <Info size={18} />
        </button>
        <span class="logo" title={displayTitle}>{displayTitle}</span>
      </div>
      <div class="header-actions">
        <div class="menu-container">
          <button
            class="menu-btn"
            onclick={toggleMenu}
            title="Menu"
            aria-label="App Menu"
          >
            <Menu size={20} />
          </button>

          {#if isMenuOpen}
            <div
              class="menu-backdrop"
              onclick={closeMenu}
              role="presentation"
            ></div>
            <div class="menu-dropdown">
              <button onclick={handleOpen} class="menu-item">
                <FolderOpen size={16} />
                <span>Open Document</span>
              </button>
              <button class="menu-item" onclick={() => handleSave()}>
                <Save size={18} />
                <span>Save Document</span>
              </button>
              <button class="menu-item" onclick={() => handleExportEpub()}>
                <FileDown size={18} />
                <span>Export as EPUB</span>
              </button>
              <button onclick={closeDocument} class="menu-item">
                <X size={16} />
                <span>Close Document</span>
              </button>
              <div class="menu-divider"></div>
              <button onclick={() => handleNewDocument()} class="menu-item">
                <Plus size={16} />
                <span>New Document</span>
              </button>
              <button
                onclick={() => {
                  isSettingsOpen = true;
                  closeMenu();
                }}
                class="menu-item"
              >
                <Settings size={16} />
                <span>Settings</span>
              </button>
              <button
                onclick={() => {
                  isAboutOpen = true;
                  closeMenu();
                }}
                class="menu-item"
              >
                <Info size={16} />
                <span>About</span>
              </button>
            </div>
          {/if}
        </div>
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
    <MetadataDialog
      isOpen={isMetadataOpen}
      bind:metadata
      onClose={() => (isMetadataOpen = false)}
    />
    <AboutDialog bind:isOpen={isAboutOpen} />
    <SettingsDialog bind:isOpen={isSettingsOpen} />
  </main>
{/if}

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
    :root:not(:global(.light)) {
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

  :global(html.dark) {
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

  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    background-color: var(--bg-color);
    color: var(--text-color);
    background-color: var(--bg-color);
    color: var(--text-color);
    overflow: hidden; /* Prevent body scroll, use content-view scroll */
    height: 100%;
    width: 100%;
  }

  :global(*),
  :global(*::before),
  :global(*::after) {
    box-sizing: border-box;
  }

  .app-container {
    display: flex;
    flex-direction: column;
    height: var(--viewport-height, 100vh);
    width: 100%;
    overflow: hidden; /* Ensure no double scroll */
  }

  .app-header {
    height: calc(var(--header-height) + max(env(safe-area-inset-top), 24px));
    display: flex;
    align-items: center;
    padding: max(env(safe-area-inset-top), 24px) 24px 0 24px;
    background: var(--header-bg);
    border-bottom: 1px solid var(--border-color);
    justify-content: space-between;
    z-index: 10;
  }

  .brand {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
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
    :root:not(:global(.light)) .content-view {
      background-color: var(--document-bg);
    }
  }

  :global(html.dark) .content-view {
    background-color: var(--document-bg);
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
    height: calc(var(--footer-height) + env(safe-area-inset-bottom, 0px));
    background: var(--header-bg);
    border-top: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px env(safe-area-inset-bottom, 0px) 24px;
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
  .meta-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    background: transparent;
    color: var(--icon-color);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    flex-shrink: 0;
  }

  .meta-btn:hover {
    background: var(--hover-bg);
    border-color: var(--icon-color);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }

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
    .meta-btn {
      width: 36px;
      height: 36px;
    }

    .content-view {
      padding: 0;
    }
    .app-header {
      padding: max(env(safe-area-inset-top), 12px) 16px 8px 16px;
      align-items: flex-end;
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

  .menu-container {
    position: relative;
    margin-left: 8px;
  }

  .menu-btn {
    background: transparent;
    border: none;
    color: var(--icon-color);
    cursor: pointer;
    padding: 8px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition:
      background-color 0.2s,
      color 0.2s;
  }

  .menu-btn:hover {
    background-color: var(--hover-bg);
    color: var(--text-color);
  }

  .menu-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: 90;
    background: transparent;
  }

  .menu-dropdown {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 8px;
    background: var(--bg-color);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    box-shadow: var(--shadow-md);
    min-width: 180px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    padding: 4px;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: transparent;
    border: none;
    color: var(--text-color);
    cursor: pointer;
    text-align: left;
    font-size: 0.9rem;
    border-radius: 4px;
    width: 100%;
  }

  .menu-item:hover {
    background-color: var(--hover-bg);
  }

  .menu-divider {
    height: 1px;
    background-color: var(--border-color);
    margin: 4px 0;
  }
</style>
