// NC12 真实浏览器档位：**浏览器内完整对局推进**（点击「自动一步」直至终局或步数上限）。
//
// 口径：headless Chrome + 真实构建产物 + 真实 WASM worker；本脚本不 mock 引擎。
// 用法：node apps/web-client/scripts/nc09-identity-fullgame.mjs <base> <chrome> <outdir> [maxSteps]

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8123/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const maxSteps = Number(process.argv[5] ?? 400);
const port = 9339;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let id = 0;
const pending = new Map();
const consoleErrors = [];
const udd = mkdtempSync(join(tmpdir(), "nc12-full-"));
const chrome = spawn(chromeBin, [
  "--headless=new", "--disable-gpu", "--no-sandbox", `--user-data-dir=${udd}`,
  `--remote-debugging-port=${port}`, "--window-size=1280,800", "about:blank",
], { stdio: "ignore" });

let page = null;
for (let k = 0; k < 60; k++) {
  try {
    const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
    page = list.find((t) => t.type === "page");
    if (page?.webSocketDebuggerUrl) break;
  } catch {}
  await sleep(200);
}
if (!page) throw new Error("CDP 未就绪");
const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
ws.onmessage = (ev) => {
  const m = JSON.parse(ev.data);
  if (m.id && pending.has(m.id)) {
    const { res, rej } = pending.get(m.id);
    pending.delete(m.id);
    m.error ? rej(new Error(JSON.stringify(m.error))) : res(m.result);
    return;
  }
  if (m.method === "Runtime.exceptionThrown") consoleErrors.push(`exception: ${m.params?.exceptionDetails?.text ?? "?"}`);
  if (m.method === "Runtime.consoleAPICalled" && m.params?.type === "error") {
    consoleErrors.push(`console.error: ${(m.params.args ?? []).map((a) => a.value ?? a.description ?? "?").join(" ")}`);
  }
};
const send = (method, params = {}) => {
  const msgId = ++id;
  ws.send(JSON.stringify({ id: msgId, method, params }));
  return new Promise((res, rej) => pending.set(msgId, { res, rej }));
};
const evalJs = async (expression) => {
  const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.text);
  return r.result?.value;
};
await send("Runtime.enable");
await send("Page.enable");
const identityUrl = base.replace(/\/+$/, "") + "/battle?players=5&seed=20267201&mode=identity";
await send("Page.navigate", { url: identityUrl });
await sleep(3000);

// 身份局：已由导航参数进入 /battle?mode=identity，无需点击首页按钮
await sleep(2500);

const revOf = async () => await evalJs(`(document.body.innerText.match(/rev (\\d+)/) || [])[1] || '0'`);
const started = Date.now();
let clicks = 0;
let lastRev = Number(await revOf());
let stalled = 0;
for (let i = 0; i < maxSteps; i++) {
  const clicked = await evalJs(`(() => {
    const el = Array.from(document.querySelectorAll('button,a')).find(x => (x.innerText||'').trim() === '自动一步');
    if (!el || el.disabled) return false;
    el.click();
    return true;
  })()`);
  if (!clicked) { stalled++; if (stalled > 5) break; await sleep(300); continue; }
  stalled = 0;
  clicks++;
  await sleep(120);
  const rev = Number(await revOf());
  if (rev > lastRev) lastRev = rev;
}
const wallMs = Date.now() - started;
const finalText = await evalJs(`document.body.innerText.slice(0, 400)`);
const ended = /胜利|败北|平局|终局|结束|胜负/.test(finalText);

const report = {
  clicks,
  final_revision: lastRev,
  wall_ms: wallMs,
  ended_marker: ended,
  final_text: finalText,
  console_errors: consoleErrors,
};
writeFileSync(join(outDir, "identity-fullgame-report.json"), JSON.stringify(report, null, 2));
console.log("RESULT scenario=browser_identity_fullgame tier=web " + JSON.stringify({
  clicks, final_revision: lastRev, wall_ms: wallMs, ended_marker: ended, console_errors: consoleErrors,
}));
console.log("NOTE headless Chrome + 本机静态服务；点击间隔 120ms 为脚本节流，非引擎耗时。");
console.log("NOTE 身份局（players=5, mode=identity）：终局胜负文案来自 view.outcome 的映射。");
ws.close();
chrome.kill("SIGTERM");
await sleep(300);
rmSync(udd, { recursive: true, force: true });
