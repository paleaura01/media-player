// vite.config.js

import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  root: './renderer', // Root for renderer files
  base: './', // Use relative paths for loading assets in production
  build: {
    outDir: '../dist', // Output directory
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, './renderer/index.html'),
      },
    },
  },
  server: {
    port: 3000, // Port for development server
    strictPort: true, // Ensure the port is not changed
  },
});
