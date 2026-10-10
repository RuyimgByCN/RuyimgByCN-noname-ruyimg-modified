#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const webClient = join(dirname(fileURLToPath(import.meta.url)), "..");
const desktopClient = join(webClient, "../desktop-client");
const executable = join(
  webClient,
  "node_modules/.bin",
  process.platform === "win32" ? "tauri.cmd" : "tauri"
);
const mode = process.argv[2];

if (mode !== "dev" && mode !== "build") {
  throw new Error("usage: run-tauri.mjs <dev|build>");
}

const args = mode === "build" ? ["build", "--bundles", "app"] : ["dev"];
execFileSync(executable, args, { cwd: desktopClient, stdio: "inherit" });
