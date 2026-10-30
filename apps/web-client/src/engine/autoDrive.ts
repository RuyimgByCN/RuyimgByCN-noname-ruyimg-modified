// PA08 扩面：无 CDP 环境的自驱动与报告回传（Tauri WKWebView / Safari / Firefox / 移动端浏览器）。
//
// URL 参数（无 autodrive 参数时本模块完全惰性，不影响正常使用）：
//   autodrive=click|auto|manual
//     click  = 页面内逐次点击「自动一步」（合成事件——用于帧节奏与 Worker 链路计时，
//              不代表真实输入的调度行为）
//     auto   = 点击一次「连续自动」，由页面计时器推进
//     manual = 不自行驱动；等待外部真实输入（配合系统级点击工具），按捕获监听计数
//   report=<url>   终局或步数耗尽后 POST 完整采样 JSON 到该地址
//   steps=<n>      click/manual 模式步数上限（默认 1500）
//   gap=<ms>       click 模式步间隔（默认 50）
//   timeout=<ms>   manual 模式等待外部输入兜底（默认 120000）
//
// 采样内容与 CDP 驱动（web-frame-latency.mjs）一致：frames/longTasks/eventTimings/
// interactions（由 perf=1 的 perfProbe 记录）+ 状态文本/UA/DPR/视口。

const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));

const AUTOSTEP_BUTTON = "自动一步";
const AUTOALL_BUTTON = "连续自动";
const FINISHED_RE = /终局/;

/** PA08-5：driveAuto 期间采的内存趋势（30s 一次强制 GC 后取值）。 */
let autoDriveMemorySeries: Array<{ t_ms: number; bytes: number }> = [];
let startedAtAuto = 0;

function findButton(text: string): HTMLButtonElement | null {
  return (
    Array.from(document.querySelectorAll("button")).find(
      (b) => (b.innerText || "").trim() === text,
    ) as HTMLButtonElement | undefined
  ) ?? null;
}

/** 等到按钮可用（本步结算完毕）或终局；MutationObserver 降低轮询扰动。 */
function waitIdle(): Promise<"ready" | "finished" | "gone" | "timeout"> {
  return new Promise((resolve) => {
    const status = () => {
      const b = findButton(AUTOSTEP_BUTTON);
      if (!b) return "gone";
      if (FINISHED_RE.test(document.body.innerText)) return "finished";
      return b.disabled ? null : "ready";
    };
    const first = status();
    if (first) {
      resolve(first as "ready" | "finished" | "gone");
      return;
    }
    const obs = new MutationObserver(() => {
      const s = status();
      if (s) {
        obs.disconnect();
        resolve(s as "ready" | "finished" | "gone");
      }
    });
    obs.observe(document.body, {
      attributes: true,
      subtree: true,
      childList: true,
      attributeFilter: ["disabled"],
    });
    setTimeout(() => {
      obs.disconnect();
      resolve("timeout");
    }, 15000);
  });
}

interface PerfProbeApi {
  frames(): number[];
  longTasks(): Array<{ start: number; duration: number }>;
  eventTimings(): Array<Record<string, number | string>>;
  interactions(): Array<Record<string, unknown>>;
  reset(): void;
}

function perfApi(): PerfProbeApi | null {
  const w = window as unknown as { __nonamePerf?: PerfProbeApi };
  return w.__nonamePerf ?? null;
}

function collect() {
  const api = perfApi();
  const status =
    document.body.innerText.match(/终局[^\n]*/) ??
    document.body.innerText.match(/rev \d+[^\n]*/);
  return {
    frames: api?.frames() ?? [],
    longTasks: api?.longTasks() ?? [],
    eventTimings: api?.eventTimings() ?? [],
    interactions: api?.interactions() ?? [],
    status: status?.[0] ?? null,
    ua: navigator.userAgent,
    dpr: window.devicePixelRatio,
    viewport: [window.innerWidth, window.innerHeight],
    href: location.href,
    // SAFETY: EngineClient records the actual selected transport before opening its connection.
    engine_transport: (globalThis as unknown as { __engineTransport?: string }).__engineTransport ?? null,
    // PA08-5 长时内存采样（sampleMemory 在上报前覆写；series 为 driveAuto 周期采样）
    memory: null as { used: number; total: number; limit: number; api: string } | null,
    memory_series: [] as Array<{ t_ms: number; bytes: number }>,
  };
}

