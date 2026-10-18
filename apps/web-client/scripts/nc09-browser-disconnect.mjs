// NC09 断线中途（服务端崩溃）→ 可见错误 → 重启后恢复：由脚本自身启停 game-server。
//
// 命令：node apps/web-client/scripts/nc09-browser-disconnect.mjs <base> <chrome> <outdir> <server-bin>

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8123/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const serverBin = process.argv[5] ?? "./target/release/game-server";
const wsUrl = "ws://127.0.0.1:7700/ws";
const port = 9337;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function startServer() {
  return spawn(serverBin, [], {
    stdio: "ignore",
    env: {
      ...process.env,
      NONAME_ALLOW_UNSIGNED_EXTENSIONS: "1",
      NONAME_NATIVE_ROOMS: "1",
      NONAME_NATIVE_ROOM_PACKAGES: "shahai-swd",
      GAME_SERVER_ADDR: "127.0.0.1:7700",
      NONAME_NATIVE_ROOM_STORE: process.env.NONAME_NATIVE_ROOM_STORE ?? "data/native-rooms",
    },
  });
}
let server = startServer();
await sleep(6000);

const udd = mkdtempSync(join(tmpdir(), "nc09-disc-"));
const chrome = spawn(chromeBin, [
  "--headless=new", "--disable-gpu", "--no-sandbox", `--user-data-dir=${udd}`,
  `--remote-debugging-port=${port}`, "--window-size=1280,800", "about:blank",
], { stdio: "ignore" });

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

const fill = (label) => `(() => {
  const inputs = Array.from(document.querySelectorAll('input'));
  const setVal = (el, v) => { if (!el) return false;
    Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el), 'value').set.call(el, v);
    el.dispatchEvent(new Event('input', { bubbles: true })); return true; };
  setVal(inputs.find(i => i.autocomplete === 'nickname'), ${JSON.stringify(label)});
  setVal(inputs.find(i => i.type === 'url'), ${JSON.stringify(wsUrl)});
  const btn = Array.from(document.querySelectorAll('button')).find(b => (b.innerText||'').includes('创建并入座'));
  if (!btn) return { clicked: false };
  btn.click();
  return { clicked: true };
})()`;
console.log("STEP online " + JSON.stringify(await evalJs(fill("cdp-disc"))));
await sleep(5000);
const joined = await evalJs(`(document.body.innerText.match(/房间 #(\\d+)/) || [])[1] || '0'`);
console.log("STEP joined_room " + joined);

// 杀掉服务端 → 客户端应出现可见错误
server.kill("SIGKILL");
await sleep(6000);
const afterKill = await evalJs(`(() => {
  const t = document.body.innerText;
  return {
    len: t.length,
    hasErrorText: /失败|错误|断开|关闭|无法|重试|重连/i.test(t),
    errorLine: (t.match(/[^\\n]*(失败|错误|断开|关闭|无法|重试|重连)[^\\n]*/) || [])[0] ?? "",
  };
})()`);
console.log("STEP after_kill " + JSON.stringify(afterKill));
const shot = await send("Page.captureScreenshot", { format: "png" });
writeFileSync(join(outDir, "disconnect-screenshot.png"), Buffer.from(shot.data, "base64"));

// 重启服务端 → **不刷新页面**，等待客户端自动重连 → 应回到房间
server = startServer();
await sleep(12000);
const afterRestart = await evalJs(`(() => {
  const t = document.body.innerText;
  return {
    len: t.length,
    hasRoom: /房间 #\\d+/.test(t),
    roomLine: (t.match(/房间 #\\d+[^\\n]*/) || [])[0] ?? "",
    hasErrorText: /失败|错误|断开|关闭|无法/i.test(t),
    statusLine: (t.match(/[^\\n]*(重连|已重连|房间 #\\d+)[^\\n]*/) || [])[0] ?? "",
    seatLine: (t.match(/[^\\n]*(座位|P1|观察者 P\\d+)[^\\n]*/) || [])[0] ?? "",
  };
})()`);
console.log("STEP after_restart " + JSON.stringify(afterRestart));

const report = { room_id: joined, after_kill: afterKill, after_restart: afterRestart, console_errors: consoleErrors };
writeFileSync(join(outDir, "disconnect-report.json"), JSON.stringify(report, null, 2));
console.log("RESULT " + JSON.stringify(report));

ws.close();
chrome.kill("SIGTERM");
server.kill("SIGTERM");
await sleep(500);
rmSync(udd, { recursive: true, force: true });
