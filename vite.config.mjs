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
      external: ["speaker", "fluent-ffmpeg"],
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
    include: ["howler"],
    esbuildOptions: {
      target: "es2020",
    },
  },
});
