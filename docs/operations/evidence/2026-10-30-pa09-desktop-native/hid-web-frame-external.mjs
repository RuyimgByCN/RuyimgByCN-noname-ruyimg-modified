// PA08 扩面：无 CDP 环境的采样驱动（Tauri WKWebView / Safari / Firefox / 移动端浏览器）。
//
// 页面侧由 src/engine/autoDrive.ts（autodrive 参数门控）自驱动并把采样 POST 回本脚本；
// 本脚本提供静态服务（COOP/COEP 跨源隔离头）、/report 接收端、按 --app 拉起目标
// 浏览器/应用，并按 CDP 驱动同口径落盘 CSV 与 summary.json。
//
// 用法（apps/web-client 下）：
//   node --import ./scripts/ts-register.mjs scripts/web-frame-external.mjs \
//     --dist <构建目录> --out <输出目录> [--players 4] [--seed 42] \
//     [--driver click|auto|manual] [--steps 1500] [--gap 50] [--timeout 180000] \
//     [--app chrome-headed|safari|firefox|tauri|none] [--binary <应用路径>] [--port 5173]
//
// 说明：--driver click 为页面内合成点击（帧节奏/Worker 链路口径）；真实输入延迟用
// --driver manual 配合系统级点击工具（perfProbe 捕获真实事件时间戳）。

import { spawn } from "node:child_process";
import assert from "node:assert/strict";
import { createInterface } from "node:readline";
import { createServer } from "node:http";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { extname, join, normalize, resolve } from "node:path";
import {
  frameDeltas,
  summarize,
  summarizeFrames,
  summarizeInteractions,
} from "/Users/ruyimg/Documents/Claude/sanguosha/noname-next/apps/web-client/src/engine/perfStats.ts";

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

function screenPoint(frame, ready) {
  const { viewport, rect } = ready;
  assert.ok([frame.x, frame.y, frame.width, frame.height, viewport.width, viewport.height,
    rect.x, rect.y, rect.width, rect.height].every(Number.isFinite));
  assert.ok(Math.abs(frame.width - viewport.width) <= 2, "AX/CSS width mismatch");
  const inset = frame.height - viewport.height;
  assert.ok(inset >= 0 && inset <= 100, "unverified title/content inset");
  assert.ok(rect.width > 0 && rect.height > 0 && rect.x >= 0 && rect.y >= 0);
  assert.ok(rect.x + rect.width <= viewport.width && rect.y + rect.height <= viewport.height);
  // AX and DOM use logical pixels. DPR is recorded, never used as an implicit scale.
  return { x: frame.x + rect.x + rect.width / 2, y: frame.y + inset + rect.y + rect.height / 2 };
}
if (process.argv.includes("--self-check")) {
  const frame = { x: 100, y: 80, width: 1280, height: 820 };
  const ready = { viewport: { width: 1280, height: 788 }, rect: { x: 40, y: 50, width: 80, height: 30 }, dpr: 2 };
  assert.deepEqual(screenPoint(frame, ready), { x: 180, y: 177 });
  assert.deepEqual(screenPoint(frame, { ...ready, dpr: 1 }), { x: 180, y: 177 });
  assert.throws(() => screenPoint({ ...frame, width: 640 }, ready), /width mismatch/);
  assert.throws(() => screenPoint(frame, { ...ready, rect: { ...ready.rect, x: 1300 } }));
  console.log("HID coordinate self-check: 4 passed; no application or input invoked");
  process.exit(0);
}
const args = parseArgs(process.argv.slice(2));
if (!args.dist || !args.out) {
  console.error("用法：--dist <构建目录> --out <输出目录> [--players N] [--seed S] [--driver click|auto|manual] [--app ...]");
  process.exit(2);
}
const distDir = resolve(String(args.dist));
const outDir = resolve(String(args.out));
const players = Number(args.players ?? 4);
const seed = Number(args.seed ?? 42);
const driver = String(args.driver ?? "click");
const steps = Number(args.steps ?? 1500);
const gapMs = Number(args.gap ?? 50);
const timeoutMs = Number(args.timeout ?? 180_000);
const app = String(args.app ?? "none");
const port = Number(args.port ?? (app === "tauri" ? 5173 : 0));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
mkdirSync(outDir, { recursive: true });

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".wasm": "application/wasm",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".webp": "image/webp",
  ".ogg": "audio/ogg",
  ".mp3": "audio/mpeg",
};

