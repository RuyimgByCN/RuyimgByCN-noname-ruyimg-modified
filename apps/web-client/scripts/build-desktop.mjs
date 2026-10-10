#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const webClient = join(dirname(fileURLToPath(import.meta.url)), "..");
const pnpm = process.platform === "win32" ? "pnpm.cmd" : "pnpm";

execFileSync(pnpm, ["build:wasm"], { cwd: webClient, stdio: "inherit" });
execFileSync(pnpm, ["build"], { cwd: webClient, stdio: "inherit" });
