<script>
    import "../app.css";
    import { settingsStore } from "$lib/settingsStore";

    let { children } = $props();

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

{@render children()}
