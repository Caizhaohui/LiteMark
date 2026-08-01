import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Vite config for the LiteMark Tauri 2 webview.
//
// Tauri's dev server expects the frontend on a fixed port with strictPort so
// the Rust shell can connect deterministically during `pnpm tauri dev`.
// `clearScreen: false` keeps Rust compiler output visible in the shared
// terminal. `host` must be loopback — the webview is local only.
export default defineConfig({
  plugins: [react()],
  // Tauri expects a fixed port; fail instead of silently picking another.
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    hmr: {
      protocol: "ws",
      host: "127.0.0.1",
      port: 1421,
    },
  },
  envDir: ".",
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
  },
});
