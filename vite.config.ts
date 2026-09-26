/// <reference types="vitest/config" />
import { defineConfig, type Plugin } from 'vite';
import react from '@vitejs/plugin-react';
import { readFileSync } from 'fs';

const { version } = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8'));

// CSP de production : la balise d'index.html garde 'unsafe-inline' et le WebSocket
// dont le serveur de développement (HMR) a besoin ; le build n'utilise ni l'un ni l'autre
const PRODUCTION_CSP = [
  "default-src 'self'",
  "script-src 'self'",
  "style-src 'self'",
  "img-src 'self' blob: data: local-file:",
  "font-src 'self'",
  "connect-src 'self'",
  "object-src 'none'",
  "base-uri 'none'",
  "form-action 'none'",
].join('; ') + ';';

function productionContentSecurityPolicy(): Plugin {
  return {
    name: 'production-content-security-policy',
    apply: 'build',
    transformIndexHtml(html) {
      const cspMeta = /(<meta http-equiv="Content-Security-Policy" content=")[^"]*(")/;
      // Échouer plutôt que de publier une page sans CSP
      if (!cspMeta.test(html)) {
        throw new Error('Balise Content-Security-Policy introuvable dans index.html');
      }
      return html.replace(cspMeta, `$1${PRODUCTION_CSP}$2`);
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig({
  // Chemins relatifs : la page est chargée en file:// dans l'application packagée
  base: './',
  plugins: [
    react(),
    productionContentSecurityPolicy(),
  ],
  define: {
    // Seule la version est injectée, pas tout le package.json
    __APP_VERSION__: JSON.stringify(version),
  },
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
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts', 'electron/**/*.test.js'],
  },
});
