// 表现层帧间隔与输入延迟采样驱动：headless Chrome + 真实构建产物 + 真实原生 WASM Worker。
//
// 驱动经 CDP 派发真实鼠标事件点击「自动一步」，直到终局或步数上限；页面侧由
// `src/engine/perfProbe.ts`（`perf=1` 开启）记录帧时间戳、Long Task、Event Timing
// 与逐请求分段。本脚本只读取这些样本并落盘，不 mock 引擎、不改业务行为。
//
// 用法（在 apps/web-client 下）：
//   node --import ./scripts/ts-register.mjs scripts/web-frame-latency.mjs \
//     --dist <构建目录> --out <输出目录> [--players 4] [--seed 42] [--mode identity|prototype] \
//     [--driver click|auto] [--max-steps 800] [--gap-ms 50] [--idle-ms 3000] [--chrome <路径>]
//     [--gpu] [--skip-anims] [--headed] [--coop] [--latency sab] [--trace <trace.json>]
//
// --headed：有头窗口（真实显示器呈现节奏；GPU 默认开启）。PA08 扩面。
//
// 输出：idle-frames.csv、frames.csv、interactions.csv、long-tasks.csv、event-timings.csv、
// summary.json，以及一行 `RESULT scenario=web_frame_latency ...`。

import { spawn } from "node:child_process";
import { createServer } from "node:http";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { extname, join, normalize, resolve } from "node:path";
import {
  frameDeltas,
  summarize,
  summarizeFrames,
  summarizeInteractions,
} from "../src/engine/perfStats.ts";

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i++) {
    const key = argv[i];
    if (!key.startsWith("--")) continue;
    const name = key.slice(2);
    const next = argv[i + 1];
    if (next === undefined || next.startsWith("--")) out[name] = true;
    else {
      out[name] = next;
      i++;
    }
  }
  return out;
}

const args = parseArgs(process.argv.slice(2));
if (!args.dist || !args.out) {
  console.error("用法：--dist <构建目录> --out <输出目录> [--players N] [--seed S] [--mode identity|prototype]");
  process.exit(2);
}
const distDir = resolve(String(args.dist));
const outDir = resolve(String(args.out));
const players = Number(args.players ?? 4);
const seed = Number(args.seed ?? 42);
const mode = String(args.mode ?? "identity");
const maxSteps = Number(args["max-steps"] ?? 800);
const gapMs = Number(args["gap-ms"] ?? 50);
const idleMs = Number(args["idle-ms"] ?? 3000);
const chromeBin = String(
  args.chrome ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
);
const useGpu = args.gpu === true;
const skipAnims = args["skip-anims"] === true;
// PA08：有头档——去掉 headless 与 disable-gpu，窗口按真实显示器节奏合成。
const headed = args.headed === true;
// click：每步一次真实鼠标点击「自动一步」；auto：点一次「连续自动」后由页面计时器推进（无输入对照）。
const driver = String(args.driver ?? "click");
// --trace：PA07 甄别——驱动段以 CDP Tracing 采集 DevTools trace（ReportEvents 流式），驱动结束写盘。
const tracePath = args.trace ? String(args.trace) : null;
const autoTimeoutMs = Number(args["auto-timeout-ms"] ?? 600_000);
if (driver !== "click" && driver !== "auto") throw new Error(`未知 --driver ${driver}`);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
mkdirSync(outDir, { recursive: true });

// ---- 静态服务（SPA 回退到 index.html；.wasm 用 application/wasm 以便流式实例化）----
const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".wasm": "application/wasm",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".webp": "image/webp",
  ".ogg": "audio/ogg",
  ".mp3": "audio/mpeg",
};
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url ?? "/", "http://x").pathname);
  let file = normalize(join(distDir, path));
  if (!file.startsWith(distDir)) {
    res.writeHead(403).end();
    return;
  }
  if (!existsSync(file) || statSync(file).isDirectory()) file = join(distDir, "index.html");
  const headers = { "Content-Type": MIME[extname(file)] ?? "application/octet-stream" };
  // --coop：跨源隔离头（PA07 SAB 缓解原型 A/B 需要SharedArrayBuffer）
  if (args.coop) {
    headers["Cross-Origin-Opener-Policy"] = "same-origin";
    headers["Cross-Origin-Embedder-Policy"] = "require-corp";
  }
  res.writeHead(200, headers);
  res.end(readFileSync(file));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;

