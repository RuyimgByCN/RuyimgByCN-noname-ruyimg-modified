// NC12 真实浏览器档位：**「创建到可操作」**（模块下载/编译 + 首屏 + 本地开局可交互）。
//
// 口径（方案 §NC12）：真实浏览器 + 真实构建产物；本脚本用 CDP 驱动 headless Chrome，
// 采集 PerformanceNavigationTiming / 资源条目 / 「点击本地开局 → 战斗界面出现」的墙钟。
//
// 用法：node apps/web-client/scripts/nc12-browser-coldstart.mjs <base> <chrome> <outdir> [iters]

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8123/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const iters = Number(process.argv[5] ?? 3);
const port = 9338;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

let id = 0;
const pending = new Map();
const consoleErrors = [];

async function connect(wsUrl) {
  const ws = new WebSocket(wsUrl);
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
  return { ws, send, evalJs };
}

const results = [];
for (let i = 0; i < iters; i++) {
  const udd = mkdtempSync(join(tmpdir(), "nc12-boot-"));
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
  const c = await connect(page.webSocketDebuggerUrl);
  await c.send("Runtime.enable");
  await c.send("Page.enable");
  // NC12：worker 初始化分段耗时由客户端诊断钩子写入 window.__nativeInitTimings（见 client.ts）
  await c.send("Page.navigate", { url: base });
  // 等待大厅就绪（出现「本地开局」）
  let ready = 0;
  for (let k = 0; k < 60; k++) {
    await sleep(250);
    const ok = await c.evalJs(`Array.from(document.querySelectorAll('button,a')).some(el => (el.innerText||'').includes('本地开局'))`);
    if (ok) { ready = 1; break; }
  }
  // 可选：先点「跳过动画」（若存在）→ 用于测**纯计算**冷启动（剔除 presentationDelay 节流）
  if (process.env.NC12_SKIP_ANIMS === "1") {
    await c.evalJs(`(() => {
      const el = Array.from(document.querySelectorAll('button,a,label')).find(x => (x.innerText||'').includes('跳过动画'));
      if (el) el.click();
      return !!el;
    })()`);
    await sleep(300);
  }
  // 「创建到可操作」：点击本地开局 → 战斗界面出现（含阶段/座位标记）
  const t0 = Date.now();
  await c.evalJs(`(() => {
    const el = Array.from(document.querySelectorAll('button,a')).find(el => (el.innerText||'').includes('本地开局'));
    if (el) el.click();
    return !!el;
  })()`);
  let interactive = 0;
  for (let k = 0; k < 80; k++) {
    await sleep(250);
    const ok = await c.evalJs(`/判定|摸牌|出牌|弃牌|结束/.test(document.body.innerText) && document.body.innerText.length > 200`);
    if (ok) { interactive = 1; break; }
  }
  const toInteractiveMs = Date.now() - t0;
  const workerTimings = await c.evalJs(`window.__nativeInitTimings || []`);
  const startMarks = await c.evalJs(`window.__nativeStartMarks || {}`);
  const perf = await c.evalJs(`(() => {
    const nav = performance.getEntriesByType('navigation')[0] || {};
    const res = performance.getEntriesByType('resource').map(r => ({ name: r.name.split('/').pop(), dur: r.duration, size: r.transferSize }));
    const wasm = res.filter(r => r.name.includes('.wasm'));
    return {
      domContentLoaded: nav.domContentLoadedEventEnd ?? null,
      loadEvent: nav.loadEventEnd ?? null,
      wasm: wasm,
      workerChunks: res.filter(r => r.name.includes('worker')).map(r => r.name),
    };
  })()`);
  results.push({
    iter: i, lobby_ready: ready, interactive, to_interactive_ms: toInteractiveMs,
    worker_init_timings: workerTimings, start_marks: startMarks, perf,
  });
  c.ws.close();
  chrome.kill("SIGTERM");
  await sleep(400);
  rmSync(udd, { recursive: true, force: true });
}

const ms = results.map((r) => r.to_interactive_ms).sort((a, b) => a - b);
console.log("RESULT scenario=browser_coldstart tier=web " + JSON.stringify({
  iters,
  to_interactive_ms: ms,
  p50_ms: ms[Math.floor((ms.length - 1) * 0.5)],
  max_ms: ms[ms.length - 1],
  all_interactive: results.every((r) => r.interactive === 1),
  console_errors: consoleErrors,
}));
writeFileSync(join(outDir, "browser-coldstart.json"), JSON.stringify({ results, consoleErrors }, null, 2));
console.log("NOTE headless Chrome + 本机静态服务；含模块下载/编译与首屏，不含真实网络与磁盘冷缓存。");
