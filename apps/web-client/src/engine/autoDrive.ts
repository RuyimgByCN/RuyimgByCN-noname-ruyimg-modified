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
  };
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
  const b = findButton(AUTOALL_BUTTON);
  if (b) b.click();
  const deadline = Date.now() + 600_000;
  while (Date.now() < deadline) {
    await sleep(1000);
    if (FINISHED_RE.test(document.body.innerText)) return "finished";
  }
  return "auto_timeout";
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
    let endState = "unknown";
    if (mode === "click") endState = await driveClick(steps, gapMs);
    else if (mode === "auto") endState = await driveAuto();
    else if (mode === "manual") endState = await driveManual(steps, timeoutMs);
    await sleep(600);
    const data = collect();
    if (reportUrl) {
      await report(reportUrl, { ...data, end_state: endState, wall_ms: Date.now() - startedAt });
    }
    const mark = window as unknown as { __nonameAutoDriveDone?: string };
    mark.__nonameAutoDriveDone = endState;
  })();
}
