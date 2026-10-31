import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { resolve } from "node:path";

export default defineConfig({
  plugins: [vue(), wasm(), topLevelAwait()],
  // T13 产品面：默认原生后端；legacy 显式回退。
  define: {
    "import.meta.env.VITE_ENGINE_BACKEND": JSON.stringify(
      process.env.VITE_ENGINE_BACKEND ?? "native",
    ),
  },
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
  worker: {
    format: "es",
    plugins: () => [wasm(), topLevelAwait()],
  },
  server: {
    port: 5173,
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
  optimizeDeps: {
    exclude: ["../src/wasm-pkg/engine_wasm.js"],
  },
  assetsInclude: ["**/*.wasm"],
});
