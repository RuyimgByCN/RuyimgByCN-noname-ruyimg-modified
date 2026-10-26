// 临时构建配置：复用 web-client 的 vite 配置，原生 WASM 导向 scratchpad 重建产物，输出到 scratchpad。
import base from "<repo>/apps/web-client/vite.config.ts";
const pkg = "<scratchpad>/native-pkg/engine_native_wasm.js";
const redirect = {
  name: "perf-native-pkg-redirect",
  enforce: "pre",
  resolveId(source) {
    return source.endsWith("native-pkg/engine_native_wasm.js") ? pkg : null;
  },
  transform(code, id) {
    if (!id.endsWith("src/worker/nativeBackend.ts")) return null;
    return code.replace('"./native-pkg/engine_native_wasm_bg.wasm"', JSON.stringify("../../../../../../../../../..<scratchpad>/native-pkg/engine_native_wasm_bg.wasm"));
  },
};
export default {
  ...base,
  root: "<repo>/apps/web-client",
  plugins: [redirect, ...base.plugins],
  worker: { ...base.worker, plugins: () => [redirect, ...base.worker.plugins()] },
  build: { outDir: "<scratchpad>/web-dist", emptyOutDir: true },
};
