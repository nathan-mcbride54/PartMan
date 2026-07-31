import { fileURLToPath, URL } from "node:url";

import react from "@vitejs/plugin-react";
import { defineConfig, searchForWorkspaceRoot } from "vite";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  resolve: {
    dedupe: ["react", "react-dom"],
    alias: {
      "@partman/design-tokens": fileURLToPath(
        new URL("../../packages/design-tokens/src/generated.ts", import.meta.url),
      ),
      "@partman/ui": fileURLToPath(
        new URL("../../packages/ui/src/index.ts", import.meta.url),
      ),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: process.env.TAURI_DEV_HOST || false,
    fs: {
      allow: [searchForWorkspaceRoot(process.cwd()), repositoryRoot],
    },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
