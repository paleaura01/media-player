import { defineConfig } from "vite";
import path from "path";

export default defineConfig({
  root: "./renderer",
  base: "./",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "./renderer/index.html"),
      },
      external: ["electron"], // Prevent Electron modules from being bundled
    },
  },
  server: {
    port: 3000,
    strictPort: true,
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./renderer"),
    },
  },
  optimizeDeps: {
    exclude: ["electron"], // Exclude Electron from dependency optimization
  },
});
