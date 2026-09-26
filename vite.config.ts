import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import electron from 'vite-plugin-electron-renderer';
import { resolve } from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    electron()
  ],
  server: {
    watch: {
      // Le watcher garde un handle sur chaque dossier : sous Windows, il bloque le renommage de release/win-unpacked.tmp par electron-builder
      ignored: ['**/release/**'],
    },
  },
  optimizeDeps: {
    exclude: ['lucide-react'],
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      external: [
        "sharp"
      ],
      input: {
        app: resolve(__dirname, 'index.html'),
        preload: resolve(__dirname, 'electron/preload.js'),
      },
    },
  },
});