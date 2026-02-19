<script lang="ts">
    import "../app.css";
    import { settingsStore } from "$lib/settingsStore";
    import { onMount } from "svelte";
    import DebugOverlay from "$lib/DebugOverlay.svelte";
    import { listen } from "@tauri-apps/api/event";
    import { addDebugLog } from "$lib/debugStore";

    let { children } = $props();

    onMount(() => {
        console.log("FRONTEND_STARTUP: layout.svelte onMount");
        addDebugLog("App mounted. Waiting for events...");

        let unlisten: (() => void) | undefined;

        const setup = async () => {
            // Listen for backend debug events
            unlisten = await listen<string>("debug_log", (event) => {
                addDebugLog(event.payload);
            });
        };
        setup();

        return () => {
            if (unlisten) unlisten();
        };
    });

    $effect(() => {
        const root = document.documentElement;
        const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

        function applyTheme() {
            const theme = $settingsStore.theme;
            if (theme === "dark") {
                root.classList.add("dark");
                root.classList.remove("light");
            } else if (theme === "light") {
                root.classList.add("light");
                root.classList.remove("dark");
            } else {
                root.classList.remove("dark");
                root.classList.remove("light");
                if (mediaQuery.matches) {
                    root.classList.add("dark");
                }
            }
        }

        applyTheme();
        mediaQuery.addEventListener("change", applyTheme);
        return () => mediaQuery.removeEventListener("change", applyTheme);
    });
</script>

{#if import.meta.env.DEV}
    <DebugOverlay />
{/if}
{@render children()}