/**
 * PA08-5 长时内存采样：Chrome 旧 performance.memory 已在非跨源隔离页移除——
 * 优先旧 API，其次跨源隔离下的 measureUserAgentSpecificMemory（会强制 GC）。
 */
async function sampleMemory(): Promise<{
  used: number;
  total: number;
  limit: number;
  api: string;
} | null> {
  const perf = performance as unknown as {
    memory?: { usedJSHeapSize: number; totalJSHeapSize: number; jsHeapSizeLimit: number };
    measureUserAgentSpecificMemory?: () => Promise<{ bytes: number }>;
  };
  try {
    if (perf.memory) {
      const m = perf.memory;
      return { used: m.usedJSHeapSize, total: m.totalJSHeapSize, limit: m.jsHeapSizeLimit, api: "performance.memory" };
    }
    if (perf.measureUserAgentSpecificMemory && crossOriginIsolated) {
      const m = await perf.measureUserAgentSpecificMemory();
      return { used: m.bytes, total: m.bytes, limit: 0, api: "measureUserAgentSpecificMemory" };
    }
  } catch {
    /* 采样失败不阻断上报 */
  }
  return null;
}

async function report(url: string, payload: unknown) {
  try {
    await fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
  } catch {
    /* 上报失败不重试：外部脚本有超时兜底 */
  }
}

async function driveClick(steps: number, gapMs: number) {
  for (let i = 0; i < steps; i++) {
    const b = findButton(AUTOSTEP_BUTTON);
    if (!b || FINISHED_RE.test(document.body.innerText)) return "finished";
    if (b.disabled) {
      const w = await waitIdle();
      if (w === "finished") return "finished";
      continue;
    }
    b.click();
    const w = await waitIdle();
    if (w === "finished") return "finished";
    await sleep(gapMs);
  }
  return "max_steps";
}

async function driveAuto() {
  // loop=1 + duration=<ms>：终局后点「重新开局」连续多局（PA08-5 长时采样）；
  // 缺省行为不变——点一次「连续自动」，终局或 600s 兜底返回。
  const loop = urlParam("loop") === "1";
  const durationMs = Number(urlParam("duration") ?? 0);
  const deadline = Date.now() + (durationMs > 0 ? durationMs : 600_000);
  const finished = () => FINISHED_RE.test(document.body.innerText);
  // 内存趋势采样：优先同步 legacy performance.memory（跨源隔离下 Chrome 仍提供），
  // 无则试异步 measureUserAgentSpecificMemory。每 30s 一点，随报告带回。
  const memorySeries: Array<{ t_ms: number; bytes: number }> = [];
  const legacyMem = (
    performance as unknown as { memory?: { usedJSHeapSize: number } }
  ).memory;
  const memApi = (
    performance as unknown as {
      measureUserAgentSpecificMemory?: () => Promise<{ bytes: number }>;
    }
  ).measureUserAgentSpecificMemory;
  const sampleMem = async () => {
    if (legacyMem) {
      memorySeries.push({ t_ms: Date.now() - startedAtAuto, bytes: legacyMem.usedJSHeapSize });
      return;
    }
    if (!memApi || !crossOriginIsolated) return;
    try {
      const m = await memApi();
      memorySeries.push({ t_ms: Date.now() - startedAtAuto, bytes: m.bytes });
    } catch {
      /* 采样失败忽略 */
    }
  };
  for (;;) {
    const b = findButton(AUTOALL_BUTTON);
    if (b) b.click();
    let lastMem = 0;
    while (Date.now() < deadline && !finished()) {
      await sleep(1000);
      if (Date.now() - lastMem > 30_000) {
        lastMem = Date.now();
        await sampleMem();
      }
    }
    if (!loop || Date.now() >= deadline) {
      autoDriveMemorySeries = memorySeries;
      return finished() ? "finished" : "auto_timeout";
    }
    const restart = Array.from(document.querySelectorAll("button")).find((x) =>
      (x.innerText || "").includes("重新开局"),
    ) as HTMLButtonElement | undefined;
    restart?.click();
    await sleep(1500);
  }
}

