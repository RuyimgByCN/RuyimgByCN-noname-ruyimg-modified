// node:test 的 TS 装载钩子：让测试/冒烟直接 import src 下的 .ts 生产源码。
//
// 为什么需要：CI 固定 Node 20（.github/workflows/ci.yml），没有内建 type stripping；
// 若用 Node 22+ 的 `--experimental-strip-types`，同一命令在 CI 上会直接失败。
// 这里复用 web-client 已有的 typescript devDependency 做逐文件转译，Node ≥20.6 可用。
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

import ts from "typescript";

/** 转译一个 .ts/.mts 文件为 ESM 源码。 */
function transpile(url) {
  const fileName = fileURLToPath(url);
  return ts.transpileModule(readFileSync(fileName, "utf8"), {
    fileName,
    compilerOptions: {
      module: ts.ModuleKind.ESNext,
      target: ts.ScriptTarget.ES2022,
      verbatimModuleSyntax: false,
    },
  }).outputText;
}

/** 异步装载钩子（module.register 路径，Node ≥20.6）。 */
export async function load(url, context, nextLoad) {
  if (!url.endsWith(".ts")) return nextLoad(url, context);
  return { format: "module", source: transpile(url), shortCircuit: true };
}

/** 同步装载钩子（module.registerHooks 路径，Node ≥22.15/23.5）。 */
export function loadSync(url, context, nextLoad) {
  if (!url.endsWith(".ts")) return nextLoad(url, context);
  return { format: "module", source: transpile(url), shortCircuit: true };
}