const mode = String(args.mode ?? "identity");
// PA08-4 压测载荷：--packs <安装载荷 JSON> / --hex <存档十六进制> 由静态服务在
// /packs、/hex 提供，页面经 packsUrl/hexUrl 拉取（避免 URL 长度上限）。
const packsFile = args.packs ? resolve(String(args.packs)) : null;
const hexFile = args.hex ? resolve(String(args.hex)) : null;
if (packsFile && !existsSync(packsFile)) {
  console.error(`--packs 文件不存在：${packsFile}`);
  process.exit(2);
}
if (hexFile && !existsSync(hexFile)) {
  console.error(`--hex 文件不存在：${hexFile}`);
  process.exit(2);
}
const battleQuery = new URLSearchParams({
  players: String(players),
  seed: String(seed),
  mode,
  perf: "1",
  autodrive: driver,
  steps: String(steps),
  gap: String(gapMs),
});
// PA08-5 长时：loop=1 连续多局 + duration=<ms> 总时长（auto 档）
if (args.loop) battleQuery.set("loop", "1");
if (args.duration) battleQuery.set("duration", String(Number(args.duration)));
if (args["engine-transport"] === "native" || args["engine-transport"] === "wasm") {
  battleQuery.set("engineTransport", args["engine-transport"]);
}
// PA07 缓解原型 A/B：latency=sab（回包 SAB 镜像 + rAF 轮询提前应用；需跨源隔离）
if (args.latency === "sab") {
  battleQuery.set("latency", "sab");
  args.coop = true; // SAB 需要跨源隔离头
}

