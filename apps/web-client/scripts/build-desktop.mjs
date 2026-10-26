#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const webClient = join(dirname(fileURLToPath(import.meta.url)), "..");
const pnpm = process.platform === "win32" ? "pnpm.cmd" : "pnpm";

const options = { cwd: webClient, stdio: "inherit", shell: process.platform === "win32" };
execFileSync(pnpm, ["build:native"], {
  ...options,
  // 桌面始终交付 web 模块，即使调用环境曾为 Node 冒烟选择 nodejs。
  env: { ...process.env, WASM_PACK_TARGET: "web" },
});
execFileSync(pnpm, ["build:wasm"], options);
execFileSync(pnpm, ["build"], options);
