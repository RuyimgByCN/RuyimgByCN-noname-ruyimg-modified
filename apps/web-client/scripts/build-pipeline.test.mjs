import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const scripts = dirname(fileURLToPath(import.meta.url));

// 运行真实入口，仅替换外部工具；临时 checkout 不借用现有 WASM 或 dist。
function fixture(t, name) {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "native build fixture-")));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const web = join(root, "apps/web-client");
  mkdirSync(join(web, "scripts"), { recursive: true });
  copyFileSync(join(scripts, name), join(web, "scripts", name));
  const preload = join(root, "tool-stub.cjs");
  const log = join(root, "calls.jsonl");
  writeFileSync(preload, `
    const cp = require("node:child_process");
    const fs = require("node:fs");
    const path = require("node:path");
    require("node:os").homedir = () => path.join(process.env.BUILD_TEST_ROOT, "user home");
    if (process.env.BUILD_TEST_PLATFORM) {
      Object.defineProperty(process, "platform", { value: process.env.BUILD_TEST_PLATFORM });
    }
    cp.spawnSync = () => ({ status: 0 });
    cp.execFileSync = (command, args, options = {}) => {
      fs.appendFileSync(process.env.BUILD_TEST_LOG, JSON.stringify({
        command, args, cwd: options.cwd, shell: options.shell,
        path: options.env?.PATH, target: options.env?.WASM_PACK_TARGET,
      }) + "\\n");
      if (process.env.BUILD_TEST_FAIL === command || process.env.BUILD_TEST_FAIL === args[0]) {
        throw new Error("tool failed");
      }
      const outIndex = args.indexOf("--out-dir");
      if (outIndex !== -1) {
        const out = args[outIndex + 1];
        fs.mkdirSync(out, { recursive: true });
        fs.writeFileSync(path.join(out, "fresh.wasm"), "fresh source build");
      }
    };
    require("node:module").syncBuiltinESMExports();
  `);
  const run = (extra = {}) => {
    const env = { ...process.env, BUILD_TEST_ROOT: root, BUILD_TEST_LOG: log, ...extra };
    if (!("WASM_PACK_TARGET" in extra)) delete env.WASM_PACK_TARGET;
    const result = spawnSync(process.execPath, ["--require", preload, join(web, "scripts", name)], {
      cwd: tmpdir(), encoding: "utf8", env,
    });
    const calls = existsSync(log)
      ? readFileSync(log, "utf8").trim().split("\n").map((line) => JSON.parse(line))
      : [];
    return { ...result, calls };
  };
  return { root, web, run };
}

test("desktop builds both WASM engines before the frontend, forcing native web output", (t) => {
  const { web, run } = fixture(t, "build-desktop.mjs");
  const result = run({ WASM_PACK_TARGET: "nodejs" });
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(result.calls.map((call) => call.args), [["build:native"], ["build:wasm"], ["build"]]);
  assert.equal(result.calls[0].target, "web");
  assert.ok(result.calls.every((call) => resolve(call.cwd) === web));
});

test("Windows desktop runs pnpm.cmd through a shell", (t) => {
  const { run } = fixture(t, "build-desktop.mjs");
  const result = run({ BUILD_TEST_PLATFORM: "win32" });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.calls.length, 3);
  assert.ok(result.calls.every((call) => call.command === "pnpm.cmd" && call.shell === true));
});

test("a failed native build prevents publishing a frontend with a stale native package", (t) => {
  const { run } = fixture(t, "build-desktop.mjs");
  const result = run({ BUILD_TEST_FAIL: "build:native" });
  assert.notEqual(result.status, 0);
  assert.deepEqual(result.calls.map((call) => call.args), [["build:native"]]);
});

test("native build creates a missing package from source with portable PATH and argument boundaries", (t) => {
  const { root, web, run } = fixture(t, "build-native.mjs");
  const result = run();
  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(join(web, "src/worker/native-pkg/fresh.wasm"), "utf8"), "fresh source build");
  assert.equal(existsSync(join(web, "src/worker/.native-pkg-tmp")), false);
  assert.equal(result.calls.length, 1);
  const call = result.calls[0];
  assert.equal(call.command, "wasm-pack");
  assert.deepEqual(call.args.slice(0, 6), [
    "build", join(root, "crates/engine-native-wasm"), "--target", "web", "--release", "--no-opt",
  ]);
  assert.ok(call.path.startsWith(join(root, "user home/.cargo/bin") + delimiter));
  assert.equal(call.cwd, root);
});

test("native build failure preserves the previous usable package and clears temporary output", (t) => {
  const { web, run } = fixture(t, "build-native.mjs");
  const out = join(web, "src/worker/native-pkg");
  mkdirSync(out, { recursive: true });
  writeFileSync(join(out, "previous.wasm"), "previous build");
  const result = run({ BUILD_TEST_FAIL: "wasm-pack" });
  assert.notEqual(result.status, 0);
  assert.equal(readFileSync(join(out, "previous.wasm"), "utf8"), "previous build");
  assert.equal(existsSync(join(web, "src/worker/.native-pkg-tmp")), false);
});

test("native Node smoke target remains available without shell interpolation", (t) => {
  const { run } = fixture(t, "build-native.mjs");
  const result = run({ WASM_PACK_TARGET: "nodejs" });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.calls[0].args[3], "nodejs");
  assert.equal(result.calls[0].shell, undefined);
});

test("legacy build prepends rustup PATH even when the cargo executable has a Windows suffix", (t) => {
  const { root, web, run } = fixture(t, "build-wasm.mjs");
  const result = run();
  assert.equal(result.status, 0, result.stderr);
  assert.ok(result.calls[0].path.startsWith(join(root, "user home/.cargo/bin") + delimiter));
  assert.ok(existsSync(join(web, "src/wasm-pkg/fresh.wasm")));
  assert.ok(existsSync(join(web, "public/pkg/fresh.wasm")));
});

test("Tauri hooks resolve the actual sibling web client from the desktop crate", () => {
  const desktop = resolve(scripts, "../../desktop-client");
  const config = JSON.parse(readFileSync(join(desktop, "tauri.conf.json"), "utf8"));
  for (const [name, script] of [["beforeBuildCommand", "pnpm build:desktop"], ["beforeDevCommand", "pnpm dev"]]) {
    const hook = config.build[name];
    assert.equal(resolve(desktop, hook.cwd), resolve(scripts, ".."));
    assert.equal(hook.script, script);
  }
});
