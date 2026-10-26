#!/usr/bin/env node
/**
 * Build engine-wasm for the web client.
 * Prefers wasm-pack; falls back to cargo + wasm-bindgen-cli if available.
 */
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, cpSync, rmSync } from "node:fs";
import { homedir } from "node:os";
import { delimiter, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "../../..");
const crate = join(root, "crates/engine-wasm");
const outDir = join(__dirname, "../src/wasm-pkg");
const publicDir = join(__dirname, "../public/pkg");
const rustupBin = join(homedir(), ".cargo/bin");
const toolEnv = { ...process.env, PATH: `${rustupBin}${delimiter}${process.env.PATH ?? ""}` };

function has(cmd) {
  const r = spawnSync(cmd, ["--version"], { encoding: "utf8", env: toolEnv });
  return r.status === 0;
}

mkdirSync(outDir, { recursive: true });

if (has("wasm-pack")) {
  console.log("Using wasm-pack…");
  execFileSync(
    "wasm-pack",
    ["build", crate, "--target", "web", "--out-dir", outDir, "--release", "--", "--no-default-features"],
    {
      stdio: "inherit",
      cwd: root,
      env: { ...toolEnv, CARGO_TARGET_DIR: join(root, "target") },
    }
  );
} else {
  console.log("wasm-pack not found; using cargo + wasm-bindgen…");
  const targetDir = join(root, "target/wasm32-unknown-unknown/release");
  execFileSync(
    "cargo",
    ["build", "-p", "engine-wasm", "--target", "wasm32-unknown-unknown", "--release"],
    { stdio: "inherit", cwd: root, env: toolEnv }
  );
  if (!has("wasm-bindgen")) {
    console.log("Installing wasm-bindgen-cli…");
    execFileSync("cargo", ["install", "wasm-bindgen-cli", "--version", "0.2.100", "--locked"], {
      stdio: "inherit",
      env: toolEnv,
    });
  }
  const wasm = join(targetDir, "engine_wasm.wasm");
  if (!existsSync(wasm)) {
    console.error("Missing", wasm);
    process.exit(1);
  }
  // clean previous
  for (const f of ["engine_wasm.js", "engine_wasm_bg.wasm", "engine_wasm.d.ts", "package.json"]) {
    const p = join(outDir, f);
    if (existsSync(p)) rmSync(p);
  }
  execFileSync(
    "wasm-bindgen",
    [wasm, "--target", "web", "--out-dir", outDir, "--out-name", "engine_wasm"],
    { stdio: "inherit", env: toolEnv }
  );
}

// Mirror into public/ for static serving / fallback
mkdirSync(publicDir, { recursive: true });
try {
  cpSync(outDir, publicDir, { recursive: true });
} catch (e) {
  console.warn("Could not mirror to public/pkg:", e);
}
console.log("WASM package written to", outDir);
