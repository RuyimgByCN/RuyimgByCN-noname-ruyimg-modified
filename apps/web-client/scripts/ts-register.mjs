// 注册 TS 装载钩子（见 ts-hook.mjs）。用法：
//   node --import ./scripts/ts-register.mjs --test
//   node --import ./scripts/ts-register.mjs scripts/native-worker-smoke.mjs
//
// Node 26 起 `module.register` 已标记弃用，优先用同步的 `registerHooks`
// （Node ≥22.15/23.5）；CI 的 Node 20 走 `register` 回退路径。
import * as nodeModule from "node:module";

import { loadSync } from "./ts-hook.mjs";

if (typeof nodeModule.registerHooks === "function") {
  nodeModule.registerHooks({ load: loadSync });
} else {
  nodeModule.register("./ts-hook.mjs", import.meta.url);
}
