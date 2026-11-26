#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { initSync, NativeEngine } from "../apps/web-client/src/worker/native-pkg/engine_native_wasm.js";

const players = Number(process.argv[2]);
const seed = Number(process.argv[3]);

if (!Number.isInteger(players) || !Number.isInteger(seed)) {
  console.error("usage: node scripts/wasm-hash.mjs <players> <seed>");
  process.exit(2);
}

const wasm = readFileSync(
  new URL("../apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm", import.meta.url),
);
initSync({ module: wasm });

const engine = new NativeEngine(players, BigInt(seed));
console.log(engine.state_hash_hex());
