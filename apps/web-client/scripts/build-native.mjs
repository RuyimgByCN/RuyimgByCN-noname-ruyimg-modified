// 构建 engine-native-wasm 并布署到 web-client（T13 客户端原生后端）。
// 用法：node scripts/build-native.mjs
import { execSync } from "node:child_process";
import { cpSync, rmSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const crate = join(root, "crates", "engine-native-wasm");
const out = join(root, "apps", "web-client", "src", "worker", "native-pkg");
const tmp = join(root, "apps", "web-client", "src", "worker", ".native-pkg-tmp");

execSync(
  `wasm-pack build ${crate} --target nodejs --release --no-opt --out-dir ${tmp}`,
  { stdio: "inherit" },
);
rmSync(out, { recursive: true, force: true });
cpSync(tmp, out, { recursive: true });
rmSync(tmp, { recursive: true, force: true });
console.log(`native engine pkg deployed to ${out}`);
