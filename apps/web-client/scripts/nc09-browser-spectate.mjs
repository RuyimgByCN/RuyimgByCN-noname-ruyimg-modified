// NC09 真实浏览器**旁观 + 多客户端信息隔离**：
// 页面 A 创建并入座 → 页面 B 旁观同一房间 → 断言 B 看到房间/名册但**看不到他人手牌内容**。
//
// 前置：game-server 在 127.0.0.1:7700（NONAME_NATIVE_ROOMS=1）、dist 由静态服务提供。
// 命令：node apps/web-client/scripts/nc09-browser-spectate.mjs <base> <chrome> <outdir> [ws]

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8123/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const wsUrl = process.argv[5] ?? "ws://127.0.0.1:7700/ws";
const port = 9335;
const udd = mkdtempSync(join(tmpdir(), "nc09-spec-"));
const chrome = spawn(chromeBin, [
  "--headless=new", "--disable-gpu", "--no-sandbox", `--user-data-dir=${udd}`,
  `--remote-debugging-port=${port}`, "--window-size=1280,800", "about:blank",
], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function pages() {
  for (let i = 0; i < 50; i++) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      const ps = list.filter((t) => t.type === "page" && t.webSocketDebuggerUrl);
      if (ps.length >= 1) return ps;
    } catch {}
    await sleep(200);
  }
  throw new Error("CDP 未就绪");
}

function connect(url, errors) {
  let id = 0;
  const pending = new Map();
  const ws = new WebSocket(url);
  const ready = new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
  ws.onmessage = (ev) => {
    const m = JSON.parse(ev.data);
    if (m.id && pending.has(m.id)) {
      const { res, rej } = pending.get(m.id);
      pending.delete(m.id);
      m.error ? rej(new Error(JSON.stringify(m.error))) : res(m.result);
      return;
    }
    if (m.method === "Runtime.exceptionThrown") errors.push(`exception: ${m.params?.exceptionDetails?.text ?? "?"}`);
    if (m.method === "Runtime.consoleAPICalled" && m.params?.type === "error") {
      errors.push(`console.error: ${(m.params.args ?? []).map((a) => a.value ?? a.description ?? "?").join(" ")}`);
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
  return { ws, ready, send, evalJs };
}

const errA = [];
const errB = [];
const listA = await pages();
const a = connect(listA[0].webSocketDebuggerUrl, errA);
await a.ready;
await a.send("Runtime.enable");
await a.send("Page.enable");
await a.send("Page.navigate", { url: base });
await sleep(3500);

const fillAndClick = (evalFn, label) => evalFn(`(() => {
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
})()`);

console.log("STEP A_fill_create " + JSON.stringify(await fillAndClick(a.evalJs, "cdp-host")));
await sleep(4000);
const roomId = await a.evalJs(`(document.body.innerText.match(/房间 #(\\d+)/) || [])[1] || '0'`);
console.log("STEP A_room " + roomId);

// 页面 B：旁观同一房间
const t = await a.send("Target.createTarget", { url: "about:blank" });
await sleep(1200);
const listB = await pages();
const pageB = listB.find((p) => p.id === t.targetId) ?? listB[listB.length - 1];
const b = connect(pageB.webSocketDebuggerUrl, errB);
await b.ready;
await b.send("Runtime.enable");
await b.send("Page.enable");
await b.send("Page.navigate", { url: base });
await sleep(3500);
const specFill = await b.evalJs(`(() => {
  const inputs = Array.from(document.querySelectorAll('input'));
  const setVal = (el, v) => { if (!el) return false;
    Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el), 'value').set.call(el, v);
    el.dispatchEvent(new Event('input', { bubbles: true })); return true; };
  setVal(inputs.find(i => i.autocomplete === 'nickname'), 'cdp-spec');
  setVal(inputs.find(i => i.type === 'url'), ${JSON.stringify(wsUrl)});
  const num = inputs.find(i => i.type === 'number');
  setVal(num, ${JSON.stringify(String(roomId))});
  const btn = Array.from(document.querySelectorAll('button')).find(b => (b.innerText||'').trim() === '旁观');
  if (!btn) return { clicked: false };
  btn.click();
  return { clicked: true };
})()`);
console.log("STEP B_spectate_click " + JSON.stringify(specFill));
await sleep(4500);
const specState = await b.evalJs(`(() => {
  const t = document.body.innerText;
  return {
    len: t.length,
    hasRoom: /房间 #\\d+/.test(t),
    hasRoster: /在线|玩家概况/.test(t),
    handLine: (t.match(/手牌（[^）]*）/) || [])[0] ?? "",
    hasOwnHandCards: /手牌（[1-9]/.test(t),
    snippet: t.slice(0, 300),
  };
})()`);
console.log("STEP B_state " + JSON.stringify(specState));

// 多客户端命令一致性：页面 A 结束阶段 → 旁观页面 B 的 rev 也应前进，且仍看不到手牌内容
const revA0 = await a.evalJs(`(document.body.innerText.match(/rev (\\d+)/) || [])[1] || '0'`);
const revB0 = await b.evalJs(`(document.body.innerText.match(/rev (\\d+)/) || [])[1] || '0'`);
const cmdA = await a.evalJs(`(() => {
  const el = Array.from(document.querySelectorAll('button')).find(x => (x.innerText||'').trim() === '结束阶段');
  if (!el) return { found: false };
  el.click();
  return { found: true };
})()`);
await sleep(3000);
const revA1 = await a.evalJs(`(document.body.innerText.match(/rev (\\d+)/) || [])[1] || '0'`);
const revB1 = await b.evalJs(`(document.body.innerText.match(/rev (\\d+)/) || [])[1] || '0'`);
const specStillPublic = await b.evalJs(`(() => {
  const t = document.body.innerText;
  return { handLine: (t.match(/手牌（[^）]*）/) || [])[0] ?? "", hasOwnHandCards: /手牌（[1-9]/.test(t) };
})()`);
console.log("STEP multi_client_command " + JSON.stringify({ revA0, revA1, revB0, revB1, cmdA, specStillPublic }));

const shot = await b.send("Page.captureScreenshot", { format: "png" });
writeFileSync(join(outDir, "spectator-screenshot.png"), Buffer.from(shot.data, "base64"));
const shotA = await a.send("Page.captureScreenshot", { format: "png" });
writeFileSync(join(outDir, "host-screenshot.png"), Buffer.from(shotA.data, "base64"));

const report = {
  room_id: roomId,
  spectator: specState,
  console_errors_host: errA,
  console_errors_spectator: errB,
  multi_client_command: { revA0, revA1, revB0, revB1, cmdA, specStillPublic },
};
writeFileSync(join(outDir, "spectate-report.json"), JSON.stringify(report, null, 2));
console.log("RESULT " + JSON.stringify(report));

a.ws.close();
b.ws.close();
chrome.kill("SIGTERM");
await sleep(300);
rmSync(udd, { recursive: true, force: true });
