// NC09 真实浏览器端到端（CDP 驱动）：加载构建产物 → 点击「本地开局」→ 断言战斗界面出现。
//
// 口径：真实 Chrome（headless）+ 真实构建产物 + 真实 WASM Worker；本脚本**不 mock**
// 任何引擎或 worker。命令：
//   node apps/web-client/scripts/nc09-browser-drive.mjs <base-url> <chrome-bin> <outdir>
//
// 依赖：仅 Node 内置（fetch + WebSocket + CDP HTTP/WS 接口）。

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8123/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const port = 9333;

const udd = mkdtempSync(join(tmpdir(), "nc09-cdp-"));
const chrome = spawn(chromeBin, [
  "--headless=new",
  "--disable-gpu",
  "--no-sandbox",
  `--user-data-dir=${udd}`,
  `--remote-debugging-port=${port}`,
  "--window-size=1280,800",
  "about:blank",
], { stdio: "ignore" });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function cdpTarget() {
  for (let i = 0; i < 50; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/list`);
      const list = await res.json();
      const page = list.find((t) => t.type === "page");
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
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
  return new Promise((resolve, reject) => pending.set(msgId, { resolve, reject }));
}

const wsUrl = await cdpTarget();
const ws = new WebSocket(wsUrl);
await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
ws.onmessage = (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.id && pending.has(msg.id)) {
    const { resolve, reject } = pending.get(msg.id);
    pending.delete(msg.id);
    msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
    return;
  }
  if (msg.method === "Runtime.exceptionThrown") {
    consoleErrors.push(`exception: ${msg.params?.exceptionDetails?.text ?? "?"}`);
  }
  if (msg.method === "Runtime.consoleAPICalled" && msg.params?.type === "error") {
    consoleErrors.push(`console.error: ${(msg.params.args ?? []).map((a) => a.value ?? a.description ?? "?").join(" ")}`);
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

// ① 大厅就绪（文案含「本地开局」）
const lobbyText = await evalJs("document.body.innerText.slice(0, 400)");
const hasLocalStart = await evalJs(
  `Array.from(document.querySelectorAll('button,a')).some(el => (el.innerText||'').includes('本地开局'))`,
);
if (!hasLocalStart) throw new Error(`未找到「本地开局」入口；大厅文本：${lobbyText}`);
console.log("STEP lobby_ready ok");

// ② 点击「本地开局」
const clicked = await evalJs(`(() => {
  const el = Array.from(document.querySelectorAll('button,a')).find(el => (el.innerText||'').includes('本地开局'));
  if (!el) return false;
  el.click();
  return true;
})()`);
if (!clicked) throw new Error("点击本地开局失败");
console.log("STEP clicked_local_start ok");

// ③ 等待战斗界面（出现座位/阶段/手牌等标记）
let battle = null;
for (let i = 0; i < 40; i++) {
  await sleep(500);
  battle = await evalJs(`(() => {
    const t = document.body.innerText;
    return {
      len: t.length,
      hasPhase: /判定|摸牌|出牌|弃牌|结束/.test(t),
      hasSeat: /座位|玩家|主公|体力/.test(t),
      snippet: t.slice(0, 200),
    };
  })()`);
  if (battle?.hasPhase && battle?.len > 200) break;
}
console.log("STEP battle_view " + JSON.stringify(battle));

const shot = await send(ws, "Page.captureScreenshot", { format: "png" });
writeFileSync(join(outDir, "battle-screenshot.png"), Buffer.from(shot.data, "base64"));

// ④ 报告
const report = {
  lobby_has_local_start: hasLocalStart,
  battle_text_len: battle?.len ?? 0,
  battle_has_phase: !!battle?.hasPhase,
  battle_has_seat: !!battle?.hasSeat,
  console_errors: consoleErrors,
};
writeFileSync(join(outDir, "drive-report.json"), JSON.stringify(report, null, 2));
console.log("RESULT " + JSON.stringify(report));

ws.close();
chrome.kill("SIGTERM");
await sleep(300);
rmSync(udd, { recursive: true, force: true });
if (!report.battle_has_phase) process.exit(2);
if (report.console_errors.length > 0) process.exit(3);