function driveManual(steps: number, timeoutMs: number): Promise<string> {
  return new Promise((resolve) => {
    let clicks = 0;
    const finish = (why: string) => {
      window.removeEventListener("click", onClick, true);
      resolve(why);
    };
    const onClick = (ev: Event) => {
      const t = ev.target as HTMLElement | null;
      const b = findButton(AUTOSTEP_BUTTON);
      if (b && (t === b || b.contains(t))) {
        clicks += 1;
        if (clicks >= steps) {
          setTimeout(() => finish("manual_steps"), 800);
        }
      }
    };
    window.addEventListener("click", onClick, true);
    const poll = setInterval(() => {
      if (FINISHED_RE.test(document.body.innerText)) {
        clearInterval(poll);
        finish("finished");
      }
    }, 500);
    setTimeout(() => {
      clearInterval(poll);
      finish(clicks > 0 ? "manual_timeout" : "manual_no_input");
    }, timeoutMs);
  });
}

/**
 * SPA hash 路由下 query 可能位于 search 或 hash 内——两处合并读取。
 * client.start 的 PA08-4 压测入口（packsUrl/hexUrl）与本模块共用此口径。
 */
export function urlParam(key: string): string | null {
  const hashQuery = location.hash.split("?")[1];
  const hashParams = hashQuery ? new URLSearchParams(hashQuery) : null;
  return new URLSearchParams(location.search).get(key) ?? hashParams?.get(key) ?? null;
}

/**
 * 幂等入口：URL 带 autodrive 参数时启动自驱动；由 perfProbe 开启时调用
 * （保证 __nonamePerf 已就位）。重复调用无副作用。
 */
export function startAutoDrive(): void {
  if (typeof window === "undefined") return;
  const w = window as unknown as { __nonameAutoDriveStarted?: boolean };
  if (w.__nonameAutoDriveStarted) return;
  const mode = urlParam("autodrive");
  if (!mode) return;
  w.__nonameAutoDriveStarted = true;
  const reportUrl = urlParam("report");
  const steps = Number(urlParam("steps") ?? 1500);
  const gapMs = Number(urlParam("gap") ?? 50);
  const timeoutMs = Number(urlParam("timeout") ?? 120_000);
  void (async () => {
    // 等待战斗页就绪（按钮出现且 perf 数据面可用）
    for (let k = 0; k < 150; k++) {
      const b = findButton(AUTOSTEP_BUTTON);
      if (b && perfApi() && !b.disabled) break;
      await sleep(200);
    }
    const api = perfApi();
    api?.reset();
    await sleep(300); // 空闲基线由外部脚本按需另采；此处直接开始驱动
    const startedAt = Date.now();
    startedAtAuto = startedAt;
    let endState = "unknown";
    if (mode === "click") endState = await driveClick(steps, gapMs);
    else if (mode === "auto") endState = await driveAuto();
    else if (mode === "manual") endState = await driveManual(steps, timeoutMs);
    await sleep(600);
    const data = collect();
    data.memory = await sampleMemory();
    data.memory_series = autoDriveMemorySeries;
    if (reportUrl) {
      await report(reportUrl, { ...data, end_state: endState, wall_ms: Date.now() - startedAt });
    }
    const mark = window as unknown as { __nonameAutoDriveDone?: string };
    mark.__nonameAutoDriveDone = endState;
  })();
}
