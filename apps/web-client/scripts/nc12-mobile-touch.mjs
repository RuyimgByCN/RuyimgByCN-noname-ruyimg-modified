// NC12 移动档位：**触摸手势端到端**（CDP `Input.dispatchTouchEvent`）——视口模拟下的真实触摸路径。
//
// 口径：headless Chrome + 移动视口/触摸模拟；**不是真机**。本脚本验证的是
// 「触摸事件能否驱动应用进入对局并推进一步」，不替代真机档位验收。
// 用法：node apps/web-client/scripts/nc12-mobile-touch.mjs <base> <chrome> <outdir>

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8123/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const port = 9341;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let id = 0;
const pending = new Map();
const consoleErrors = [];
const udd = mkdtempSync(join(tmpdir(), "nc12-touch-"));
const chrome = spawn(chromeBin, [
  "--headless=new", "--disable-gpu", "--no-sandbox", `--user-data-dir=${udd}`,
  `--remote-debugging-port=${port}`, "--window-size=390,844", "about:blank",
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
// 移动视口 + 触摸模拟
await send("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 3, mobile: true });
await send("Emulation.setTouchEmulationEnabled", { enabled: true, maxTouchPoints: 5 });
await send("Page.navigate", { url: base });
await sleep(3000);

// 找到「本地开局」按钮的中心坐标
const rect = await evalJs(`(() => {
  const el = Array.from(document.querySelectorAll('button,a')).find(el => (el.innerText||'').includes('本地开局'));
  if (!el) return null;
  const r = el.getBoundingClientRect();
  return { x: Math.round(r.left + r.width/2), y: Math.round(r.top + r.height/2) };
})()`);
if (!rect) throw new Error("未找到『本地开局』按钮");

// 触摸点击（touchStart + touchEnd）
const touch = async (type, x, y) =>
  send("Input.dispatchTouchEvent", {
    type,
    touchPoints: type === "touchEnd" ? [] : [{ x, y, radiusX: 8, radiusY: 8, force: 1, id: 1 }],
  });
await touch("touchStart", rect.x, rect.y);
await sleep(60);
await touch("touchEnd", rect.x, rect.y);
await sleep(2500);

const url = await evalJs(`location.pathname + location.search`);
const interactive = await evalJs(`(() => {
  const el = Array.from(document.querySelectorAll('button,a')).find(x => (x.innerText||'').trim() === '自动一步');
  return !!el && !el.disabled;
})()`);
const viewport = await evalJs(`JSON.stringify({ w: window.innerWidth, h: window.innerHeight, touch: 'ontouchstart' in window })`);
const report = { tapped_at: rect, url_after_tap: url, interactive, viewport, console_errors: consoleErrors };
writeFileSync(join(outDir, "mobile-touch-report.json"), JSON.stringify(report, null, 2));
console.log("RESULT scenario=mobile_touch tier=web " + JSON.stringify(report));
console.log("NOTE 视口/触摸为 CDP 模拟，非真机；仅验证触摸事件可驱动应用进入对局。");
ws.close();
chrome.kill("SIGTERM");
await sleep(300);
rmSync(udd, { recursive: true, force: true });
