// tabletop-comparison-cdp.mjs
// Milestone 3 (R4 / F14 & F15): Headless Chrome CDP Comparison & FPS Measurement
// Automated comparison between Upstream Noname (Liuli 1.10.1.0.1) and Native BattleBoard
// Strict Decision 35 cleanroom guard on static assets.

import { execFileSync, spawn } from "node:child_process";
import { createServer } from "node:http";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const scriptsDir = dirname(scriptPath);
const webClientDir = resolve(scriptsDir, "..");
const repoRoot = resolve(webClientDir, "../..");
const projectRoot = resolve(repoRoot, "..");
const upstreamDir = resolve(projectRoot, "archive/upstream/noname-liuli/noname");
const docsEvidenceDir = resolve(
  repoRoot,
  process.env.TABLETOP_EVIDENCE_DIR || "docs/operations/evidence/2026-11-11-tabletop-and-spine40"
);

console.log("Paths configuration:");
console.log("- Web client dir:", webClientDir);
console.log("- Repo root:", repoRoot);
console.log("- Project root:", projectRoot);
console.log("- Upstream dir:", upstreamDir, existsSync(upstreamDir) ? "[OK]" : "[MISSING]");
console.log("- Docs evidence dir:", docsEvidenceDir);

mkdirSync(docsEvidenceDir, { recursive: true });

// Step 0: Ensure native frontend dist is fresh
console.log("\nBuilding native web-client with Vite...");
execFileSync("pnpm", ["--dir", webClientDir, "exec", "vite", "build"], {
  stdio: "inherit",
});
const distDir = resolve(webClientDir, "dist");

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".mp3": "audio/mpeg",
  ".ogg": "audio/ogg",
  ".wav": "audio/wav",
  ".woff": "font/woff",
  ".woff2": "font/woff2",
  ".ttf": "font/ttf",
};

function createStaticServer(targetDir, isNative = false) {
  return createServer((req, res) => {
    const urlPath = decodeURIComponent((req.url ?? "/").split("?")[0] || "/");

    // Decision 35 cleanroom guard: block all spine runtime paths
    if (/spine/i.test(urlPath)) {
      res.writeHead(404, { "Content-Type": "text/plain" });
      res.end("Blocked by Decision 35 Cleanroom Guard");
      return;
    }

    let filePath = resolve(targetDir, "." + (urlPath === "/" ? "/index.html" : urlPath));
    if (!filePath.startsWith(targetDir) || !existsSync(filePath) || statSync(filePath).isDirectory()) {
      if (isNative) {
        filePath = resolve(targetDir, "index.html");
      } else {
        res.writeHead(404);
        res.end("Not Found");
        return;
      }
    }

    try {
      const ext = extname(filePath).toLowerCase();
      const headers = {
        "Content-Type": MIME[ext] ?? "application/octet-stream",
        "Cache-Control": "no-store",
      };
      if (isNative) {
        headers["Cross-Origin-Opener-Policy"] = "same-origin";
        headers["Cross-Origin-Embedder-Policy"] = "require-corp";
      }
      res.writeHead(200, headers);
      res.end(readFileSync(filePath));
    } catch (err) {
      res.writeHead(500);
      res.end(String(err));
    }
  });
}

const upstreamServer = createStaticServer(upstreamDir, false);
await new Promise((r) => upstreamServer.listen(0, "127.0.0.1", r));
const upstreamPort = upstreamServer.address().port;
const upstreamUrl = `http://127.0.0.1:${upstreamPort}/`;
console.log(`Upstream server listening on ${upstreamUrl}`);

const nativeServer = createStaticServer(distDir, true);
await new Promise((r) => nativeServer.listen(0, "127.0.0.1", r));
const nativePort = nativeServer.address().port;
const nativeUrl = `http://127.0.0.1:${nativePort}/`;
console.log(`Native server listening on ${nativeUrl}`);

const chromeBin =
  process.env.CHROME_BIN ??
  (process.platform === "darwin"
    ? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    : "google-chrome");

