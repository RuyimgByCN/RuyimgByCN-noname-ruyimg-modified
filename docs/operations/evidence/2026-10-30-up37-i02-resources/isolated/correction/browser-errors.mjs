import assert from "node:assert/strict";
import { createServer } from "node:http";
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";

const repo = path.resolve(process.argv[2]);
const out = path.resolve(process.argv[3]);
const build = path.resolve(process.argv[4]);
const profile = mkdtempSync(path.join(out, "chrome-"));
const badPath = "/original-content/assets/audio/die/star_dongyun.mp3";
const mime = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".json": "application/json", ".jpg": "image/jpeg", ".png": "image/png", ".mp3": "audio/mpeg" };
const requests = [];
const matrix = [];
let corruptAudio = true;
let failManifest = false;
const server = createServer((req, res) => {
  const url = new URL(req.url, "http://localhost");
  if (url.pathname === "/__control") {
    if (url.searchParams.has("audio")) corruptAudio = url.searchParams.get("audio") === "1";
    if (url.searchParams.has("manifest")) failManifest = url.searchParams.get("manifest") === "1";
    res.end("ok"); return;
  }
  const resource = url.pathname.startsWith("/original-content/");
  const root = resource ? path.join(repo, "content/original") : build;
  let file = path.resolve(root, "." + decodeURIComponent(resource ? url.pathname.slice("/original-content".length) : url.pathname));
  if (!file.startsWith(root + path.sep)) { res.writeHead(403).end(); return; }
  if (!resource && !existsSync(file)) file = path.join(root, "index.html");
  if (failManifest && url.pathname === "/original-content/asset-manifest.json") {
    requests.push({ path: url.pathname, injected: "manifest503" }); res.writeHead(503, { "Access-Control-Allow-Origin": "*" }).end("unavailable"); return;
  }
  if (corruptAudio && url.pathname === badPath) {
    requests.push({ path: url.pathname, injected: "non-audio HTML" }); res.writeHead(200, { "Content-Type": "text/html", "Access-Control-Allow-Origin": "*" }).end("<html>not audio</html>"); return;
  }
  try {
    const data = readFileSync(file);
    res.writeHead(200, { "Content-Type": mime[path.extname(file)] ?? "application/octet-stream", "Access-Control-Allow-Origin": "*", "Cross-Origin-Resource-Policy": "cross-origin" }).end(data);
    requests.push({ path: url.pathname, status: 200, bytes: data.length });
  } catch { res.writeHead(404).end(); }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const base = `http://127.0.0.1:${server.address().port}`;
const chrome = spawn("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", ["--headless=new", "--disable-gpu", "--no-sandbox", `--user-data-dir=${profile}`, "--remote-debugging-port=0", "--window-size=1280,1000", "about:blank"], { stdio: "ignore" });
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
let ws;
let id = 0;
const pending = new Map();
const exceptions = [];
try {
  for (let i = 0; i < 100 && !existsSync(path.join(profile, "DevToolsActivePort")); i++) await wait(100);
  const port = readFileSync(path.join(profile, "DevToolsActivePort"), "utf8").split("\n")[0];
  const pages = await fetch(`http://127.0.0.1:${port}/json/list`).then((response) => response.json());
  ws = new WebSocket(pages.find((page) => page.type === "page").webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
  const send = (method, params = {}) => new Promise((resolve, reject) => { const key = ++id; pending.set(key, { resolve, reject }); ws.send(JSON.stringify({ id: key, method, params })); });
  ws.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (message.id) { const item = pending.get(message.id); pending.delete(message.id); if (item) message.error ? item.reject(message.error) : item.resolve(message.result); }
    else if (message.method === "Fetch.requestPaused") void send("Fetch.continueRequest", { requestId: message.params.requestId, url: message.params.request.url.replace("http://127.0.0.1:7700", base) });
    else if (message.method === "Runtime.exceptionThrown") exceptions.push(message.params);
  };
  const evaluate = async (expression) => {
    const result = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
    assert.ok(!result.exceptionDetails, JSON.stringify(result.exceptionDetails)); return result.result.value;
  };
  const until = async (expression) => {
    for (let i = 0; i < 100; i++) { if (await evaluate(expression)) return; await wait(50); }
    throw new Error(`Timed out: ${expression}`);
  };
  const select = async (name) => {
    await evaluate(`(()=>{const input=document.querySelector('[aria-label="搜索武将"]');input.value=${JSON.stringify(name)};input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
    await until(`Array.from(document.querySelectorAll('.character-row')).some(row=>row.innerText.includes(${JSON.stringify(name)}))`);
    await evaluate(`Array.from(document.querySelectorAll('.character-row')).find(row=>row.innerText.includes(${JSON.stringify(name)})).click()`);
    await until(`document.querySelector('.detail header p')?.textContent.includes(${JSON.stringify(name)})`);
  };
  const snapshot = async (name) => {
    const result = await evaluate(`({case:${JSON.stringify(name)},alerts:[...document.querySelectorAll('.detail [role="alert"]')].map(node=>({text:node.textContent,voice:node.closest('li')?.querySelector('audio')?.getAttribute('src')??null})),audio:[...document.querySelectorAll('audio')].map(node=>({src:node.getAttribute('src'),error:node.error?.code??null,readyState:node.readyState})),originalA:window.originalFailedAudio?{error:window.originalFailedAudio.error?.code??null,connected:window.originalFailedAudio.isConnected,src:window.originalFailedAudio.getAttribute('src')}:null})`);
    matrix.push(result); writeFileSync(path.join(out, "browser-checkpoint.json"), JSON.stringify(matrix, null, 2) + "\n"); return result;
  };
  await send("Runtime.enable"); await send("Page.enable");
  await send("Fetch.enable", { patterns: [{ urlPattern: "http://127.0.0.1:7700/original-content/*" }] });
  await send("Page.navigate", { url: base + "/collection" });
  await until("document.querySelectorAll('.character-row').length>0");
  await select("star_dongyun");
  await evaluate("(()=>{window.originalFailedAudio=document.querySelector('audio');window.originalFailedAudio.load();})()");
  await until("document.querySelector('audio')?.error?.code===4");
  const a = await snapshot("A genuine media error from HTML response");
  assert.equal(a.alerts.length, 1);
  assert.equal(a.alerts[0].voice, a.audio[0].src, "A error belongs to A's own voice row");
  await evaluate("document.querySelectorAll('audio')[1].load()");
  await until("document.querySelectorAll('audio')[1]?.readyState>=2");
  const sameCharacter = await snapshot("another A voice succeeds without clearing failed voice");
  assert.equal(sameCharacter.alerts.length, 1); assert.equal(sameCharacter.alerts[0].voice, a.audio[0].src);
  await select("huan_zhouyu");
  await evaluate("document.querySelector('audio').load()");
  await until("document.querySelector('audio')?.readyState>=2");
  const b = await snapshot("B loadeddata succeeds without A pollution");
  assert.equal(b.audio[0].error, null); assert.deepEqual(b.alerts, []);
  assert.equal(b.originalA.error, 4); assert.equal(b.originalA.connected, false); assert.equal(b.originalA.src, a.audio[0].src);
  await select("tangtang");
  const t = await snapshot("T has no audio and no unrelated alert");
  assert.deepEqual(t.audio, []); assert.deepEqual(t.alerts, []);
  await select("star_dongyun");
  const remembered = await snapshot("returning to A retains only A's failed path");
  assert.equal(remembered.alerts.length, 1); assert.equal(remembered.alerts[0].voice, a.audio[0].src);
  await fetch(base + "/__control?audio=0");
  await evaluate("document.querySelector('audio').load()");
  await until("document.querySelector('audio')?.readyState>=2");
  const recovered = await snapshot("successful A loadeddata clears only its path");
  assert.equal(recovered.audio[0].error, null); assert.deepEqual(recovered.alerts, []);
  await fetch(base + "/__control?manifest=1");
  await send("Page.reload", { ignoreCache: true });
  await until("document.querySelector('.detail [role=alert]')?.textContent.includes('无法读取语音资源清单')");
  const manifest = await snapshot("manifest request really fails with 503");
  assert.equal(manifest.alerts.length, 1); assert.equal(manifest.alerts[0].voice, null);
  for (const name of ["huan_zhouyu", "tangtang"]) {
    await select(name); const result = await snapshot(`manifest failure persists on ${name}`);
    assert.deepEqual(result.alerts, manifest.alerts); assert.deepEqual(result.audio, []);
  }
  assert.deepEqual(exceptions, []);
  writeFileSync(path.join(out, "browser-result.json"), JSON.stringify({ passed: true, base, build, sourceRouting: "Actual built Vue component and source bytes; CDP routes original-content to private static server; only designated A response and manifest503 are injected", matrix, exceptions, requests }, null, 2) + "\n");
  console.log(JSON.stringify({ passed: true, cases: matrix.map((row) => row.case), assertions: "per-path failure; other voice success; B/T isolation; A remains local; loadeddata clears A; true manifest failure preserved" }));
} finally {
  ws?.close(); chrome.kill("SIGTERM");
  await new Promise((resolve) => chrome.once("exit", resolve));
  await new Promise((resolve) => server.close(resolve));
  rmSync(profile, { recursive: true, force: true });
}