let reportData = null;
let hidReady = null;
let hidProcess = null;
let hidCalibration = null;
const hidTool = args["hid-tool"] ? resolve(String(args["hid-tool"])) : null;
if (hidTool) assert.equal(driver, "manual", "real-HID mode requires manual page driver");
const server = createServer((req, res) => {
  // COOP/COEP 仅在 --coop 显式开启（SAB 缓解原型需要跨源隔离）。
  // 默认不发：Safari/Firefox 进入 crossOriginIsolated 后 Worker 往返失败
  //（2026-10-26 实测：整局卡选将，控制台「Worker 往返失败: 选择失败」）；
  // CDP 驱动（web-frame-latency.mjs）亦不发，保持口径一致。
  if (args.coop) {
    res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
    res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
  }
  if (req.method === "POST" && req.url === "/report") {
    let body = "";
    req.on("data", (c) => (body += c));
    req.on("end", () => {
      try {
        reportData = JSON.parse(body);
        res.writeHead(204).end();
        console.error(`report received (${body.length} bytes)`);
      } catch {
        res.writeHead(400).end();
      }
    });
    return;
  }
  if (req.url === "/hid-ready" && req.method === "POST") {
    let body = "";
    req.on("data", (chunk) => { body += chunk; if (body.length > 16384) req.destroy(); });
    req.on("end", () => {
      try { hidReady = JSON.parse(body); res.writeHead(204).end(); }
      catch { res.writeHead(400).end(); }
    });
    return;
  }
  if (req.url === "/pa09-hid-bootstrap.js") {
    const target = `${battleUrl}&report=/report`;
    const boot = `(() => {
      if (!location.hash.startsWith("#/battle")) location.replace(${JSON.stringify(target)});
      const clicks = [];
      addEventListener("click", (event) => {
        const button = event.target instanceof Element ? event.target.closest("button") : null;
        if (button?.textContent?.trim() === "自动一步") clicks.push({ trusted: event.isTrusted, timestamp: event.timeStamp });
      }, true);
      const original = window.fetch.bind(window);
      window.fetch = (input, init) => {
        if (typeof input === "string" && new URL(input, location.href).pathname === "/report" && typeof init?.body === "string") {
          const payload = JSON.parse(init.body);
          payload.hid_evidence = { clicks, tauri: typeof window.__TAURI_INTERNALS__?.invoke === "function" };
          return original(input, { ...init, body: JSON.stringify(payload) });
        }
        return original(input, init);
      };
      const timer = setInterval(() => {
        const button = [...document.querySelectorAll("button")].find((item) => item.textContent?.trim() === "自动一步");
        if (!button || button.disabled) return;
        const rect = button.getBoundingClientRect();
        if (!rect.width || !rect.height || !button.contains(document.elementFromPoint(rect.x + rect.width / 2, rect.y + rect.height / 2))) return;
        clearInterval(timer);
        original("/hid-ready", { method: "POST", body: JSON.stringify({ viewport: { width: innerWidth, height: innerHeight }, dpr: devicePixelRatio, rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height } }) });
      }, 100);
    })();`;
    res.writeHead(200, { "Content-Type": "text/javascript; charset=utf-8" }).end(boot);
    return;
  }
  const path = decodeURIComponent(new URL(req.url ?? "/", "http://x").pathname);
  if (packsFile && path === "/packs") {
    res.writeHead(200, { "Content-Type": "application/json" }).end(readFileSync(packsFile));
    return;
  }
  if (hexFile && path === "/hex") {
    res.writeHead(200, { "Content-Type": "text/plain; charset=utf-8" }).end(
      readFileSync(hexFile, "utf8"),
    );
    return;
  }
  let file = normalize(join(distDir, path));
  if (!file.startsWith(distDir)) {
    res.writeHead(403).end();
    return;
  }
  if (!existsSync(file) || statSync(file).isDirectory()) file = join(distDir, "index.html");
  const headers = { "Content-Type": MIME[extname(file)] ?? "application/octet-stream" };
  if (extname(file) === ".html") {
    if (path === "/") {
      // 仅根路径注入自驱动引导（Tauri devUrl 固定加载根路径）。battle 直开时 URL
      // 已带全部参数：若在此也注入，replace 到「同路径不同 query」会整页重载并
      // 再次注入＝无限重载循环（path 路由下实测；hash 路由无此问题因 replace 不换文档）。
      const boot = hidTool ? '<script src="/pa09-hid-bootstrap.js"></script>' : `<script>location.replace(${JSON.stringify(
        `${battleUrl}&report=http://${req.headers.host}/report`,
      )})</script>`;
      const html = readFileSync(file, "utf8").replace("</head>", `${boot}</head>`);
      res.writeHead(200, headers).end(html);
      return;
    }
    res.writeHead(200, headers).end(readFileSync(file));
    return;
  }
  res.writeHead(200, headers).end(readFileSync(file));
});
await new Promise((r) => server.listen(port, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;
// 相对路径（按页面源解析）：Tauri devUrl 是 localhost 而 server 绑 127.0.0.1，
// 绝对地址会跨源被 WKWebView/CSP 拦截。
if (packsFile) battleQuery.set("packsUrl", "/packs");
if (hexFile) battleQuery.set("hexUrl", "/hex");
// 路由模式随宿主分叉（router.ts：Tauri=hash、浏览器=HTML5 path）——
// 浏览器收到 /#/battle 会落回大厅路由，必须用 path 形态。
const battleRoute = app === "tauri" ? "/#/battle" : "/battle";
const battleUrl = `${battleRoute}?${battleQuery}`;
const entry = `${base}/`;
console.error(`serving ${distDir} at ${entry}`);

// ---- 拉起目标 ----
let child = null;
process.once("exit", () => {
  if (child && app !== "safari" && app !== "firefox") { try { child.kill("SIGTERM"); } catch {} }
  if (hidProcess) { try { hidProcess.kill("SIGTERM"); } catch {} }
  server.close();
});
process.once("SIGINT", () => process.exit(130));
process.once("SIGTERM", () => process.exit(143));
const fullBattleUrl = `${base}${battleUrl}&report=${base}/report`;
if (app === "chrome-headed") {
  // 直传完整 battle URL（path 路由）：经根路径 + 注入引导会被 hash 形态/解析期
  // replace 差异坑住，浏览器目标一律直接打开最终 URL。
  child = spawn(
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ["--no-sandbox", "--no-first-run", `--user-data-dir=/tmp/pa08-external-chrome`, fullBattleUrl],
    { stdio: "ignore", detached: false },
  );
} else if (app === "safari" || app === "firefox") {
  const name = app === "safari" ? "Safari" : "Firefox";
  child = spawn("open", ["-a", name, fullBattleUrl], {
    stdio: "ignore",
  });
} else if (app === "tauri") {
  const bin = String(args.binary ?? "");
  if (!bin || !existsSync(bin)) {
    console.error(`--binary <桌面二进制路径> 必填（tauri）`);
    process.exit(2);
  }
  child = spawn(bin, [], { stdio: "ignore" });
}

// ---- Optional real system input; no screenshots or permission-request APIs ----
if (hidTool) {
  assert.equal(app, "tauri");
  assert.ok(child?.pid && existsSync(hidTool));
  const untilReady = Date.now() + 20000;
  while (!hidReady && Date.now() < untilReady) await sleep(100);
  assert.ok(hidReady, "no real enabled-button geometry received");
  hidProcess = spawn(hidTool, [String(child.pid)], { stdio: ["pipe", "pipe", "inherit"] });
  const lines = createInterface({ input: hidProcess.stdout })[Symbol.asyncIterator]();
  const readReply = async () => {
    let timeout;
    try {
      const item = await Promise.race([lines.next(), new Promise((_, reject) => {
        timeout = setTimeout(() => reject(new Error("HID controller reply timed out")), 5000);
      })]);
      assert.ok(!item.done, "HID controller ended before reply");
      const value = JSON.parse(item.value);
      assert.ok(!value.error, value.error);
      return value;
    } finally { clearTimeout(timeout); }
  };
  const first = await readReply();
  assert.equal(first.pid, child.pid);
  const point = screenPoint(first.frame, hidReady);
  hidCalibration = { frame: first.frame, ready: hidReady, point, gap_ms: gapMs };
  writeFileSync(join(outDir, "hid-calibration.json"), JSON.stringify(hidCalibration, null, 2));
  for (let sequence = 0; sequence < steps && !reportData; sequence++) {
    hidProcess.stdin.write(JSON.stringify({ ...point, sequence }) + "\n");
    const posted = await readReply();
    assert.equal(posted.posted, true);
    assert.equal(posted.sequence, sequence);
    await sleep(gapMs);
  }
  hidProcess.stdin.end();
}
// ---- 等报告 ----
const deadline = Date.now() + timeoutMs;
while (!reportData && Date.now() < deadline) await sleep(500);
if (child && app !== "safari" && app !== "firefox") {
  try {
    child.kill("SIGTERM");
  } catch { /* 已退出 */ }
}
if (!reportData) {
  console.error(`TIMEOUT：${timeoutMs}ms 内未收到报告（autodrive=${driver} app=${app}）`);
  server.close();
  process.exit(1);
}

// ---- 落盘（与 CDP 驱动同口径） ----
function writeCsv(name, rows, columns) {
  const esc = (v) =>
    v === null || v === undefined ? "" : typeof v === "string" ? `"${v.replace(/"/g, '""')}"` : String(v);
  const lines = [columns.join(",")];
  for (const row of rows) lines.push(columns.map((c) => esc(row[c])).join(","));
  writeFileSync(join(outDir, name), lines.join("\n") + "\n");
}
const framesToRows = (ts) => ts.map((t, i) => ({ index: i, ts_ms: t, delta_ms: i ? t - ts[i - 1] : null }));
writeCsv("frames.csv", framesToRows(reportData.frames), ["index", "ts_ms", "delta_ms"]);
const interactionColumns = reportData.interactions.length ? Object.keys(reportData.interactions[0]) : ["id"];
writeCsv("interactions.csv", reportData.interactions, interactionColumns);
writeCsv("long-tasks.csv", reportData.longTasks, ["start", "duration"]);
writeCsv("event-timings.csv", reportData.eventTimings, ["name", "start", "duration", "input_delay", "processing"]);

