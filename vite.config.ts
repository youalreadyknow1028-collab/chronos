import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  // REQUIRED for Tauri production builds — without this, Vite emits absolute
  // paths like /assets/index-xxx.js which resolve to the filesystem root in a
  // WebView (file:///C:/assets/...), not the app's dist/ folder.
  base: "./",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    // Ensure assets are output relative to index.html
    assetsDir: "assets",
    cssCodeSplit: true,
  },
});
