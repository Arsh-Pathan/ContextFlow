import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri runs the UI from a webview against a fixed dev-server port.
// See https://v2.tauri.app/start/frontend/vite/ for the canonical config.
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Tauri expects a fixed port and fails if the port is unavailable.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Ignore the Rust source tree — `cargo` watches that itself.
      ignored: ["**/src-tauri/**"],
    },
  },
}));
