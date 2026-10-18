// NC09 网络异常可见性：服务端**不可达**时，界面必须给出可见错误（不得静默卡住）。
//
// 前置：dist 由静态服务提供；**不启动** game-server（或给一个无人监听的端口）。
// 命令：node apps/web-client/scripts/nc09-browser-online-error.mjs <base> <chrome> <outdir> [bad-ws]

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8123/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const badWs = process.argv[5] ?? "ws://127.0.0.1:7799/ws";
const port = 9336;
const udd = mkdtempSync(join(tmpdir(), "nc09-err-"));
const chrome = spawn(chromeBin, [
  "--headless=new", "--disable-gpu", "--no-sandbox", `--user-data-dir=${udd}`,
  `--remote-debugging-port=${port}`, "--window-size=1280,800", "about:blank",
], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function target() {
  for (let i = 0; i < 50; i++) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      const p = list.find((t) => t.type === "page");
      if (p?.webSocketDebuggerUrl) return p.webSocketDebuggerUrl;
    } catch {}
    await sleep(200);
  }
  throw new Error("CDP 未就绪");
}
let id = 0;
const pending = new Map();
const consoleErrors = [];
const ws = new WebSocket(await target());
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
await send("Page.navigate", { url: base });
await sleep(3500);

const filled = await evalJs(`(() => {
  const inputs = Array.from(document.querySelectorAll('input'));
  const setVal = (el, v) => { if (!el) return false;
    Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el), 'value').set.call(el, v);
    el.dispatchEvent(new Event('input', { bubbles: true })); return true; };
  setVal(inputs.find(i => i.autocomplete === 'nickname'), 'cdp-err');
  setVal(inputs.find(i => i.type === 'url'), ${JSON.stringify(badWs)});
  const btn = Array.from(document.querySelectorAll('button')).find(b => (b.innerText||'').includes('创建并入座'));
  if (!btn) return { clicked: false };
  btn.click();
  return { clicked: true };
})()`);
console.log("STEP fill_create_bad_ws " + JSON.stringify(filled));
await sleep(6000);

const state = await evalJs(`(() => {
  const t = document.body.innerText;
  return {
    len: t.length,
    hasErrorText: /失败|错误|无法|连接不上|closed|error|重试/i.test(t),
    errorLine: (t.match(/[^\\n]*(失败|错误|无法|重试)[^\\n]*/) || [])[0] ?? "",
    snippet: t.slice(0, 300),
  };
})()`);
console.log("STEP error_state " + JSON.stringify(state));
const shot = await send("Page.captureScreenshot", { format: "png" });
writeFileSync(join(outDir, "online-error-screenshot.png"), Buffer.from(shot.data, "base64"));
const report = { bad_ws: badWs, state, console_errors: consoleErrors };
writeFileSync(join(outDir, "online-error-report.json"), JSON.stringify(report, null, 2));
console.log("RESULT " + JSON.stringify(report));
ws.close();
chrome.kill("SIGTERM");
await sleep(300);
rmSync(udd, { recursive: true, force: true });
