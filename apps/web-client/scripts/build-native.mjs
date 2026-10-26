// 构建 engine-native-wasm（web 目标）并部署到 web-client。
// 用法：node scripts/build-native.mjs
// Node 冒烟另用：WASM_PACK_TARGET=nodejs node scripts/build-native.mjs
import { execFileSync } from "node:child_process";
import { cpSync, rmSync, existsSync, mkdirSync } from "node:fs";
import { delimiter, join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { homedir } from "node:os";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const crate = join(root, "crates", "engine-native-wasm");
const out = join(root, "apps", "web-client", "src", "worker", "native-pkg");
const tmp = join(root, "apps", "web-client", "src", "worker", ".native-pkg-tmp");
const target = process.env.WASM_PACK_TARGET || "web";
const rustupBin = join(homedir(), ".cargo/bin");
const env = {
  ...process.env,
  PATH: `${rustupBin}${delimiter}${process.env.PATH ?? ""}`,
  // 禁止 cargo clean；共用 target 目录
  CARGO_TARGET_DIR: join(root, "target"),
};

if (existsSync(tmp)) rmSync(tmp, { recursive: true, force: true });
mkdirSync(tmp, { recursive: true });

try {
  execFileSync(
    "wasm-pack",
    ["build", crate, "--target", target, "--release", "--no-opt", "--out-dir", tmp],
    { stdio: "inherit", cwd: root, env },
  );
  rmSync(out, { recursive: true, force: true });
  cpSync(tmp, out, { recursive: true });
} finally {
  rmSync(tmp, { recursive: true, force: true });
}
console.log(`native engine pkg (${target}) deployed to ${out}`);
