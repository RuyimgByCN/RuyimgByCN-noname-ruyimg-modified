// Exercise the production native Worker in a real browser thread, using the
// existing protocol. Usage: node <script> <dist> <repo> <chrome>.
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
const runDir = mkdtempSync(join(tmpdir(), "nc03-input-worker-"));
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

  const start = await request({ type: "start", players: 4, seed: 20261028, observer: 0 });
  assert.equal(start.ok, true, start.error);
  const baseline = (await request({ type: "save" })).data.hex;
  const candidates = await request({ type: "generalCandidates" });
  assert.equal(candidates.ok, true, candidates.error);
  const name = candidates.data.candidates[0].name;
  const invalid = [
    { type: "start", players: 2 ** 32 + 4, seed: 1, observer: 0 },
    { type: "start", players: 4, seed: 1, observer: 2 ** 32 },
    { type: "chooseGeneral", seat: 2 ** 32, name },
    { type: "endPhase", actor: 2 ** 32 },
    { type: "playCard", actor: 0, card: 0, targets: [2 ** 32] },
    { type: "playCard", actor: 0, card: 2 ** 32, targets: [] },
    { type: "activateSkill", actor: 0, skill: 2 ** 16 },
  ];
  for (const msg of invalid) {
    const reply = await request(msg);
    assert.equal(reply.ok, false, `${msg.type} accepted`);
    assert.match(reply.error, /非法整数/);
    assert.equal((await request({ type: "save" })).data.hex, baseline, `${msg.type} changed state`);
  }
  pass("integer_coercion_rejected", `${invalid.length} requests; save unchanged`);
  let choice;
  for (let step = 0; step < 100; step++) {
    const advanced = await request({ type: "autoStep" });
    assert.equal(advanced.ok, true, advanced.error);
    if (advanced.data.choice) { choice = advanced.data.choice; break; }
  }
  assert.ok(choice, "no open choice after 100 steps");
  // Only the observer sees options; with hidden options seat 0 is not the owner.
  const other = choice.options.length ? 1 : 0;
  const beforeChoice = (await request({ type: "save" })).data.hex;
  const forged = await request({ type: "resolveChoice", actor: other, choiceId: choice.choice_id, selections: [], cancelled: true });
  assert.equal(forged.ok, true, forged.error);
  assert.equal(forged.data.result.outcome, "rejected");
  assert.equal(forged.data.result.reason, "WrongPlayer");
  assert.equal((await request({ type: "save" })).data.hex, beforeChoice, "forged choice changed state");
  assert.equal((await request({ type: "autoStep" })).ok, true, "real choice could not continue");
  pass("foreign_choice_rejected", "WrongPlayer; save unchanged; real choice continued");
  assert.deepEqual(await evaluate("window.__workerErrors"), []);
  assert.deepEqual(exceptions, []);
  pass("browser_and_worker_errors", "0");
  console.log(`RESULT ${checks} passed / 0 failed`);
} finally {
  if (ws?.readyState === WebSocket.OPEN) {
    try { await new Promise((done) => { ws.onclose = done; ws.close(); }); } catch { /* already closed */ }
  }
  browserWs?.close();
  chrome.kill("SIGTERM");
  await new Promise((done) => server.close(done));
}