// ---- Chrome + CDP ----
const udd = mkdtempSync(join(tmpdir(), "web-frame-latency-"));
const chromeArgs = [
  "--no-sandbox",
  "--no-first-run",
  `--user-data-dir=${udd}`,
  "--remote-debugging-port=0",
  "--window-size=1280,800",
  "about:blank",
];
if (!headed) chromeArgs.unshift("--headless=new");
if (!headed && !useGpu) chromeArgs.splice(1, 0, "--disable-gpu");
const chrome = spawn(chromeBin, chromeArgs, { stdio: "ignore" });

let port = null;
for (let k = 0; k < 100 && port === null; k++) {
  try {
    port = Number(readFileSync(join(udd, "DevToolsActivePort"), "utf8").split("\n")[0]);
  } catch {
    await sleep(100);
  }
}
if (!port) throw new Error("环境失败：Chrome 远程调试端口未就绪");
let page = null;
for (let k = 0; k < 60 && !page; k++) {
  try {
    const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
    page = list.find((t) => t.type === "page" && t.webSocketDebuggerUrl) ?? null;
  } catch {
    /* 未就绪 */
  }
  if (!page) await sleep(100);
}
if (!page) throw new Error("环境失败：CDP 页面未就绪");

const ws = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((r, j) => {
  ws.onopen = r;
  ws.onerror = j;
});
let msgId = 0;
const pending = new Map();
const consoleErrors = [];
const traceEvents = [];
let tracingDone = null;
ws.onmessage = (ev) => {
  const m = JSON.parse(ev.data);
  if (m.method === "Tracing.dataCollected") {
    traceEvents.push(...(m.params?.value ?? []));
    return;
  }
  if (m.method === "Tracing.tracingComplete") {
    tracingDone?.();
    return;
  }
  if (m.id && pending.has(m.id)) {
    const { res, rej } = pending.get(m.id);
    pending.delete(m.id);
    m.error ? rej(new Error(JSON.stringify(m.error))) : res(m.result);
    return;
  }
  if (m.method === "Runtime.exceptionThrown") {
    consoleErrors.push(`exception: ${m.params?.exceptionDetails?.exception?.description ?? m.params?.exceptionDetails?.text ?? "?"}`);
  }
  if (m.method === "Runtime.consoleAPICalled" && m.params?.type === "error") {
    consoleErrors.push(`console.error: ${(m.params.args ?? []).map((a) => a.value ?? a.description ?? "?").join(" ")}`);
  }
};
const send = (method, params = {}) => {
  const id = ++msgId;
  ws.send(JSON.stringify({ id, method, params }));
  return new Promise((res, rej) => pending.set(id, { res, rej }));
};
const evalJs = async (expression) => {
  const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
  return r.result?.value;
};
const browserVersion = await send("Browser.getVersion").catch(() => null);
await send("Runtime.enable");
await send("Page.enable");

// 先在同源页预置开关（localStorage 按源隔离），再进入战斗页
await send("Page.navigate", { url: `${base}/` });
await sleep(1500);
await evalJs(`(() => {
  localStorage.setItem("noname-next-perf", "1");
  ${skipAnims ? 'localStorage.setItem("noname-next-skip-anims", "1");' : 'localStorage.removeItem("noname-next-skip-anims");'}
  return true;
})()`);
// SAB 探针：新文档级钩子（导航换文档后原型补丁会丢，须 addScriptToEvaluateOnNewDocument）
await send("Page.addScriptToEvaluateOnNewDocument", {
  source: `(() => {
    const w = window; const orig = Worker.prototype.postMessage;
    Worker.prototype.postMessage = function (msg, ...rest) {
      try { if (msg && msg.type === "init") w.__initLatencySab = !!msg.latencySab; } catch {}
      return orig.call(this, msg, ...rest);
    };
  })()`,
}).catch(() => {});const query = new URLSearchParams({ players: String(players), seed: String(seed), perf: "1" });
if (mode === "identity") query.set("mode", "identity");
// --latency sab：PA07 缓解原型（回包 SAB 镜像 + rAF 轮询提前应用）
if (String(args.latency ?? "") === "sab") query.set("latency", "sab");
await send("Page.navigate", { url: `${base}/battle?${query}` });

const BUTTON = `Array.from(document.querySelectorAll('button')).find((b) => (b.innerText || '').trim() === '自动一步')`;
let ready = false;
for (let k = 0; k < 150 && !ready; k++) {
  await sleep(200);
  ready = await evalJs(`(() => {
    const b = ${BUTTON};
    return !!(window.__nonamePerf && b && !b.disabled && /rev \\d+/.test(document.body.innerText));
  })()`).catch(() => false);
}
if (!ready) throw new Error("测量失败：战斗页未就绪（无 __nonamePerf 或「自动一步」不可用）");
// 隔离态探针（--coop/--latency sab 的前提；PA07 A/B 诊断）
const coi = await evalJs("crossOriginIsolated").catch(() => null);
console.error(`crossOriginIsolated=${coi}`);
const sabProbe = await evalJs("window.__initLatencySab ?? 'no-init-seen'").catch(() => "probe-error");
console.error(`initLatencySab=${sabProbe}`);

