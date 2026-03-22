/// <reference types="vitest" />
import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';
import path from "path";

/**
 * Watches public/locales/en/*.json and logs a reminder to re-run the TypeScript
 * check whenever the English reference locale files change.
 */
function localeWatcherPlugin(): Plugin {
  return {
    name: 'locale-watcher',
    handleHotUpdate({ file, server }) {
      if (file.includes('/public/locales/en/')) {
        server.config.logger.info(
          '[locale-watcher] English locale file changed — run `npx tsc --noEmit` to refresh type safety.',
          { timestamp: true },
        );
      }
    },
  };
}

export default defineConfig({
  plugins: [react(), localeWatcherPlugin()],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Tauri expects files in dist/
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },

  // public/ is Vite's default publicDir — locale JSON files in
  // public/locales/ are served as-is at /locales/{lng}/{ns}.json in dev
  // and copied verbatim into dist/ during production builds.

  // Prevent vite from obscuring rust errors
  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
  },

  test: {
    environment: 'jsdom',
    include: ['src/**/*.test.{ts,tsx}'],
  },

  envPrefix: ['VITE_', 'TAURI_'],
});