const clickSteps = reportData.interactions.filter((r) => r.type === "autoStep" && r.input_kind === "click");
if (hidTool) {
  assert.equal(reportData.hid_evidence?.tauri, true);
  assert.equal(reportData.hid_evidence?.clicks.length, steps, "button clicks lost/duplicated");
  assert.ok(reportData.hid_evidence.clicks.every((item) => item.trusted === true), "untrusted click");
  assert.equal(clickSteps.length, steps, "trusted clicks did not all produce measured autoStep requests");
}
writeFileSync(join(outDir, "raw-report.json"), JSON.stringify(reportData, null, 2));
const timerSteps = reportData.interactions.filter((r) => r.type === "autoStep" && r.input_kind === null);
const mainSteps = driver === "auto" ? timerSteps : clickSteps.length ? clickSteps : reportData.interactions;
const summary = {
  config: {
    players,
    seed,
    mode,
    driver,
    steps,
    gap_ms: gapMs,
    app,
    user_agent: reportData.ua,
    dpr: reportData.dpr,
    viewport: reportData.viewport,
    href: reportData.href,
    engine_transport: reportData.engine_transport ?? null,
    dist: distDir,
  },
  run: {
    end_state: reportData.end_state,
    wall_ms: reportData.wall_ms,
    final_status: reportData.status,
    // PA08-5 长时内存采样（页面 performance.memory，Chrome 系；无则 null）
    memory: reportData.memory ?? null,
    memory_series: reportData.memory_series ?? [],
  },
  frames: summarizeFrames(frameDeltas(reportData.frames)),
  long_tasks: {
    ...summarize(reportData.longTasks.map((t) => t.duration)),
    total_ms: reportData.longTasks.reduce((a, t) => a + t.duration, 0),
  },
  event_timings: summarize(reportData.eventTimings.map((t) => t.duration)),
  click_autostep: summarizeInteractions(clickSteps),
  timer_autostep: summarizeInteractions(timerSteps),
  all_requests: summarizeInteractions(reportData.interactions),
};
writeFileSync(join(outDir, "summary.json"), JSON.stringify(summary, null, 2));
const pick = (s) => (s ? { p50: s.p50, p95: s.p95, p99: s.p99, max: s.max, n: s.count } : null);
console.log(
  "RESULT scenario=web_frame_external " +
    JSON.stringify({
      players,
      driver,
      app,
      end_state: reportData.end_state,
      final_status: reportData.status,
      frame_ms: pick(summary.frames),
      steps: mainSteps.length,
      input_to_paint_ms: pick(summarizeInteractions(mainSteps).input_to_paint_ms),
      roundtrip_ms: pick(summarizeInteractions(mainSteps).roundtrip_ms),
      transfer_ms: pick(summarizeInteractions(mainSteps).transfer_ms),
      long_tasks: summary.long_tasks.count,
      memory_mb: reportData.memory ? Math.round(reportData.memory.used / 1048576) : null,
    }),
);
server.close();
if (app === "safari" || app === "firefox") {
  // open 拉起的窗口不属于本进程，输出提示由人工/外部关闭
  console.error("note: 浏览器窗口由 open 拉起，请自行关闭");
}
await sleep(300);
process.exit(0);
