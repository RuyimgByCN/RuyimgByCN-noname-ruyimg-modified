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
import { createServer } from "node:http";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
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
const battleQuery = new URLSearchParams({
  players: String(players),
  seed: String(seed),
  mode,
  perf: "1",
  autodrive: driver,
  steps: String(steps),
  gap: String(gapMs),
});
const battleUrl = `/#/battle?${battleQuery}`;  // SPA hash 路由

let reportData = null;
const server = createServer((req, res) => {
  res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
  res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
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
  const path = decodeURIComponent(new URL(req.url ?? "/", "http://x").pathname);
  let file = normalize(join(distDir, path));
  if (!file.startsWith(distDir)) {
    res.writeHead(403).end();
    return;
  }
  if (!existsSync(file) || statSync(file).isDirectory()) file = join(distDir, "index.html");
  const headers = { "Content-Type": MIME[extname(file)] ?? "application/octet-stream" };
  if (extname(file) === ".html") {
    // 注入自驱动引导：在 Vue 模块脚本执行前设好 hash 路由（Tauri devUrl 只加载
    // 根路径；hash-only 的 replace 不换文档，故必须走注入而非重定向页）。
    const boot = `<script>location.replace(${JSON.stringify(
      `${battleUrl}&report=http://${req.headers.host}/report`,
    )})</script>`;
    const html = readFileSync(file, "utf8").replace("</head>", `${boot}</head>`);
    res.writeHead(200, headers).end(html);
    return;
  }
  res.writeHead(200, headers).end(readFileSync(file));
});
await new Promise((r) => server.listen(port, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;
const entry = `${base}/`;
console.error(`serving ${distDir} at ${entry}`);

// ---- 拉起目标 ----
let child = null;
if (app === "chrome-headed") {
  child = spawn(
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ["--no-sandbox", "--no-first-run", `--user-data-dir=/tmp/pa08-external-chrome`, entry],
    { stdio: "ignore", detached: false },
  );
} else if (app === "safari" || app === "firefox") {
  const name = app === "safari" ? "Safari" : "Firefox";
  child = spawn("open", ["-a", name, entry], { stdio: "ignore" });
} else if (app === "tauri") {
  const bin = String(args.binary ?? "");
  if (!bin || !existsSync(bin)) {
    console.error(`--binary <桌面二进制路径> 必填（tauri）`);
    process.exit(2);
  }
  child = spawn(bin, [], { stdio: "ignore" });
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
    dist: distDir,
  },
  run: {
    end_state: reportData.end_state,
    wall_ms: reportData.wall_ms,
    final_status: reportData.status,
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
    }),
);
server.close();
if (app === "safari" || app === "firefox") {
  // open 拉起的窗口不属于本进程，输出提示由人工/外部关闭
  console.error("note: 浏览器窗口由 open 拉起，请自行关闭");
}
await sleep(300);
process.exit(0);
