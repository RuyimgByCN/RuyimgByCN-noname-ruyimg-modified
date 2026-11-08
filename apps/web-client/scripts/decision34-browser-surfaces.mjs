// 裁决 34（2026-11-08）真实无头 Chrome 产品路径验收（CDP 驱动）：
// 真实构建产物 (apps/web-client/dist) + 真实原生 WASM Worker + BattleView + ExtensionSurfacePanel
// 全 5 个标签页交互、暗牌/eval/导出文件名门禁、真实存档/读档恢复与浏览器帧率测量。
//
// 命令：
//   node apps/web-client/scripts/decision34-browser-surfaces.mjs <outdir> [chrome-bin]

import { execFileSync, spawn } from "node:child_process";
import { createServer } from "node:http";
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { extname, join, resolve } from "node:path";

const outDir = resolve(process.argv[2] ?? ".");
const chromeBin =
  process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const webClientDir = resolve(new URL("..", import.meta.url).pathname);
const distDir = mkdtempSync(join(tmpdir(), "decision34-dist-"));
execFileSync("pnpm", ["--dir", webClientDir, "exec", "vite", "build", "--outDir", distDir], {
  stdio: "ignore",
});

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
};

const server = createServer((req, res) => {
  const urlPath = (req.url ?? "/").split("?")[0] || "/";
  let filePath = join(distDir, urlPath === "/" ? "index.html" : urlPath);
  if (!existsSync(filePath) || statSync(filePath).isDirectory()) {
    filePath = join(distDir, "index.html");
  }
  try {
    const body = readFileSync(filePath);
    res.writeHead(200, {
      "Content-Type": MIME[extname(filePath)] ?? "application/octet-stream",
      "Cache-Control": "no-store",
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    });
    res.end(body);
  } catch (err) {
    res.writeHead(500);
    res.end(String(err));
  }
});

await new Promise((r) => server.listen(0, "127.0.0.1", r));
const httpPort = server.address().port;
const baseUrl = `http://127.0.0.1:${httpPort}/`;

