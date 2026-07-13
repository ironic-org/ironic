import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';
import mdx from 'fumadocs-mdx/vite';
import * as sourceConfig from './source.config';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default (async () => {
  const fumadocsPlugin = await mdx(sourceConfig as Record<string, unknown>);

  return defineConfig({
    base: process.env.VITE_CDN_URL || '/',
    plugins: [react(), tailwindcss(), fumadocsPlugin],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, '.'),
      },
    },
  });
})();
