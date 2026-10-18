// NC09 真实浏览器**联机闭环**：加载构建产物 → 填昵称 → 创建并入座 → 断言房间/座位出现。
//
// 前置：`game-server` 已在 127.0.0.1:7700 监听（建议 NONAME_NATIVE_ROOMS=1），
// 且 `dist/` 已构建并由静态服务提供（默认 http://127.0.0.1:8123/）。
// 命令：node apps/web-client/scripts/nc09-browser-online.mjs <base> <chrome> <outdir> [ws]

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8123/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const wsUrl = process.argv[5] ?? "ws://127.0.0.1:7700/ws";
const port = 9334;
const udd = mkdtempSync(join(tmpdir(), "nc09-online-"));
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
  throw new Error("CDP target 未就绪");
}

let id = 0;
const pending = new Map();
const consoleErrors = [];
function send(ws, method, params = {}) {
  const msgId = ++id;
  ws.send(JSON.stringify({ id: msgId, method, params }));
  return new Promise((res, rej) => pending.set(msgId, { res, rej }));
}
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
await send(ws, "Runtime.enable");
await send(ws, "Page.enable");
await send(ws, "Page.navigate", { url: base });
await sleep(3500);

async function evalJs(expression) {
  const r = await send(ws, "Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.text);
  return r.result?.value;
}

// 填昵称 + 服务端 URL（Vue v-model 需要 input 事件）
const filled = await evalJs(`(() => {
  const inputs = Array.from(document.querySelectorAll('input'));
  const nick = inputs.find(i => i.autocomplete === 'nickname');
  const url = inputs.find(i => i.type === 'url');
  const setVal = (el, v) => {
    if (!el) return false;
    const proto = Object.getPrototypeOf(el);
    Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, v);
    el.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  };
  const a = setVal(nick, 'cdp-p1');
  const b = setVal(url, ${JSON.stringify(wsUrl)});
  return { nick: a, url: b };
})()`);
console.log("STEP filled " + JSON.stringify(filled));
await sleep(500);

// 点击「创建并入座」
const clicked = await evalJs(`(() => {
  const el = Array.from(document.querySelectorAll('button')).find(b => (b.innerText||'').includes('创建并入座'));
  if (!el) return false;
  el.click();
  return true;
})()`);
console.log("STEP clicked_create " + clicked);

// 等待房间/座位/连接状态出现
let online = null;
for (let i = 0; i < 40; i++) {
  await sleep(500);
  online = await evalJs(`(() => {
    const t = document.body.innerText;
    return {
      len: t.length,
      hasRoom: /房间\\s*#?\\d+/.test(t) || /房号/.test(t),
      hasSeat: /座位|P1|入座|已连接|连接中/.test(t),
      hasError: /失败|错误|error/i.test(t),
      snippet: t.slice(0, 400),
    };
  })()`);
  if (online?.hasSeat) break;
}
console.log("STEP online_state " + JSON.stringify(online));

const shot = await send(ws, "Page.captureScreenshot", { format: "png" });
writeFileSync(join(outDir, "online-screenshot.png"), Buffer.from(shot.data, "base64"));

const report = {
  filled,
  clicked_create: clicked,
  online_text_len: online?.len ?? 0,
  online_has_seat: !!online?.hasSeat,
  online_has_error_text: !!online?.hasError,
  online_snippet: online?.snippet ?? "",
  console_errors: consoleErrors,
};
writeFileSync(join(outDir, "online-report.json"), JSON.stringify(report, null, 2));
console.log("RESULT " + JSON.stringify(report));

ws.close();
chrome.kill("SIGTERM");
await sleep(300);
rmSync(udd, { recursive: true, force: true });