const cdpPort = 9347;
const udd = mkdtempSync(join(tmpdir(), "decision34-cdp-"));
const chrome = spawn(
  chromeBin,
  [
    "--headless=new",
    "--disable-gpu",
    "--no-sandbox",
    "--autoplay-policy=no-user-gesture-required",
    `--user-data-dir=${udd}`,
    `--remote-debugging-port=${cdpPort}`,
    "--window-size=1440,960",
    "about:blank",
  ],
  { stdio: "ignore" },
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function cdpTarget() {
  for (let i = 0; i < 50; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${cdpPort}/json/list`);
      const list = await res.json();
      const page = list.find((t) => t.type === "page");
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(150);
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

try {
  const wsUrl = await cdpTarget();
  const ws = new WebSocket(wsUrl);
  await new Promise((r, j) => {
    ws.onopen = r;
    ws.onerror = j;
  });
  ws.onmessage = (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) {
      const { resolve: res, reject: rej } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? rej(new Error(JSON.stringify(msg.error))) : res(msg.result);
      return;
    }
    if (msg.method === "Runtime.exceptionThrown") {
      consoleErrors.push(`exception: ${msg.params?.exceptionDetails?.text ?? "?"}`);
    }
    if (msg.method === "Runtime.consoleAPICalled" && msg.params?.type === "error") {
      consoleErrors.push(
        `console.error: ${(msg.params.args ?? []).map((a) => a.value ?? a.description ?? "?").join(" ")}`,
      );
    }
  };

  await send(ws, "Runtime.enable");
  await send(ws, "Page.enable");
  await send(ws, "Page.navigate", { url: baseUrl });

  async function evalJs(expression) {
    const r = await send(ws, "Runtime.evaluate", {
      expression,
      returnByValue: true,
      awaitPromise: true,
    });
    if (r.exceptionDetails) {
      throw new Error(
        r.exceptionDetails.exception?.description ?? r.exceptionDetails.text ?? "evalJs error",
      );
    }
    return r.result?.value;
  }

  for (let i = 0; i < 40; i++) {
    await sleep(200);
    const hasStart = await evalJs(`Array.from(document.querySelectorAll('button,a')).some(el => (el.innerText||'').includes('本地开局'))`);
    if (hasStart) break;
  }

  // 1. 开启 skipAnims 并点击「本地开局」进入 BattleView
  await evalJs(`localStorage.setItem("noname-next-skip-anims", "1")`);
  const clickedStart = await evalJs(`(() => {
    const el = Array.from(document.querySelectorAll('button,a')).find(el => (el.innerText||'').includes('本地开局'));
    if (!el) return false;
    el.click();
    return true;
  })()`);
  if (!clickedStart) throw new Error(`未找到本地开局按钮; errors=${JSON.stringify(consoleErrors)}`);

  for (let i = 0; i < 50; i++) {
    await sleep(200);
    const ready = await evalJs(`Boolean(
      document.querySelector('[data-testid="extension-surface-panel"]') &&
      /rev\\s+\\d+/.test(document.body.innerText)
    )`);
    if (ready) break;
  }

  const panelRect = await evalJs(`(() => {
    const el = document.querySelector('[data-testid="extension-surface-panel"]');
    if (!el) return null;
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    return { width: Math.round(r.width), height: Math.round(r.height), display: cs.display, border: cs.borderTopStyle };
  })()`);
  if (!panelRect || panelRect.width <= 0 || panelRect.height <= 0) {
    const diag = await evalJs(`({ href: location.href, text: document.body.innerText.slice(0, 500) })`);
    throw new Error(`ExtensionSurfacePanel 未渲染或尺寸为 0: ${JSON.stringify({ panelRect, diag, consoleErrors })}`);
  }

  // 2. 驱动 Tab 1 (战局 HUD)
  const hudResult = await evalJs(`(async () => {
    const click = (tid) => {
      const btn = document.querySelector('[data-testid="' + tid + '"]');
      if (!btn) throw new Error('missing ' + tid);
      btn.click();
    };
    click('btn-longxue-plus');
    click('btn-l20-mark-plus');
    click('btn-thunder-toggle');
    click('btn-cycle-decade');
    click('btn-cycle-shousha');
    click('btn-cycle-ruzhen');
    click('btn-cycle-qhly');
    click('btn-switch-skin');
    click('btn-fool-emotion');
    await new Promise(r => setTimeout(r, 50));
    const text = (tid) => document.querySelector('[data-testid="' + tid + '"]')?.textContent?.trim() ?? '';
    return {
      daoxin: text('longxue-daoxin'),
      thresholds: text('longxue-thresholds'),
      unlockedSkills: text('longxue-unlocked-skills'),
      l20MarkBadges: text('l20-mark-badges'),
      badgeDecade: text('badge-decade'),
      badgeShousha: text('badge-shousha'),
      badgeRuzhen: text('badge-ruzhen'),
      badgeQhly: text('badge-qhly'),
      notice: text('surface-notice'),
    };
  })()`);

  // 3. 驱动 Tab 2 (单机战役)
  const campaignResult = await evalJs(`(async () => {
    const click = (tid) => {
      const btn = document.querySelector('[data-testid="' + tid + '"]');
      if (!btn) throw new Error('missing ' + tid);
      btn.click();
    };
    click('tab-campaign');
    await new Promise(r => setTimeout(r, 50));
    click('btn-boss-advance');
    click('btn-boss-mode');
    click('btn-u08-tafang-wave');
    click('btn-taixu-advance');
    await new Promise(r => setTimeout(r, 30));
    click('btn-taixu-buy-blessing');
    click('btn-taixu-bind-pet');
    click('btn-martial-draw');
    click('btn-martial-win');
    click('btn-doomsday-advance');
    await new Promise(r => setTimeout(r, 50));
    const text = (tid) => document.querySelector('[data-testid="' + tid + '"]')?.textContent?.trim() ?? '';
    return {
      bossName: text('boss-name'),
      bossHp: text('boss-hp'),
      bossMode: text('boss-mode'),
      tafangWave: text('u08-tafang-wave'),
      taixuNode: text('taixu-node'),
      taixuGold: text('taixu-gold'),
      taixuPet: text('taixu-pet'),
      taixuBlessingsCount: text('taixu-blessings-count'),
      martialReplacement: text('martial-replacement'),
      doomsdayWave: text('doomsday-wave'),
    };
  })()`);

  // 4. 驱动 Tab 3 (金币假人 + 天气粒子层)
  const coinResult = await evalJs(`(async () => {
    const click = (tid) => {
      const btn = document.querySelector('[data-testid="' + tid + '"]');
      if (!btn) throw new Error('missing ' + tid);
      btn.click();
    };
    click('tab-coin');
    await new Promise(r => setTimeout(r, 50));
    click('btn-coin-snow');
    await new Promise(r => setTimeout(r, 50));
    const text = (tid) => document.querySelector('[data-testid="' + tid + '"]')?.textContent?.trim() ?? '';
    const weatherOverlay = document.querySelector('[data-testid="coin-weather-overlay"]');
    return {
      balance: text('coin-balance'),
      honorTitle: text('coin-honor-title'),
      snow: text('coin-snow'),
      weatherOverlayRendered: Boolean(weatherOverlay),
      weatherAttr: weatherOverlay?.getAttribute('data-weather') ?? null,
    };
  })()`);

  // 5. 驱动 Tab 4 (全能搜索：公开检索 cangchizhibi + eval 门禁拦截)
  const omnisearchResult = await evalJs(`(async () => {
    const click = (tid) => {
      const btn = document.querySelector('[data-testid="' + tid + '"]');
      if (!btn) throw new Error('missing ' + tid);
      btn.click();
    };
    click('tab-omnisearch');
    await new Promise(r => setTimeout(r, 50));
    const input = document.querySelector('[data-testid="input-omnisearch"]');
    input.value = 'cangchizhibi';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    click('btn-omnisearch-run');
    await new Promise(r => setTimeout(r, 50));
    const hitText = document.querySelector('[data-testid="omnisearch-results"]')?.textContent?.trim() ?? '';

    input.value = 'eval:alert(1)';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    click('btn-omnisearch-run');
    await new Promise(r => setTimeout(r, 50));
    const blockedText = document.querySelector('[data-testid="omnisearch-blocked"]')?.textContent?.trim() ?? '';
    return { hitText, blockedText };
  })()`);

  // 6. 驱动 Tab 5 (工作台与适配：导出文件名安全校验)
  const workbenchResult = await evalJs(`(async () => {
    const click = (tid) => {
      const btn = document.querySelector('[data-testid="' + tid + '"]');
      if (!btn) throw new Error('missing ' + tid);
      btn.click();
    };
    click('tab-workbench');
    await new Promise(r => setTimeout(r, 50));
    const input = document.querySelector('[data-testid="input-export-filename"]');
    input.value = '../passwd';
    input.dispatchEvent(new Event('input', { bubbles: true }));
    click('btn-validate-export');
    await new Promise(r => setTimeout(r, 50));
    const rejectedMsg = document.querySelector('[data-testid="export-validation-msg"]')?.textContent?.trim() ?? '';
    return { rejectedMsg };
  })()`);

  // 7. 验证真实浏览器 BattleView 存档与读档（同时恢复原生引擎状态与扩展面板快照）
  const saveRestoreResult = await evalJs(`(async () => {
    const clickByText = (label) => {
      const btn = Array.from(document.querySelectorAll('button')).find(b => (b.innerText||'').trim() === label);
      if (!btn) throw new Error('missing button ' + label);
      btn.click();
    };
    const clickTid = (tid) => {
      const btn = document.querySelector('[data-testid="' + tid + '"]');
      if (!btn) throw new Error('missing ' + tid);
      btn.click();
    };
    clickTid('tab-hud');
    await new Promise(r => setTimeout(r, 50));
    const beforeSaveDaoxin = document.querySelector('[data-testid="longxue-daoxin"]')?.textContent?.trim();
    const beforeSaveDecade = document.querySelector('[data-testid="badge-decade"]')?.textContent?.trim();

    // 点击「存档」
    clickByText('存档');
    for (let i = 0; i < 30; i++) {
      await new Promise(r => setTimeout(r, 100));
      if ((document.querySelector('.toolbar .status')?.textContent || '').includes('已存档')) break;
    }
    const saveStatus = document.querySelector('.toolbar .status')?.textContent?.trim() ?? '';

    // 存档后继续修改状态（道心 50 -> 80，十周年风格 shousha -> xinzhan）
    clickTid('btn-longxue-plus');
    clickTid('btn-cycle-decade');
    await new Promise(r => setTimeout(r, 50));
    const mutatedDaoxin = document.querySelector('[data-testid="longxue-daoxin"]')?.textContent?.trim();
    const mutatedDecade = document.querySelector('[data-testid="badge-decade"]')?.textContent?.trim();

    // 点击「读档」
    clickByText('读档');
    for (let i = 0; i < 30; i++) {
      await new Promise(r => setTimeout(r, 100));
      const d = document.querySelector('[data-testid="longxue-daoxin"]')?.textContent?.trim();
      if (d === beforeSaveDaoxin) break;
    }
    const restoredDaoxin = document.querySelector('[data-testid="longxue-daoxin"]')?.textContent?.trim();
    const restoredDecade = document.querySelector('[data-testid="badge-decade"]')?.textContent?.trim();
    const weatherStillActive = Boolean(document.querySelector('[data-testid="coin-weather-overlay"]'));
    return {
      beforeSaveDaoxin,
      beforeSaveDecade,
      saveStatus,
      mutatedDaoxin,
      mutatedDecade,
      restoredDaoxin,
      restoredDecade,
      weatherStillActive,
    };
  })()`);

  // 8. 真实浏览器帧率测量（60 帧 requestAnimationFrame，天气粒子层 + 扩展面板 + BattleBoard 同屏激活）
  const frameStats = await evalJs(`(async () => {
    const deltas = [];
    let last = performance.now();
    await new Promise((resolve) => {
      function step(now) {
        deltas.push(Math.round((now - last) * 100) / 100);
        last = now;
        if (deltas.length >= 60) {
          resolve();
        } else {
          requestAnimationFrame(step);
        }
      }
      requestAnimationFrame((now) => {
        last = now;
        requestAnimationFrame(step);
      });
    });
    const sorted = [...deltas].sort((a, b) => a - b);
    const pick = (p) => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))];
    const avg = sorted.reduce((a, b) => a + b, 0) / sorted.length;
    return {
      n: sorted.length,
      p50_ms: pick(0.5),
      p95_ms: pick(0.95),
      p99_ms: pick(0.99),
      max_ms: sorted[sorted.length - 1],
      approx_fps: Math.round((1000 / Math.max(0.1, avg)) * 10) / 10,
    };
  })()`);

  // 9. 验证点击「重新开局」干净重置扩展表现面（消除上一局残留），再次「读档」仍可完整恢复
  const restartAndReloadResult = await evalJs(`(async () => {
    const clickByText = (label) => {
      const btn = Array.from(document.querySelectorAll('button')).find(b => (b.innerText||'').trim() === label);
      if (!btn) throw new Error('missing button ' + label);
      btn.click();
    };
    clickByText('重新开局');
    for (let i = 0; i < 30; i++) {
      await new Promise(r => setTimeout(r, 100));
      const d = document.querySelector('[data-testid="longxue-daoxin"]')?.textContent?.trim();
      if (d === '20') break;
    }
    const afterRestartDaoxin = document.querySelector('[data-testid="longxue-daoxin"]')?.textContent?.trim();
    const afterRestartWeather = Boolean(document.querySelector('[data-testid="coin-weather-overlay"]'));

    clickByText('读档');
    for (let i = 0; i < 30; i++) {
      await new Promise(r => setTimeout(r, 100));
      const d = document.querySelector('[data-testid="longxue-daoxin"]')?.textContent?.trim();
      if (d === '50') break;
    }
    const afterReloadDaoxin = document.querySelector('[data-testid="longxue-daoxin"]')?.textContent?.trim();
    const afterReloadWeather = Boolean(document.querySelector('[data-testid="coin-weather-overlay"]'));
    return {
      afterRestartDaoxin,
      afterRestartWeather,
      afterReloadDaoxin,
      afterReloadWeather,
    };
  })()`);

  const shot = await send(ws, "Page.captureScreenshot", { format: "png" });
  const pngPath = join(outDir, "decision34-browser-surfaces.png");
  writeFileSync(pngPath, Buffer.from(shot.data, "base64"));

  const report = {
    passed:
      consoleErrors.length === 0 &&
      panelRect.width > 0 &&
      hudResult.daoxin === "50" &&
      hudResult.unlockedSkills === "mbcmqingzheng/mbcmjiushi" &&
      hudResult.l20MarkBadges.includes("龙印 ×1") &&
      campaignResult.bossHp === "∞" &&
      campaignResult.tafangWave === "#2/5" &&
      campaignResult.taixuGold === "90" &&
      coinResult.weatherOverlayRendered === true &&
      coinResult.honorTitle === "初入茅庐" &&
      omnisearchResult.hitText.includes("苍螭之璧") &&
      omnisearchResult.blockedText.includes("error:eval_refused") &&
      workbenchResult.rejectedMsg.includes("unsafe export filename") &&
      saveRestoreResult.restoredDaoxin === "50" &&
      saveRestoreResult.restoredDecade === "十周年:shousha" &&
      saveRestoreResult.weatherStillActive === true &&
      restartAndReloadResult.afterRestartDaoxin === "20" &&
      restartAndReloadResult.afterRestartWeather === false &&
      restartAndReloadResult.afterReloadDaoxin === "50" &&
      restartAndReloadResult.afterReloadWeather === true,
    panelRect,
    hudResult,
    campaignResult,
    coinResult,
    omnisearchResult,
    workbenchResult,
    saveRestoreResult,
    restartAndReloadResult,
    frameStats,
    consoleErrors,
    screenshot: pngPath,
  };

  const reportPath = join(outDir, "decision34-browser-surfaces-report.json");
  writeFileSync(reportPath, JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report, null, 2));
  ws.close();
  if (!report.passed) {
    process.exitCode = 1;
  }
} finally {
  try {
    chrome.kill("SIGKILL");
  } catch {}
  try {
    server.close();
  } catch {}
  try {
    rmSync(udd, { recursive: true, force: true });
  } catch {}
  try {
    rmSync(distDir, { recursive: true, force: true });
  } catch {}
}