// ---- 空闲基线 ----
await evalJs(`window.__nonamePerf.reset()`);
await sleep(idleMs);
const idleFrames = await evalJs(`window.__nonamePerf.frames()`);

// ---- 驱动 ----
await evalJs(`window.__nonamePerf.reset()`);
// 等待本次点击结算完毕（按钮重新可用或终局）；用 MutationObserver 而非轮询，减少对主线程的扰动。
const waitIdle = `new Promise((resolve) => {
  const status = () => {
    const b = ${BUTTON};
    if (!b) return "gone";
    if (/终局/.test(document.body.innerText)) return "finished";
    return b.disabled ? null : "ready";
  };
  const first = status();
  if (first) return resolve(first);
  const obs = new MutationObserver(() => {
    const s = status();
    if (s) { obs.disconnect(); resolve(s); }
  });
  obs.observe(document.body, { attributes: true, subtree: true, childList: true, attributeFilter: ["disabled"] });
  setTimeout(() => { obs.disconnect(); resolve("timeout"); }, 15000);
})`;
const startedAt = Date.now();
let clicks = 0;
let timeouts = 0;
let endState = "max_steps";
if (tracePath) {
  await send("Tracing.start", {
    transferMode: "ReportEvents",
    traceConfig: {
      includedCategories: [
        "devtools.timeline",
        "disabled-by-default-devtools.timeline.frame",
        "benchmark",
      ],
      excludedCategories: ["*"],
    },
  });
}
async function clickAt(x, y) {
  const at = { x, y, button: "left", clickCount: 1 };
  await send("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
  await send("Input.dispatchMouseEvent", { type: "mousePressed", ...at });
  await send("Input.dispatchMouseEvent", { type: "mouseReleased", ...at });
}
if (driver === "auto") {
  // 对照档：点一次「连续自动」，此后由页面计时器推进（请求无关联输入事件），驱动端每秒查一次是否终局。
  const rect = await evalJs(`(() => {
    const b = Array.from(document.querySelectorAll('button')).find((x) => (x.innerText || '').trim() === '连续自动');
    if (!b) return null;
    const r = b.getBoundingClientRect();
    return { x: r.left + r.width / 2, y: r.top + r.height / 2 };
  })()`);
  if (!rect) throw new Error("测量失败：找不到「连续自动」按钮");
  await clickAt(rect.x, rect.y);
  clicks = 1;
  const deadline = Date.now() + autoTimeoutMs;
  while (Date.now() < deadline) {
    await sleep(1000);
    if (await evalJs(`/终局/.test(document.body.innerText)`)) {
      endState = "finished";
      break;
    }
  }
  if (endState !== "finished") endState = "auto_timeout";
}
for (let i = 0; driver === "click" && i < maxSteps; i++) {
  const state = await evalJs(`(() => {
    const b = ${BUTTON};
    if (/终局/.test(document.body.innerText)) return { finished: true };
    if (!b) return { gone: true };
    const r = b.getBoundingClientRect();
    return { x: r.left + r.width / 2, y: r.top + r.height / 2, disabled: b.disabled };
  })()`);
  if (state.finished) {
    endState = "finished";
    break;
  }
  if (state.gone) {
    endState = "button_gone";
    break;
  }
  if (state.disabled) {
    const w = await evalJs(waitIdle);
    if (w === "timeout") timeouts++;
    continue;
  }
  await clickAt(state.x, state.y);
  clicks++;
  const w = await evalJs(waitIdle);
  if (w === "timeout") timeouts++;
  if (w === "finished") {
    endState = "finished";
    break;
  }
  await sleep(gapMs);
}
const wallMs = Date.now() - startedAt;
await sleep(500);
const raw = await evalJs(`JSON.stringify({
  frames: window.__nonamePerf.frames(),
  longTasks: window.__nonamePerf.longTasks(),
  eventTimings: window.__nonamePerf.eventTimings(),
  interactions: window.__nonamePerf.interactions(),
  status: (document.body.innerText.match(/终局[^\\n]*/) ?? document.body.innerText.match(/rev \\d+[^\\n]*/))?.[0] ?? null,
  ua: navigator.userAgent,
  dpr: window.devicePixelRatio,
  viewport: [window.innerWidth, window.innerHeight],
})`);
const data = JSON.parse(raw);

// ---- 落盘 ----
function writeCsv(name, rows, columns) {
  const esc = (v) => (v === null || v === undefined ? "" : typeof v === "string" ? `"${v.replace(/"/g, '""')}"` : String(v));
  const lines = [columns.join(",")];
  for (const row of rows) lines.push(columns.map((c) => esc(row[c])).join(","));
  writeFileSync(join(outDir, name), lines.join("\n") + "\n");
}
const framesToRows = (ts) => ts.map((t, i) => ({ index: i, ts_ms: t, delta_ms: i ? t - ts[i - 1] : null }));
writeCsv("idle-frames.csv", framesToRows(idleFrames), ["index", "ts_ms", "delta_ms"]);
writeCsv("frames.csv", framesToRows(data.frames), ["index", "ts_ms", "delta_ms"]);
const interactionColumns = data.interactions.length ? Object.keys(data.interactions[0]) : ["id"];
writeCsv("interactions.csv", data.interactions, interactionColumns);
writeCsv("long-tasks.csv", data.longTasks, ["start", "duration"]);
writeCsv("event-timings.csv", data.eventTimings, ["name", "start", "duration", "input_delay", "processing"]);

const clickSteps = data.interactions.filter((r) => r.type === "autoStep" && r.input_kind === "click");
const timerSteps = data.interactions.filter((r) => r.type === "autoStep" && r.input_kind === null);
const mainSteps = driver === "click" ? clickSteps : timerSteps;
const summary = {
  config: {
    players,
    seed,
    mode,
    max_steps: maxSteps,
    driver,
    gap_ms: gapMs,
    idle_ms: idleMs,
    gpu: useGpu,
    headed,
    skip_anims: skipAnims,
    chrome: browserVersion?.product ?? null,
    user_agent: data.ua,
    dpr: data.dpr,
    viewport: data.viewport,
    dist: distDir,
  },
  run: {
    clicks,
    timeouts,
    end_state: endState,
    wall_ms: wallMs,
    final_status: data.status,
    console_errors: consoleErrors,
  },
  idle_frames: summarizeFrames(frameDeltas(idleFrames)),
  frames: summarizeFrames(frameDeltas(data.frames)),
  long_tasks: { ...summarize(data.longTasks.map((t) => t.duration)), total_ms: data.longTasks.reduce((a, t) => a + t.duration, 0) },
  event_timings: summarize(data.eventTimings.map((t) => t.duration)),
  click_autostep: summarizeInteractions(clickSteps),
  timer_autostep: summarizeInteractions(timerSteps),
  all_requests: summarizeInteractions(data.interactions),
};
writeFileSync(join(outDir, "summary.json"), JSON.stringify(summary, null, 2));
// SAB 调试探针（latency=sab）：seq 是否增长、提前解析计数
const sabDbg = await evalJs(
  "window.__latencySabDebug ? {seq: window.__latencySabDebug.seq(), early: window.__latencySabDebug.resolvedEarly} : null",
).catch(() => null);
if (sabDbg) console.error(`sabDebug=${JSON.stringify(sabDbg)}`);
const pick = (s) => (s ? { p50: s.p50, p95: s.p95, p99: s.p99, max: s.max, n: s.count } : null);
console.log(
  "RESULT scenario=web_frame_latency " +
    JSON.stringify({
      players,
      mode,
      driver,
      gpu: useGpu,
      headed,
      clicks,
      end_state: endState,
      final_status: data.status,
      frame_ms: pick(summary.frames),
      over_1_5x_refresh: summary.frames.over_1_5x_refresh_ratio,
      steps: mainSteps.length,
      input_to_paint_ms: pick(summarizeInteractions(mainSteps).input_to_paint_ms),
      roundtrip_ms: pick(summarizeInteractions(mainSteps).roundtrip_ms),
      transfer_ms: pick(summarizeInteractions(mainSteps).transfer_ms),
      long_tasks: summary.long_tasks.count,
      console_errors: consoleErrors.length,
    }),
);

if (tracePath) {
  const done = new Promise((r) => {
    tracingDone = r;
    setTimeout(r, 10_000);
  });
  await send("Tracing.end").catch(() => {});
  await done;
  writeFileSync(tracePath, JSON.stringify({
    source: "web-frame-latency --trace",
    browser: browserVersion,
    headed,
    driver,
    latency: String(args.latency ?? ""),
    players,
    seed,
    clicks,
    events: traceEvents,
  }));
  console.error(`trace=${tracePath} events=${traceEvents.length}`);
}
ws.close();
chrome.kill("SIGTERM");
server.close();
await sleep(300);
rmSync(udd, { recursive: true, force: true });
