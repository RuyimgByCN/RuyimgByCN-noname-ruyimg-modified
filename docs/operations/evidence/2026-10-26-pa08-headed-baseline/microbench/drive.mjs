// PA07 判别驱动：有头 Chrome + CDP 真实点击 ×N，收集双通道延迟。
import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { readFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const server = createServer((req, res) => {
  res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
  res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
  if (req.url === "/worker.js") {
    res.setHeader("Content-Type", "text/javascript");
    res.end(`let seq, buf;
self.onmessage = (e) => {
  if (e.data && e.data.sab) { seq = new Int32Array(e.data.sab, 0, 2); buf = new Uint8Array(e.data.sab, 8); return; }
  const { n, t0, pad } = e.data;
  postMessage({ kind: 'pm', n, t0, pad });
  const s = JSON.stringify({ kind: 'sab', n, t0, pad });
  const bytes = new TextEncoder().encode(s);
  buf.fill(0);
  buf.set(bytes.subarray(0, 64*1024), 0);
  Atomics.store(seq, 0, n);
  Atomics.notify(seq, 0, 1);
};`);
    return;
  }
  res.setHeader("Content-Type", "text/html");
  res.end(readFileSync("/tmp/pa07-microbench/index.html"));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;
const udd = mkdtempSync(join(tmpdir(), "pa07-micro-"));
const chrome = spawn("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", [
  "--no-sandbox", "--no-first-run", `--user-data-dir=${udd}`,
  "--remote-debugging-port=0", "--window-size=1280,800", "about:blank",
], { stdio: "ignore" });
let port = null;
for (let k = 0; k < 100 && !port; k++) {
  try { port = Number(readFileSync(join(udd, "DevToolsActivePort"), "utf8").split("\n")[0]); } catch { await sleep(100); }
}
const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
const page = list.find((t) => t.type === "page");
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
let id = 0; const pending = new Map();
ws.onmessage = (ev) => { const m = JSON.parse(ev.data); if (m.id && pending.has(m.id)) { pending.get(m.id)(m.result); pending.delete(m.id); } };
const send = (method, params = {}) => new Promise((res) => { const i = ++id; pending.set(i, res); ws.send(JSON.stringify({ id: i, method, params })); });
const evalJs = async (expression) => (await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true })).result?.value;
await send("Page.enable");
await send("Page.navigate", { url: `${base}/` });
await sleep(2000);
const ready = await evalJs(`!!window.__micro && SharedArrayBuffer !== undefined && typeof Atomics.waitAsync === 'function'`);
if (!ready) { console.log("READY_FAIL: SAB/waitAsync 不可用或页面未就绪"); process.exit(1); }
const rect = await evalJs(`(() => { const r = document.getElementById('btn').getBoundingClientRect(); return { x: r.left + r.width/2, y: r.top + r.height/2 }; })()`);
for (let i = 0; i < 60; i++) {
  const at = { x: rect.x, y: rect.y, button: "left", clickCount: 1 };
  await send("Input.dispatchMouseEvent", { type: "mousePressed", ...at });
  await send("Input.dispatchMouseEvent", { type: "mouseReleased", ...at });
  await sleep(60);
}
await sleep(800);
const raw = await evalJs(`window.__micro.report()`);
console.log("RESULT " + raw);
ws.close(); chrome.kill("SIGTERM"); server.close();
await sleep(300); rmSync(udd, { recursive: true, force: true }); process.exit(0);
