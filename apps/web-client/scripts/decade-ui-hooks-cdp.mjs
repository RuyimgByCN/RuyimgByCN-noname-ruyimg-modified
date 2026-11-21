// Verify L08 (十周年UI) 319 ui_hooks + remaining config/surface CSS fixes end-to-end in headless Chrome.
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { createServer } from "node:http";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const webClientDir = resolve(scriptsDir, "..");
const repoRoot = resolve(webClientDir, "../..");
const rawDir = resolve(repoRoot, "docs/operations/evidence-raw/ui-batch3");
const docsDir = resolve(repoRoot, "docs/operations/evidence/2026-11-21-ui-l08-hooks");
mkdirSync(rawDir, { recursive: true });
mkdirSync(docsDir, { recursive: true });

execFileSync("pnpm", ["--dir", webClientDir, "exec", "vite", "build"], { stdio: "inherit" });
const distDir = resolve(webClientDir, "dist");

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
};

const server = createServer((req, res) => {
  const urlPath = decodeURIComponent((req.url ?? "/").split("?")[0] || "/");
  let fp = resolve(distDir, "." + (urlPath === "/" ? "/index.html" : urlPath));
  if (!fp.startsWith(distDir) || !existsSync(fp) || statSync(fp).isDirectory()) {
    fp = resolve(distDir, "index.html");
  }
  res.writeHead(200, {
    "Content-Type": MIME[extname(fp)] ?? "application/octet-stream",
    "Cache-Control": "no-store",
    "Cross-Origin-Opener-Policy": "same-origin",
    "Cross-Origin-Embedder-Policy": "require-corp",
  });
  res.end(readFileSync(fp));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const baseUrl = `http://127.0.0.1:${server.address().port}`;

const chromeBin = process.env.CHROME_BIN ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const cdpPort = 9495;
const udd = mkdtempSync(join(tmpdir(), "decade-hooks-cdp-"));
const chrome = spawn(
  chromeBin,
  ["--headless=new", "--no-sandbox", `--user-data-dir=${udd}`, `--remote-debugging-port=${cdpPort}`, "--window-size=1440,1080", "about:blank"],
  { stdio: "ignore" },
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let msgId = 0;
const pending = new Map();

function sha256File(fp) {
  return createHash("sha256").update(readFileSync(fp)).digest("hex");
}

try {
  let wsUrl = null;
  for (let i = 0; i < 50; i++) {
    try {
      const list = await (await fetch(`http://127.0.0.1:${cdpPort}/json/list`)).json();
      wsUrl = list.find((t) => t.type === "page")?.webSocketDebuggerUrl;
      if (wsUrl) break;
    } catch {}
    await sleep(150);
  }
  if (!wsUrl) throw new Error("CDP target not ready");
  const ws = new WebSocket(wsUrl);
  await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
  ws.onmessage = (ev) => {
    const m = JSON.parse(ev.data);
    if (m.id && pending.has(m.id)) {
      const { resolve: res, reject: rej } = pending.get(m.id);
      pending.delete(m.id);
      m.error ? rej(new Error(JSON.stringify(m.error))) : res(m.result);
    }
  };
  const send = (method, params = {}) => {
    const id = ++msgId;
    ws.send(JSON.stringify({ id, method, params }));
    return new Promise((res, rej) => pending.set(id, { resolve: res, reject: rej }));
  };
  const evalJs = async (expression) => {
    const r = await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
    if (r.exceptionDetails) {
      throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    }
    return r.result?.value;
  };

  await send("Runtime.enable");
  await send("Page.enable");
  await send("Emulation.setDeviceMetricsOverride", { width: 1440, height: 1080, deviceScaleFactor: 1, mobile: false });
  await send("Page.navigate", { url: `${baseUrl}/battle?players=5&seed=42&mode=identity&skipAnims=1` });

  for (let i = 0; i < 60; i++) {
    await sleep(150);
    const ok = await evalJs(`document.querySelectorAll('.seat').length === 5 && document.querySelectorAll('.hand-dock .card').length > 0`);
    if (ok) break;
  }
  await sleep(250);

  // 1. Configure UI-3 remaining config surfaces via real modal
  await evalJs(`document.querySelector('[data-testid="btn-open-decade-menu-modal"]')?.click()`);
  await sleep(120);
  const modalMetrics = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const setCheck = (k, v) => {
      const el = document.querySelector('[data-testid="decade-cfg-input-' + k + '"]');
      el.checked = v;
      el.dispatchEvent(new Event('change'));
    };
    const setSel = (k, v) => {
      const el = document.querySelector('[data-testid="decade-cfg-input-' + k + '"]');
      el.value = v;
      el.dispatchEvent(new Event('change'));
    };
    document.querySelector('[data-testid="decade-cfg-tab-layout"]')?.click();
    await tick();
    setSel('borderLevel', 'kill');
    setSel('loadingStyle', 'On');
    setSel('skill_control_gold', 'onstart');
    setCheck('JBDDZ', true);

    document.querySelector('[data-testid="decade-cfg-tab-cards"]')?.click();
    await tick();
    setCheck('cardAlternateNameVisible', true);

    document.querySelector('[data-testid="decade-cfg-tab-marks"]')?.click();
    await tick();
    setSel('shadowStyle', 'on');

    document.querySelector('[data-testid="decade-cfg-tab-effects"]')?.click();
    await tick();
    setCheck('dynamicSkin', true);
    setCheck('dynamicSkinOutcrop', true);
    setSel('changeWujiangTX', 'shine');
    setSel('dynamicBackground', 'shousha_amber');
    await tick();
    const modalCard = document.querySelector('[data-testid="l08-decade-menu-modal"] .modal-card');
    const modalPaddingTop = modalCard ? parseFloat(getComputedStyle(modalCard).paddingTop) : 0;
    document.querySelector('[data-testid="btn-close-decade-modal"]')?.click();
    return { modalPaddingTop };
  })()`);
  if (!modalMetrics || modalMetrics.modalPaddingTop < 12) {
    throw new Error(`loadingStyle crushed modal padding: ${JSON.stringify(modalMetrics)}`);
  }
  await sleep(180);

  // Pick a general from GeneralSelectPanel so visible general slot + dynamicSkin canvas render live
  await evalJs(`document.querySelector('.general-select .candidate')?.click()`);
  for (let i = 0; i < 20; i++) {
    await sleep(120);
    const assigned = await evalJs(`document.querySelectorAll('[data-testid="portrait-dynamic-canvas"].outcrop-crop').length >= 1`);
    if (assigned) break;
  }

  // Advance until player 0 has a targetable card, test live CPZS arc & skill_control_gold prestart scoping
  let cpzsCheck = null;
  for (let i = 0; i < 25; i++) {
    cpzsCheck = await evalJs(`(async () => {
      const tick = () => new Promise((r) => setTimeout(r, 35));
      const playables = [...document.querySelectorAll('.hand-dock .card:not(:disabled)')];
      for (const playable of playables) {
        playable.click();
        await tick();
        const target = document.querySelector('.seat.target-selectable');
        if (target) {
          target.click();
          await tick();
          const line = document.querySelector('[data-testid="decade-effect-line"] line');
          if (line) {
            return {
              x1: line.getAttribute('x1'),
              y1: line.getAttribute('y1'),
              x2: line.getAttribute('x2'),
              y2: line.getAttribute('y2'),
              stroke: line.getAttribute('stroke'),
            };
          }
        }
      }
      const stepBtn = [...document.querySelectorAll('.toolbar button, button')].find((b) => b.textContent?.trim() === '自动一步' && !b.disabled);
      stepBtn?.click();
      return null;
    })()`);
    if (cpzsCheck) break;
    await sleep(160);
  }
  if (!cpzsCheck) {
    throw new Error('Live CPZS targeting arc failed to render on target selection');
  }

  const configVerify = await evalJs(`(() => {
    const seats = [...document.querySelectorAll('.seat.decade-seat-frame')];
    const borderLevels = seats.map((s) => s.getAttribute('data-border-level'));
    const nodeKeysAttr = seats.map((s) => s.getAttribute('data-decade-node-keys'));
    const subNodeKeys = document.querySelectorAll('.seat [data-node-key]').length;
    const uniqueFirstSeatKeys = new Set(
      [...(seats[0]?.querySelectorAll('[data-node-key]') ?? [])].map((e) => e.getAttribute('data-node-key')),
    ).size;
    const table = document.querySelector('.battle-table');
    const sec = document.querySelector('.battle-section');
    const gsPanel = document.querySelector('.general-select');
    const modalCard = document.querySelector('[data-testid="l08-decade-menu-modal"] .modal-card');
    const canvas = document.querySelector('[data-testid="portrait-dynamic-canvas"].outcrop-crop');
    let canvasNonZeroPixels = 0;
    if (canvas) {
      const ctx = canvas.getContext('2d');
      const data = ctx?.getImageData(0, 0, canvas.width, canvas.height)?.data;
      if (data) {
        for (let i = 3; i < data.length; i += 4) {
          if (data[i] > 0) canvasNonZeroPixels++;
        }
      }
    }
    const dynamicCanvases = document.querySelectorAll('[data-testid="portrait-dynamic-canvas"].outcrop-crop').length;
    const dynamicOutcropAttrs = document.querySelectorAll('.general-slot[data-dynamic-skin="on"][data-dynamic-skin-outcrop="on"]').length;
    const cardAuxCount = document.querySelectorAll('[data-testid="card-aux-overlay"]').length;
    const jbddzTag = Boolean(document.querySelector('[data-testid="jbddz-mode-tag"]'));
    const falseSelfHpDeltaCount = document.querySelectorAll('.seat.self [data-testid="seat-hp-delta"]').length;
    const falseLoseHpCount = document.querySelectorAll('.seat.self.has-lose-hp').length;
    return {
      borderLevels,
      nodeKeysAttr,
      subNodeKeys,
      uniqueFirstSeatKeys,
      prestart: table?.getAttribute('data-prestart'),
      skillGold: table?.getAttribute('data-skill-gold'),
      shadowStyle: sec?.getAttribute('data-shadow-style'),
      loadingStyle: sec?.getAttribute('data-loading-style'),
      changeWujiangTx: gsPanel?.getAttribute('data-change-wujiang-tx'),
      dynamicCanvases,
      canvasNonZeroPixels,
      dynamicOutcropAttrs,
      cardAuxCount,
      jbddzTag,
      falseSelfHpDeltaCount,
      falseLoseHpCount,
      modalExists: Boolean(modalCard),
    };
  })()`);

  if (
    !configVerify.borderLevels.includes("five") ||
    configVerify.nodeKeysAttr.some((v) => v !== "20") ||
    configVerify.subNodeKeys !== 100 ||
    configVerify.uniqueFirstSeatKeys !== 20 ||
    configVerify.prestart !== "off" ||
    configVerify.shadowStyle !== "on" ||
    configVerify.loadingStyle !== "On" ||
    configVerify.changeWujiangTx !== "shine" ||
    configVerify.dynamicOutcropAttrs < 5 ||
    configVerify.dynamicCanvases < 1 ||
    configVerify.canvasNonZeroPixels < 100 ||
    configVerify.cardAuxCount < 1 ||
    !configVerify.jbddzTag ||
    configVerify.falseSelfHpDeltaCount !== 0 ||
    configVerify.falseLoseHpCount !== 0
  ) {
    throw new Error(`UI-3 config surface check failed: ${JSON.stringify(configVerify)}`);
  }

  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(120);
  const shotPath1 = join(rawDir, "raw-hooks-config-surface.png");
  const shot1 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotPath1, Buffer.from(shot1.data, "base64"));

  // 2. Open extension workbench drawer -> test markskill badges + core UI hooks + drlt_poxi
  await evalJs(`(() => {
    const d = document.querySelector('[data-testid="extension-surface-panel-drawer"]');
    if (d && !d.open) d.open = true;
  })()`);
  await sleep(100);

  // Mount type+suit mark (`rejingce_add`) + nonumber mark (`zhaxiang2`) and demo core hooks
  const drawerCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    const sel = document.querySelector('[data-testid="select-l08-markskill"]');
    const btnApply = document.querySelector('[data-testid="btn-l08-apply-mark"]');
    sel.value = 'rejingce_add';
    sel.dispatchEvent(new Event('change'));
    await tick();
    btnApply.click();
    await tick();

    sel.value = 'zhaxiang2';
    sel.dispatchEvent(new Event('change'));
    await tick();
    const metaText = document.querySelector('[data-testid="l08-selected-mark-meta"]')?.textContent?.trim() ?? '';
    btnApply.click();
    await tick();

    document.querySelector('[data-testid="btn-l08-demo-core-hooks"]')?.click();
    await tick();
    return {
      panelTitle: document.querySelector('[data-testid="l08-decade-full-surface"] strong')?.textContent?.trim() ?? '',
      metaText,
      barMarks: document.querySelectorAll('[data-testid="l08-decade-mark-bar"] .decade-mark-chip').length,
    };
  })()`);

  if (
    !drawerCheck.panelTitle.includes("319钩子 / 43表面") ||
    !drawerCheck.metaText.includes("nonumberMark") ||
    drawerCheck.barMarks < 2
  ) {
    throw new Error(`Drawer markskill check failed: ${JSON.stringify(drawerCheck)}`);
  }

  // Test honest drlt_poxi & pro_drlt_poxi modal (non-self opponent, unrevealed card backs without fake suits)
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(120);

  const poxiCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const dialog = document.querySelector('[data-testid="decade-poxi-dialog"]');
    const targetHeader = [...(dialog?.querySelectorAll('.poxi-pane span') ?? [])].find((el) => el.textContent?.includes('目标 P'))?.textContent?.trim() ?? '';
    const hiddenTargetCards = [...document.querySelectorAll('[data-testid="poxi-hidden-card-btn"]')];
    const allHiddenDisabled = hiddenTargetCards.length > 0 && hiddenTargetCards.every((b) => b.disabled && b.getAttribute('data-suit') === '');

    const revealedCards = [...document.querySelectorAll('[data-testid="poxi-card-btn"]')];
    const pickedSuits = new Set();
    for (const c of revealedCards) {
      const s = c.getAttribute('data-suit');
      if (s && !pickedSuits.has(s) && pickedSuits.size < 4) {
        pickedSuits.add(s);
        c.click();
        await tick();
      }
    }
    const statusBefore = document.querySelector('[data-testid="poxi-validation"]');
    const validDrlt = statusBefore?.classList.contains('is-valid') ?? false;

    // If < 4 distinct revealed suits exist, drlt_poxi must remain invalid; switching to pro_drlt_poxi (1-4 distinct suits) accepts >= 1 real distinct suit
    if (!validDrlt) {
      document.querySelector('[data-testid="btn-poxi-switch-mode"]')?.click();
      await tick();
      const firstRevealed = document.querySelector('[data-testid="poxi-card-btn"]');
      firstRevealed?.click();
      await tick();
    }
    const statusAfter = document.querySelector('[data-testid="poxi-validation"]');
    return {
      dialogOpen: Boolean(dialog),
      targetHeader,
      hiddenTargetCount: hiddenTargetCards.length,
      allHiddenDisabled,
      pickedCount: pickedSuits.size,
      isValidAfter: statusAfter?.classList.contains('is-valid') ?? false,
      text: statusAfter?.textContent?.trim() ?? '',
    };
  })()`);
  if (
    !poxiCheck.dialogOpen ||
    poxiCheck.targetHeader.includes("目标 P1 ") ||
    !poxiCheck.allHiddenDisabled ||
    !poxiCheck.isValidAfter
  ) {
    throw new Error(`Honest poxi check failed: ${JSON.stringify(poxiCheck)}`);
  }

  const shotPath2 = join(rawDir, "raw-hooks-poxi-active.png");
  const shot2 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotPath2, Buffer.from(shot2.data, "base64"));

  // Close poxi modal, click mark badge on seat to verify mark intro popover, capture clean tabletop screenshot
  await evalJs(`document.querySelector('[data-testid="btn-poxi-close"]')?.click()`);
  await sleep(100);
  const hooksUiCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    const firstMark = document.querySelector('[data-testid="seat-mark-badge"]');
    firstMark?.click();
    await tick();
    return {
      jiuwoCount: document.querySelectorAll('[data-testid="seat-jiuwo-badge"]').length,
      trickBanner: document.querySelector('[data-testid="decade-trick-banner"]')?.textContent?.trim() ?? null,
      hpDeltaText: document.querySelector('[data-testid="seat-hp-delta"]')?.textContent?.trim() ?? null,
      tiesuoCount: document.querySelectorAll('[data-testid="seat-tiesuo-badge"]').length,
      virtualEquipText: document.querySelector('[data-testid="seat-virtual-equip"]')?.textContent?.trim() ?? null,
      erudaCheat: document.querySelector('[data-testid="decade-eruda-bar"]')?.textContent?.includes('_ui_auto_cheat') ?? false,
      markPopoverText: document.querySelector('[data-testid="seat-mark-intro-popover"]')?.textContent?.trim() ?? null,
      nonumberBadges: [...document.querySelectorAll('[data-testid="seat-mark-badge"][data-nonumber="on"]')].map((e) => e.textContent?.trim()),
    };
  })()`);

  if (
    hooksUiCheck.jiuwoCount < 1 ||
    !hooksUiCheck.trickBanner ||
    hooksUiCheck.hpDeltaText !== "-1" ||
    hooksUiCheck.tiesuoCount < 1 ||
    !hooksUiCheck.virtualEquipText?.includes("八卦") ||
    !hooksUiCheck.erudaCheat ||
    !hooksUiCheck.markPopoverText ||
    !hooksUiCheck.nonumberBadges.some((t) => t?.includes("[态]"))
  ) {
    throw new Error(`Live tabletop hooks DOM check failed: ${JSON.stringify(hooksUiCheck)}`);
  }

  const shotPath3 = join(rawDir, "raw-hooks-tabletop-effects.png");
  const shot3 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotPath3, Buffer.from(shot3.data, "base64"));

  // Advance turn/phase and verify transient cues (hpDelta, trickBanner, jiuwo) expire cleanly cleanly
  await evalJs(`(() => {
    const endBtn = document.querySelector('[data-testid="btn-tabletop-end-phase"]:not(:disabled)');
    if (endBtn) endBtn.click();
    else {
      const stepBtn = [...document.querySelectorAll('.toolbar button, button')].find((b) => b.textContent?.trim() === '自动一步' && !b.disabled);
      stepBtn?.click();
    }
  })()`);
  await sleep(250);
  const expiryCheck = await evalJs(`({
    hpDeltaLeft: document.querySelectorAll('[data-testid="seat-hp-delta"]').length,
    trickBannerLeft: document.querySelectorAll('[data-testid="decade-trick-banner"]').length,
    jiuwoLeft: document.querySelectorAll('[data-testid="seat-jiuwo-badge"]').length,
  })`);
  if (expiryCheck.hpDeltaLeft !== 0 || expiryCheck.trickBannerLeft !== 0 || expiryCheck.jiuwoLeft !== 0) {
    throw new Error(`Transient cues did not expire on phase advance: ${JSON.stringify(expiryCheck)}`);
  }

  // 3. Composite <= 200 KB side-by-side JPEG thumbnail
  const [b1, b2, b3] = [shotPath1, shotPath2, shotPath3].map((p) => readFileSync(p).toString("base64"));
  const thumbDataUrl = await evalJs(`(async () => {
    const loadImg = (src) => new Promise((res, rej) => {
      const im = new Image();
      im.onload = () => res(im);
      im.onerror = rej;
      im.src = 'data:image/png;base64,' + src;
    });
    const [i1, i2, i3] = await Promise.all([loadImg(${JSON.stringify(b1)}), loadImg(${JSON.stringify(b2)}), loadImg(${JSON.stringify(b3)})]);
    const w = 480, h = 360;
    const canvas = document.createElement('canvas');
    canvas.width = w * 3;
    canvas.height = h + 32;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#0d1117';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const labels = ['Kill Border / Outcrop / CardAux / CPZS', 'drlt_poxi 4-Suit Dual Hand Check', '319 UI Hooks Active + Mark Popover'];
    [i1, i2, i3].forEach((im, idx) => {
      ctx.drawImage(im, idx * w, 32, w, h);
      ctx.fillStyle = '#f5ebd4';
      ctx.font = 'bold 14px sans-serif';
      ctx.fillText(labels[idx], idx * w + 12, 22);
    });
    return canvas.toDataURL('image/jpeg', 0.8);
  })()`);
  const thumbPath = join(docsDir, "decade-hooks-side-by-side.jpg");
  writeFileSync(thumbPath, Buffer.from(thumbDataUrl.split(",")[1], "base64"));
  const thumbBytes = statSync(thumbPath).size;
  if (thumbBytes > 200 * 1024) {
    throw new Error(`Thumbnail exceeded 200 KB: ${thumbBytes}`);
  }

  console.log("DECADE_UI_HOOKS_CDP_RESULT=" + JSON.stringify({
    modalMetrics,
    cpzsCheck,
    configVerify,
    drawerCheck,
    poxiCheck,
    hooksUiCheck,
    expiryCheck,
    rawSha256: {
      "raw-hooks-config-surface.png": sha256File(shotPath1),
      "raw-hooks-poxi-active.png": sha256File(shotPath2),
      "raw-hooks-tabletop-effects.png": sha256File(shotPath3),
    },
    thumbBytes,
  }));
  ws.close();
} finally {
  try { chrome.kill("SIGKILL"); } catch {}
  try { server.close(); } catch {}
  try { rmSync(udd, { recursive: true, force: true }); } catch {}
}
