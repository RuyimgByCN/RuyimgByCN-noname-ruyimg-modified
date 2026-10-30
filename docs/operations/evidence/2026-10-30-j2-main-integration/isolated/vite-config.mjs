import {createRequire} from "node:module";
const requireFromApp=createRequire('/private/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-j2-llgkp_le/worktree/apps/web-client/package.json');
const {defineConfig}=requireFromApp("vite");
const vueModule=requireFromApp('@vitejs/plugin-vue');const vue=vueModule.default??vueModule;
const wasmModule=requireFromApp('vite-plugin-wasm');const wasm=wasmModule.default??wasmModule;
const topLevelAwaitModule=requireFromApp('vite-plugin-top-level-await');const topLevelAwait=topLevelAwaitModule.default??topLevelAwaitModule;
export default defineConfig({
  cacheDir:'/private/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-j2-llgkp_le/evidence/theft-correction/vite-cache',
  plugins: [vue(), wasm(), topLevelAwait()],
  // T13 产品面：默认原生后端；legacy 显式回退。
  define: {
    "import.meta.env.VITE_ENGINE_BACKEND": JSON.stringify(
      process.env.VITE_ENGINE_BACKEND ?? "native",
    ),
  },
  resolve: {
    alias: {
      "@": '/private/var/folders/b5/2mh_j5tj7m71xhcjsk2z1t4c0000gn/T/noname-j2-llgkp_le/worktree/apps/web-client/src',
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
