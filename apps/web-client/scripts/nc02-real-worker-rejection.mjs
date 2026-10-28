// Exercise the production native Worker in a real browser thread, using the
// existing VM fixtures and protocol. Usage: node <script> <dist> <repo> <chrome>.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { mkdtempSync, openSync, readFileSync, readdirSync, statSync } from "node:fs";
import { extname, join, resolve } from "node:path";
import { tmpdir } from "node:os";

const [distArg, repoArg, chromeBin] = process.argv.slice(2);
assert.ok(distArg && repoArg && chromeBin, "expected dist, repo and Chrome paths");
const dist = resolve(distArg);
const repo = resolve(repoArg);
const assets = join(dist, "assets");
const workers = readdirSync(assets).filter((name) => /^engine\.native\.worker-.*\.js$/.test(name));
assert.equal(workers.length, 1, `expected one production native worker, found ${workers}`);
const workerName = workers[0];
const referencedByApp = readdirSync(assets).filter((name) => name.endsWith(".js") && name !== workerName)
  .some((name) => readFileSync(join(assets, name), "utf8").includes(workerName));
assert.ok(referencedByApp, "production app does not reference this worker chunk");
const fixture = join(repo, "apps/web-client/scripts/fixtures/vm-trigger");
const vmPackage = JSON.parse(readFileSync(join(fixture, "package.npack.json"), "utf8"));
vmPackage.characters = JSON.parse(readFileSync(join(fixture, "character.json"), "utf8"));
const oldSaves = JSON.parse(readFileSync(join(repo, "apps/web-client/scripts/fixtures/vm-control-flow/old-native-saves.json"), "utf8"));
const mime = { ".html": "text/html", ".js": "text/javascript", ".wasm": "application/wasm", ".css": "text/css" };
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url ?? "/", "http://localhost").pathname);
  if (path === "/") return res.writeHead(200, { "Content-Type": "text/html" }).end("<!doctype html><title>Worker probe</title>");
  const file = resolve(dist, "." + path);
  if (!file.startsWith(dist + "/")) return res.writeHead(403).end();
  try {
    if (!statSync(file).isFile()) return res.writeHead(404).end();
    return res.writeHead(200, { "Content-Type": mime[extname(file)] ?? "application/octet-stream" }).end(readFileSync(file));
  } catch {
    return res.writeHead(404).end();
  }
});
await new Promise((done) => server.listen(0, "127.0.0.1", done));
const origin = `http://127.0.0.1:${server.address().port}`;
const runDir = mkdtempSync(join(tmpdir(), "nc02-real-worker-"));
const chromeLog = openSync(join(runDir, "chrome.log"), "w");
const chrome = spawn(chromeBin, [
  "--headless=new", "--disable-gpu", "--no-sandbox", `--user-data-dir=${runDir}/profile`,
  "--remote-debugging-port=0", "about:blank",
], { stdio: ["ignore", chromeLog, chromeLog] });
const pause = (ms) => new Promise((done) => setTimeout(done, ms));
const wait = async (description, fn) => {
  for (let i = 0; i < 200; i++) {
    const result = await fn();
    if (result) return result;
    if (chrome.exitCode !== null) throw new Error(`Chrome exited ${chrome.exitCode}`);
    await pause(100);
  }
  throw new Error(`timed out: ${description}`);
};
let ws;
let browserWs;
let checks = 0;
const pass = (name, detail = "") => {
  checks++;
  console.log(`PASS ${name}${detail ? ` | ${detail}` : ""}`);
};
try {
  const port = await wait("Chrome debug port", () => {
    try { return readFileSync(join(runDir, "profile/DevToolsActivePort"), "utf8").split("\n")[0]; }
    catch { return null; }
  });
  const page = await wait("Chrome page", async () => {
    try { return (await fetch(`http://127.0.0.1:${port}/json/list`).then((r) => r.json())).find((x) => x.type === "page"); }
    catch { return null; }
  });
  const browser = await fetch(`http://127.0.0.1:${port}/json/version`).then((r) => r.json());
  browserWs = new WebSocket(browser.webSocketDebuggerUrl);
  await new Promise((done, reject) => { browserWs.onopen = done; browserWs.onerror = reject; });
  let browserSeq = 0;
  const browserPending = new Map();
  browserWs.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (!message.id || !browserPending.has(message.id)) return;
    const { done, reject } = browserPending.get(message.id);
    browserPending.delete(message.id);
    message.error ? reject(new Error(JSON.stringify(message.error))) : done(message.result);
  };
  const sendBrowser = (method, params = {}) => new Promise((done, reject) => {
    const id = ++browserSeq;
    browserPending.set(id, { done, reject });
    browserWs.send(JSON.stringify({ id, method, params }));
  });
  await sendBrowser("Target.setDiscoverTargets", { discover: true });
  ws = new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((done, reject) => { ws.onopen = done; ws.onerror = reject; });
  let seq = 0;
  const pending = new Map();
  const exceptions = [];
  const send = (method, params = {}) => new Promise((done, reject) => {
    const id = ++seq;
    pending.set(id, { done, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
  ws.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (message.id && pending.has(message.id)) {
      const { done, reject } = pending.get(message.id);
      pending.delete(message.id);
      message.error ? reject(new Error(JSON.stringify(message.error))) : done(message.result);
    } else if (message.method === "Runtime.exceptionThrown") {
      exceptions.push(message.params.exceptionDetails.text);
    } else if (message.method === "Runtime.consoleAPICalled" && message.params.type === "error") {
      exceptions.push(message.params.args.map((arg) => arg.value ?? arg.description).join(" "));
    }
  };
  const evaluate = async (expression) => {
    const reply = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
    if (reply.exceptionDetails) throw new Error(JSON.stringify(reply.exceptionDetails));
    return reply.result.value;
  };
  await send("Runtime.enable");
  await send("Page.enable");
  await send("Page.navigate", { url: origin + "/" });
  await wait("probe page", () => evaluate(`location.origin === ${JSON.stringify(origin)} && document.readyState === 'complete'`));
  await evaluate(`(() => {
    window.__workerErrors = [];
    window.__worker = new Worker(${JSON.stringify(`/assets/${workerName}`)}, { type: 'module' });
    window.__worker.addEventListener('error', event => window.__workerErrors.push(event.message));
    window.__nextRequest = 1;
    window.__send = message => new Promise((done, reject) => {
      const id = window.__nextRequest++;
      const timeout = setTimeout(() => { window.__worker.removeEventListener('message', listener); reject(new Error('Worker timeout: ' + message.type)); }, 30000);
      const listener = event => {
        if (event.data.id !== id) return;
        clearTimeout(timeout);
        window.__worker.removeEventListener('message', listener);
        done(event.data);
      };
      window.__worker.addEventListener('message', listener);
      window.__worker.postMessage({ ...message, id });
    });
  })()`);
  const request = (message) => evaluate(`window.__send(${JSON.stringify(message)})`);
  assert.equal((await request({ type: "init" })).ok, true);
  const workerTarget = await wait("dedicated worker target", async () => {
    const result = await sendBrowser("Target.getTargets");
    return result.targetInfos.find((target) => target.type === "worker" && target.parentId === page.id);
  });
  assert.ok(workerTarget.targetId);
  pass("production_worker_thread", `target=${workerTarget.type} chunk=${workerName}`);

  const packs = JSON.stringify([vmPackage]);
  const start = await request({ type: "start", players: 4, seed: 42, observer: 0, packsJson: packs });
  assert.equal(start.ok, true, start.error);
  const picked = await request({ type: "chooseGeneral", seat: 0, name: "vm_fixture" });
  assert.equal(picked.ok, true, picked.error);
  assert.ok(picked.data.envelope.view.seats[0].skills.some((skill) => skill.name === vmPackage.trigger_index[0].module));
  const saved = await request({ type: "save" });
  assert.equal(saved.ok, true, saved.error);
  const baseline = saved.data.hex;
  pass("valid_vm_start_choose_and_save");

  const unchanged = async (label) => {
    const after = await request({ type: "save" });
    assert.equal(after.ok, true, after.error);
    assert.ok(after.data.hex === baseline, `${label}: active match changed`);
  };
  const rejectAll = async (label, badPacks, badHex, pattern) => {
    for (const mode of [undefined, "identity"]) {
      const reply = await request({ type: "start", mode, players: 4, seed: 42, observer: 0, packsJson: badPacks });
      assert.equal(reply.ok, false, `${label}: bad ${mode ?? "standard"} start accepted`);
      assert.match(reply.error, pattern);
      await unchanged(label);
    }
    const restored = await request({ type: "restore", hex: badHex, packsJson: badPacks });
    assert.equal(restored.ok, false, `${label}: bad restore accepted`);
    assert.match(restored.error, pattern);
    await unchanged(label);
    pass(label, "standard/identity/restore rejected; active save unchanged");
  };
  const badTrigger = structuredClone(vmPackage);
  badTrigger.trigger_index[0].event = "phase_draw";
  await rejectAll("tampered_package_hash", JSON.stringify([badTrigger]), baseline, /logic_hash mismatch/);
  const badModule = structuredClone(vmPackage);
  badModule.modules[0].local_count += 1;
  await rejectAll("tampered_module_hash", JSON.stringify([badModule]), baseline, /module .*logic_hash mismatch/);
  for (const old of oldSaves) {
    await rejectAll(`invalid_${old.case}_old_save`, old.packs, old.hex, /instruction boundary/);
  }

  const restored = await request({ type: "restore", hex: baseline, packsJson: packs });
  assert.equal(restored.ok, true, restored.error);
  assert.equal(restored.data.hash_hex, picked.data.envelope.hash_hex);
  // Successful restore rotates the authentication generation stored in the archive.
  assert.notEqual((await request({ type: "save" })).data.hex, baseline);
  let env = restored.data;
  let triggered = false;
  for (let step = 0; step < 80 && !triggered; step++) {
    const before = env;
    const atDiscard = before.view.current_player === 0 && before.view.phase === "Discard";
    const checkpoint = atDiscard ? await request({ type: "save" }) : null;
    const command = before.choice ? { type: "autoStep" } : { type: "endPhase", actor: before.view.current_player };
    const advanced = await request(command);
    assert.equal(advanced.ok, true, advanced.error);
    assert.notEqual(advanced.data?.result?.outcome, "rejected");
    env = advanced.data;
    if (atDiscard && env.view.current_player === 0 && env.view.phase === "End") {
      assert.equal(env.view.seats[0].hand_count, before.view.seats[0].hand_count + 1);
      const reloaded = await request({ type: "restore", hex: checkpoint.data.hex, packsJson: packs });
      assert.equal(reloaded.ok, true, reloaded.error);
      const replay = await request(command);
      assert.equal(replay.ok, true, replay.error);
      assert.equal(replay.data.hash_hex, env.hash_hex);
      triggered = true;
    }
  }
  assert.equal(triggered, true, "valid VM effect did not trigger within 80 steps");
  pass("valid_vm_effect_and_restore_replay");
  assert.deepEqual(await evaluate("window.__workerErrors"), []);
  assert.deepEqual(exceptions, []);
  pass("browser_and_worker_errors", "0");
  console.log(`RESULT ${checks} passed / 0 failed; origin=${origin}; profile=${runDir}`);
} finally {
  if (ws?.readyState === WebSocket.OPEN) {
    try { await new Promise((done) => { ws.onclose = done; ws.close(); }); } catch { /* already closed */ }
  }
  browserWs?.close();
  chrome.kill("SIGTERM");
  await new Promise((done) => server.close(done));
}