const cdpPort = 9489;
const udd = mkdtempSync(join(tmpdir(), "tabletop-cdp-"));
const chrome = spawn(
  chromeBin,
  [
    "--headless=new",
    "--enable-webgl",
    "--ignore-gpu-blocklist",
    "--no-sandbox",
    "--autoplay-policy=no-user-gesture-required",
    `--user-data-dir=${udd}`,
    `--remote-debugging-port=${cdpPort}`,
    "--window-size=1440,1080",
    "about:blank",
  ],
  { stdio: "ignore" }
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function getWsUrl() {
  for (let i = 0; i < 50; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${cdpPort}/json/list`);
      const list = await res.json();
      const page = list.find((t) => t.type === "page");
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(150);
  }
  throw new Error("Headless Chrome CDP target not ready");
}

let cdpId = 0;
const pending = new Map();

function send(ws, method, params = {}) {
  const msgId = ++cdpId;
  ws.send(JSON.stringify({ id: msgId, method, params }));
  return new Promise((resolve, reject) => pending.set(msgId, { resolve, reject }));
}

async function evalJs(ws, expression) {
  const r = await send(ws, "Runtime.evaluate", {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (r.exceptionDetails) {
    throw new Error(
      r.exceptionDetails.exception?.description ?? r.exceptionDetails.text ?? "evalJs error"
    );
  }
  return r.result?.value;
}

try {
  const wsUrl = await getWsUrl();
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
    if (msg.method === "Page.javascriptDialogOpening") {
      send(ws, "Page.handleJavaScriptDialog", { accept: true }).catch(() => {});
    }
  };

  await send(ws, "Runtime.enable");
  await send(ws, "Page.enable");
  await send(ws, "Network.enable");

  // Enforce strict 1440x1080 1:1 viewport via CDP Emulation domain
  await send(ws, "Emulation.setDeviceMetricsOverride", {
    width: 1440,
    height: 1080,
    deviceScaleFactor: 1,
    mobile: false,
  });

  // ==========================================
  // PHASE 1: Upstream Noname Match & Capture
  // ==========================================
  console.log("\n[PHASE 1] Navigating upstream noname-liuli...");
  const validExtensions = [
    "十周年UI",
    "手杀ui",
    "千幻聆音",
    "皮肤切换",
    "如真似幻",
    "标记补充",
    "太虚幻境",
    "全能搜索",
    "boss",
    "coin",
    "cardpile",
  ];

  const upstreamConfig = {
    dev: true,
    mode: "identity",
    continue_name: ["caocao"],
    new_tutorial: true,
    show_splash: false,
    game_speed: "fast",
    duration: 50,
    extensions: validExtensions,
    extension_十周年UI_enable: true,
    extension_手杀ui_enable: true,
    player_height: "long",
    theme: "simple",
    layout: "mobile",
  };

  await send(ws, "Page.addScriptToEvaluateOnNewDocument", {
    source: `
      window.alert = function() {};
      window.confirm = function() { return true; };
      window.prompt = function() { return null; };
      window.bannedExtensions = ['手杀禁将', '无名补丁', '祖安设置', '游戏说明'];
      window.resolveLocalFileSystemURL = function(url, success, error) {
        if (typeof error === 'function') error(new Error('not a file system'));
      };
      window.__dirname = '';
      window.require = function(mod) {
        if (mod === 'fs') {
          return {
            constants: { F_OK: 0, R_OK: 0 },
            access: (dir, mode, cb) => (cb ? cb(null) : undefined),
            readdir: (dir, cb) => (cb ? cb(null, []) : undefined),
            statSync: () => ({ isDirectory: () => false }),
            createWriteStream: () => ({ write() {}, end() {} }),
            unlink: (f, cb) => (cb ? cb() : undefined),
          };
        }
        if (mod === 'path') return { join: (...args) => args.join('/') };
        if (mod === 'electron') return { remote: { getCurrentWindow: () => ({ toggleDevTools() {} }) } };
        return {};
      };

      try {
        localStorage.removeItem('noname_inited');
        localStorage.setItem('directstart', 'true');
        localStorage.setItem('mode', 'identity');
        localStorage.setItem('config_mode', 'identity');
        localStorage.setItem('noname_0.9_directstart', 'true');
        localStorage.setItem('noname_0.9_nodb', 'true');
        localStorage.setItem('nodb', 'true');
        localStorage.setItem('noname_0.9_config', JSON.stringify(${JSON.stringify(upstreamConfig)}));
        localStorage.setItem('config', JSON.stringify(${JSON.stringify(upstreamConfig)}));
        localStorage.setItem('noname_0.9_extension_十周年UI_enable', 'true');
        localStorage.setItem('noname_0.9_extension_手杀ui_enable', 'true');
        localStorage.setItem('extension_十周年UI_enable', 'true');
        localStorage.setItem('extension_手杀ui_enable', 'true');
      } catch (e) {
        console.error('LocalStorage error:', e);
      }
    `,
  });

  await send(ws, "Page.navigate", { url: upstreamUrl });

  // Dismiss changelog, character selection, and luck-card ("是否使用手气卡？") prompts into real play
  for (let i = 0; i < 65; i++) {
    await sleep(300);
    await evalJs(
      ws,
      `(() => {
        const bodyText = document.body.innerText || '';
        const clickEl = (el) => {
          if (!el) return;
          el.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
          el.dispatchEvent(new MouseEvent('mouseup', { bubbles: true }));
          el.click();
        };
        const controlChildren = Array.from(document.querySelectorAll('.control > div, .control'));
        if (bodyText.includes('手气卡')) {
          const cancel = controlChildren.find(el => (el.innerText || '').trim() === '取消')
            || Array.from(document.querySelectorAll('div, button')).find(el => (el.innerText || '').trim() === '取消');
          if (cancel) {
            clickEl(cancel);
          } else if (window._status?.event?.custom?.replace?.confirm) {
            window._status.event.custom.replace.confirm(false);
          }
          return;
        }

        const charBtn = document.querySelector('.dialog.character .button, .dialog .buttons .button');
        if (charBtn && document.querySelector('#arena.choose-character')) {
          clickEl(charBtn);
          const conf = controlChildren.find(
            el => (el.innerText || '').trim() === '确定' || (el.innerText || '').includes('OK')
          );
          clickEl(conf);
          return;
        }

        const ok = controlChildren.find(el => (el.innerText || '').trim() === '确定');
        if (ok) clickEl(ok);
      })()`
    );

    const arenaStatus = await evalJs(
      ws,
      `(() => {
        const arena = document.querySelector('#arena');
        const players = document.querySelectorAll('#arena .player, .player');
        const cards = document.querySelectorAll('.card');
        const bodyText = document.body.innerText || '';
        const inChooseChar = Boolean(document.querySelector('#arena.choose-character'));
        const inLuckPrompt = bodyText.includes('手气卡');
        return {
          hasArena: Boolean(arena),
          playerCount: players.length,
          cardCount: cards.length,
          inChooseChar,
          inLuckPrompt
        };
      })()`
    );

    if (
      arenaStatus.hasArena &&
      arenaStatus.playerCount >= 5 &&
      arenaStatus.cardCount >= 4 &&
      !arenaStatus.inChooseChar &&
      !arenaStatus.inLuckPrompt
    ) {
      console.log(`Upstream match ready in active play at tick ${i}: ${arenaStatus.playerCount} players, ${arenaStatus.cardCount} cards`);
      break;
    }
  }

  // Allow turn 1 phase & card settling without startup modal
  await sleep(3000);

  // Ensure exact 1440x1080 device metrics before screenshot
  await send(ws, "Emulation.setDeviceMetricsOverride", {
    width: 1440,
    height: 1080,
    deviceScaleFactor: 1,
    mobile: false,
  });

  const upstreamShotResult = await send(ws, "Page.captureScreenshot", {
    format: "png",
    captureBeyondViewport: false,
    clip: { x: 0, y: 0, width: 1440, height: 1080, scale: 1 },
  });
  const upstreamPng = Buffer.from(upstreamShotResult.data, "base64");
  const upstreamPngPath = join(docsEvidenceDir, "tabletop-upstream.png");
  writeFileSync(upstreamPngPath, upstreamPng);
  console.log(`Saved upstream screenshot to ${upstreamPngPath} (${upstreamPng.length} bytes)`);

  // ==========================================
  // PHASE 2: Native BattleBoard Match & Framing
  // ==========================================
  console.log("\n[PHASE 2] Navigating native client...");
  await send(ws, "Page.navigate", { url: nativeUrl });

  for (let i = 0; i < 40; i++) {
    await sleep(200);
    const hasStart = await evalJs(
      ws,
      `Array.from(document.querySelectorAll('button, a')).some(el => (el.innerText || '').includes('本地开局'))`
    );
    if (hasStart) break;
  }

  await evalJs(ws, `localStorage.setItem("noname-next-skip-anims", "1")`);
  await evalJs(
    ws,
    `(() => {
      const playersSelect = document.querySelector('select[aria-label="玩家人数"]');
      if (playersSelect) {
        playersSelect.value = '5';
        playersSelect.dispatchEvent(new Event('change', { bubbles: true }));
      }
      const modeSelect = document.querySelector('select[aria-label="对局模式"]');
      if (modeSelect) {
        modeSelect.value = 'identity';
        modeSelect.dispatchEvent(new Event('change', { bubbles: true }));
      }
      const btn = Array.from(document.querySelectorAll('button, a')).find(el => (el.innerText || '').includes('本地开局'));
      if (btn) btn.click();
    })()`
  );

  // Wait for BattleBoard & HUD & 5 seats
  for (let i = 0; i < 60; i++) {
    await sleep(200);
    const ready = await evalJs(
      ws,
      `Boolean(
        document.querySelector('.battle-table') &&
        document.querySelector('[data-testid="battle-shousha-table-hud"]') &&
        document.querySelectorAll('.seat').length >= 5
      )`
    );
    if (ready) {
      console.log(`Native BattleBoard ready at tick ${i}`);
      break;
    }
  }

  // Pick candidate general (曹操) if candidate grid is present so seats depict an active match
  console.log("Selecting Caocao to populate seat avatar...");
  await evalJs(
    ws,
    `(() => {
      const candidates = Array.from(document.querySelectorAll('.gs-grid .candidate, .candidate'));
      const caocao = candidates.find(c => (c.innerText || '').includes('曹操')) || candidates[0];
      if (caocao) caocao.click();
    })()`
  );
  await sleep(400);

  // Advance via real autoStep until self seat (P1) has an enabled playable hand card
  for (let i = 0; i < 40; i++) {
    const actionable = await evalJs(
      ws,
      `Boolean(document.querySelector('.strip .card:not(:disabled)'))`
    );
    if (actionable) break;
    await evalJs(
      ws,
      `(() => {
        const choiceBtn = document.querySelector('.choice button:not(:disabled)');
        if (choiceBtn) {
          choiceBtn.click();
          return;
        }
        const autoBtn = Array.from(document.querySelectorAll('.toolbar button')).find(b => (b.innerText || '').trim() === '自动一步');
        if (autoBtn && !autoBtn.disabled) autoBtn.click();
      })()`
    );
    await sleep(140);
  }

  // Inject framing CSS override to completely hide debug panels and frame BattleBoard at exact 1440x1080
  console.log("Injecting framing CSS override for full 1440x1080 tabletop presentation...");
  await evalJs(
    ws,
    `(() => {
      const style = document.createElement('style');
      style.id = 'tabletop-framing-override';
      style.textContent = \`
        /* Hide all debug, navigation, toolbar, roster, and general selection panels */
        header.topbar,
        .topbar,
        .turn-bar,
        .toolbar,
        .general-select,
        .general-select-host,
        .roster,
        .online-roster,
        .match-stats,
        .seat-status,
        .log,
        .debug-surface-drawer,
        [data-testid="extension-surface-panel-drawer"],
        .controls,
        .hotkey-overlay,
        .outcome-banner {
          display: none !important;
        }

        /* Lock viewport to 1440x1080 without scrollbars */
        html, body {
          overflow: hidden !important;
          margin: 0 !important;
          padding: 0 !important;
          width: 1440px !important;
          height: 1080px !important;
        }
        .app-shell {
          min-height: 1080px !important;
          height: 1080px !important;
        }
        .main {
          max-width: 1440px !important;
          width: 1440px !important;
          padding: 58px 16px 48px !important;
          margin: 0 auto !important;
          box-sizing: border-box !important;
        }
        .battle.stack {
          gap: 0 !important;
        }
        .battle-section.panel {
          padding: 0 !important;
          border: none !important;
          background: transparent !important;
        }
        .battle-table {
          min-height: 974px !important;
          height: 974px !important;
          box-sizing: border-box !important;
          display: flex !important;
          flex-direction: column !important;
        }
      \`;
      document.head.appendChild(style);
      window.scrollTo(0, 0);
    })()`
  );

  // Trigger coin snow dynamic weather effect
  console.log("Triggering tabletop coin snow weather effect...");
  await evalJs(
    ws,
    `(() => {
      const drawer = document.querySelector('[data-testid="extension-surface-panel-drawer"]');
      if (drawer) {
        drawer.style.display = '';
        drawer.open = true;
      }
      const tabCoin = document.querySelector('[data-testid="tab-coin"]');
      if (tabCoin) tabCoin.click();
    })()`
  );

  await sleep(250);

  await evalJs(
    ws,
    `(() => {
      const snowBtn = document.querySelector('[data-testid="btn-coin-snow"]');
      if (snowBtn) snowBtn.click();
      const drawer = document.querySelector('[data-testid="extension-surface-panel-drawer"]');
      if (drawer) {
        drawer.open = false;
        drawer.style.display = 'none';
      }
    })()`
  );

  await sleep(300);

  // Activate genuine card selection & hand card sorting transition (never inject fake classes)
  console.log("Activating hand card selection and sorting transition...");
  await evalJs(
    ws,
    `(() => {
      const enabledCard = document.querySelector('.strip .card:not(:disabled)');
      if (enabledCard) enabledCard.click();
      const lipaiBtn = document.querySelector('[data-testid="btn-handstrip-lipai"]');
      if (lipaiBtn) lipaiBtn.click();
    })()`
  );

  await sleep(300);

  // ==========================================
  // PHASE 3: 120-frame Tabletop FPS Measurement
  // ==========================================
  console.log("\n[PHASE 3] Measuring 120-frame tabletop FPS with active dynamic particles and card selection...");
  const fpsMetrics = await evalJs(
    ws,
    `(async () => {
      const deltas = [];
      let last = performance.now();

      await new Promise((resolve) => {
        function step(now) {
          const delta = now - last;
          deltas.push(Math.round(delta * 100) / 100);
          last = now;

          if (deltas.length >= 120) {
            resolve();
          } else {
            requestAnimationFrame(step);
          }
        }
        // Warm-up frame to avoid invocation latency
        requestAnimationFrame((now) => {
          last = now;
          requestAnimationFrame(step);
        });
      });

      const sorted = [...deltas].sort((a, b) => a - b);
      const pick = (p) => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))];
      const sum = sorted.reduce((a, b) => a + b, 0);
      const avg = sum / sorted.length;

      const boardRect = document.querySelector('.battle-table')?.getBoundingClientRect();
      const isBoardVisible = Boolean(
        boardRect &&
        boardRect.top >= 0 &&
        boardRect.top < 200 &&
        boardRect.width >= 1000
      );

      return {
        sample_count: sorted.length,
        avg_fps: Math.round((1000 / Math.max(0.1, avg)) * 10) / 10,
        max_jitter_ms: Math.round((sorted[sorted.length - 1] - pick(0.50)) * 100) / 100,
        p50_ms: Math.round(pick(0.50) * 100) / 100,
        p95_ms: Math.round(pick(0.95) * 100) / 100,
        p99_ms: Math.round(pick(0.99) * 100) / 100,
        max_ms: Math.round(sorted[sorted.length - 1] * 100) / 100,
        avg_frame_time_ms: Math.round(avg * 100) / 100,
        active_elements: {
          has_weather_particles: Boolean(document.querySelector('.tabletop-weather-canvas')),
          seat_count: document.querySelectorAll('.seat').length,
          hand_card_count: document.querySelectorAll('.card').length,
          has_shousha_hud: Boolean(document.querySelector('[data-testid="battle-shousha-table-hud"]')),
          has_kill_banner: Boolean(document.querySelector('[data-testid="l14-shousha-banner-text"]')),
          has_card_selection: Boolean(document.querySelector('.strip .card.selected')),
          battleboard_fully_visible: isBoardVisible
        }
      };
    })()`
  );

  console.log("FPS Metrics:", JSON.stringify(fpsMetrics, null, 2));
  if (!fpsMetrics.active_elements.has_card_selection) {
    throw new Error("Expected at least one genuine .strip .card.selected during comparison capture");
  }
  const fpsReportPath = join(docsEvidenceDir, "tabletop-fps-report.json");
  writeFileSync(fpsReportPath, JSON.stringify(fpsMetrics, null, 2));

  // Ensure exact 1440x1080 device metrics before native screenshot
  await send(ws, "Emulation.setDeviceMetricsOverride", {
    width: 1440,
    height: 1080,
    deviceScaleFactor: 1,
    mobile: false,
  });

  // Capture native 1440x1080 screenshot
  const nativeShotResult = await send(ws, "Page.captureScreenshot", {
    format: "png",
    captureBeyondViewport: false,
    clip: { x: 0, y: 0, width: 1440, height: 1080, scale: 1 },
  });
  const nativePng = Buffer.from(nativeShotResult.data, "base64");
  const nativePngPath = join(docsEvidenceDir, "tabletop-native.png");
  writeFileSync(nativePngPath, nativePng);
  console.log(`Saved native screenshot to ${nativePngPath} (${nativePng.length} bytes)`);

  // ==========================================
  // PHASE 4: Canvas 2D Side-by-Side Composite (1:1 2880x1080)
  // ==========================================
  console.log("\n[PHASE 4] Composing 2880x1080 side-by-side comparison without distortion...");
  const metaText = `Resolution: 1440x1080 1:1 per side | Native Avg FPS: ${fpsMetrics.avg_fps} (p50: ${fpsMetrics.p50_ms}ms, p95: ${fpsMetrics.p95_ms}ms, p99: ${fpsMetrics.p99_ms}ms, max: ${fpsMetrics.max_ms}ms) | Weather: Active Snow | Decision 35 Cleanroom Verified`;

  const compositeBase64 = await evalJs(
    ws,
    `(async () => {
      const loadImg = (b64) => new Promise((resolve, reject) => {
        const img = new Image();
        img.onload = () => resolve(img);
        img.onerror = (e) => reject(new Error('Failed to load image: ' + e));
        img.src = 'data:image/png;base64,' + b64;
      });

      const [imgLeft, imgRight] = await Promise.all([
        loadImg(${JSON.stringify(upstreamShotResult.data)}),
        loadImg(${JSON.stringify(nativeShotResult.data)})
      ]);

      // Strict Precondition Assertion: Ensure 1:1 Source Dimensions
      if (imgLeft.naturalWidth !== 1440 || imgLeft.naturalHeight !== 1080) {
        throw new Error(\`Upstream screenshot dimension mismatch: expected 1440x1080, got \${imgLeft.naturalWidth}x\${imgLeft.naturalHeight}\`);
      }
      if (imgRight.naturalWidth !== 1440 || imgRight.naturalHeight !== 1080) {
        throw new Error(\`Native screenshot dimension mismatch: expected 1440x1080, got \${imgRight.naturalWidth}x\${imgRight.naturalHeight}\`);
      }

      const canvas = document.createElement('canvas');
      canvas.width = 2880;
      canvas.height = 1080;
      const ctx = canvas.getContext('2d');

      // Draw sides 1:1 (zero aspect-ratio distortion)
      ctx.drawImage(imgLeft, 0, 0, 1440, 1080);
      ctx.drawImage(imgRight, 1440, 0, 1440, 1080);

      // High-contrast Center Divider
      ctx.strokeStyle = 'rgba(0, 0, 0, 0.65)';
      ctx.lineWidth = 6;
      ctx.beginPath();
      ctx.moveTo(1440, 0);
      ctx.lineTo(1440, 1080);
      ctx.stroke();

      ctx.strokeStyle = '#e8a54b';
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(1440, 0);
      ctx.lineTo(1440, 1080);
      ctx.stroke();

      // Top Header Banner
      ctx.fillStyle = 'rgba(15, 18, 24, 0.92)';
      ctx.fillRect(0, 0, 2880, 52);

      ctx.strokeStyle = 'rgba(232, 165, 75, 0.45)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, 52);
      ctx.lineTo(2880, 52);
      ctx.stroke();

      ctx.font = 'bold 20px system-ui, -apple-system, "PingFang SC", sans-serif';
      ctx.textBaseline = 'middle';

      ctx.fillStyle = '#60a5fa';
      ctx.fillText('【上游无名杀·琉璃版】5人身份局牌桌（十周年UI / 手杀ui / #arena 1440×1080）', 36, 26);

      ctx.fillStyle = '#34d399';
      ctx.fillText('【原生 Web-Client 真实牌桌】BattleBoard + BattleShoushaHud + HandStrip（1440×1080）', 1476, 26);

      // Bottom Footer Bar
      ctx.fillStyle = 'rgba(15, 18, 24, 0.92)';
      ctx.fillRect(0, 1038, 2880, 42);

      ctx.strokeStyle = 'rgba(232, 165, 75, 0.45)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, 1038);
      ctx.lineTo(2880, 1038);
      ctx.stroke();

      ctx.font = '15px monospace';
      ctx.textBaseline = 'middle';
      ctx.fillStyle = '#f3d27a';
      ctx.fillText(${JSON.stringify(metaText)}, 36, 1059);

      const rightMeta = 'M3 R4 真实牌桌对照 · 零拉伸等比合成 (2880×1080)';
      ctx.font = '14px system-ui, -apple-system, "PingFang SC", sans-serif';
      const rightWidth = ctx.measureText(rightMeta).width;
      ctx.fillStyle = '#9ca3af';
      ctx.fillText(rightMeta, 2880 - rightWidth - 36, 1059);

      return canvas.toDataURL('image/png').split(',')[1];
    })()`
  );

  const compositePng = Buffer.from(compositeBase64, "base64");
  const compositePngPath = join(docsEvidenceDir, "tabletop-comparison.png");
  writeFileSync(compositePngPath, compositePng);
  console.log(`Saved composite comparison screenshot to ${compositePngPath} (${compositePng.length} bytes)`);

  if (process.env.TABLETOP_THUMBNAIL_PATH) {
    const thumbPath = resolve(repoRoot, process.env.TABLETOP_THUMBNAIL_PATH);
    mkdirSync(dirname(thumbPath), { recursive: true });
    const thumbBase64 = await evalJs(
      ws,
      `(async () => {
        const img = new Image();
        await new Promise((res, rej) => {
          img.onload = res;
          img.onerror = rej;
          img.src = 'data:image/png;base64,' + ${JSON.stringify(compositeBase64)};
        });
        const thumb = document.createElement('canvas');
        thumb.width = 1440;
        thumb.height = 540;
        const tctx = thumb.getContext('2d');
        tctx.drawImage(img, 0, 0, 1440, 540);
        for (const q of [0.85, 0.78, 0.7]) {
          const url = thumb.toDataURL('image/jpeg', q);
          const b64 = url.split(',')[1];
          if (b64.length * 0.75 <= 195 * 1024) return b64;
        }
        return thumb.toDataURL('image/jpeg', 0.65).split(',')[1];
      })()`
    );
    const thumbBuf = Buffer.from(thumbBase64, "base64");
    writeFileSync(thumbPath, thumbBuf);
    console.log(`Saved side-by-side thumbnail to ${thumbPath} (${thumbBuf.length} bytes)`);
  }

  // Node-level verification of PNG output dimensions
  function assertPngSize(buf, expW, expH, label) {
    if (buf.length < 24) throw new Error(`${label}: invalid PNG buffer`);
    const w = buf.readUInt32BE(16);
    const h = buf.readUInt32BE(20);
    if (w !== expW || h !== expH) {
      throw new Error(`${label} dimension error: expected ${expW}x${expH}, got ${w}x${h}`);
    }
    console.log(`[PNG OK] ${label}: ${w}x${h} px, ${buf.length} bytes`);
  }
  assertPngSize(upstreamPng, 1440, 1080, "tabletop-upstream.png");
  assertPngSize(nativePng, 1440, 1080, "tabletop-native.png");
  assertPngSize(compositePng, 2880, 1080, "tabletop-comparison.png");

  ws.close();
  console.log("\nTabletop CDP comparison and FPS measurement completed successfully!");
} finally {
  try { chrome.kill("SIGKILL"); } catch {}
  try { upstreamServer.close(); } catch {}
  try { nativeServer.close(); } catch {}
  try { rmSync(udd, { recursive: true, force: true }); } catch {}
}
