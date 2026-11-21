// Verify L08 (十周年UI) 44 configs & 43 surfaces end-to-end in headless Chrome (real vite build + real WASM match).
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
const rawDir = resolve(repoRoot, "docs/operations/evidence-raw/ui-batch2");
const docsDir = resolve(repoRoot, "docs/operations/evidence/2026-11-21-ui-l08-configs-surfaces");
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
const cdpPort = 9494;
const udd = mkdtempSync(join(tmpdir(), "decade-cfg-cdp-"));
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

  await send("Page.navigate", { url: `${baseUrl}/battle?players=5&seed=42&mode=identity` });
  for (let i = 0; i < 60; i++) {
    await sleep(150);
    const ok = await evalJs(`document.querySelectorAll('.seat').length === 5 && Boolean(document.querySelector('[data-testid="decade-start-banner"]'))`);
    if (ok) break;
  }
  await sleep(250);
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(150);

  // 1. Verify default layout + non-overlapping start banner + rightLayout=true metrics
  const defaultCheck = await evalJs(`(() => {
    const table = document.querySelector('.battle-table');
    const selfRect = document.querySelector('.seat-wrapper[data-position="0"]')?.getBoundingClientRect();
    const handRect = document.querySelector('.hand-dock')?.getBoundingClientRect();
    const bannerRect = document.querySelector('[data-testid="decade-start-banner"]')?.getBoundingClientRect();
    const seats = [...document.querySelectorAll('.seat')].map((el) => el.getBoundingClientRect());
    const bannerOverlap = bannerRect && seats.some((s) => Math.min(s.right, bannerRect.right) > Math.max(s.left, bannerRect.left) + 2 && Math.min(s.bottom, bannerRect.bottom) > Math.max(s.top, bannerRect.top) + 2);
    return {
      style: table?.getAttribute('data-new-decade-style'),
      layout: table?.getAttribute('data-decade-layout'),
      rightLayout: table?.getAttribute('data-right-layout'),
      selfLeft: selfRect?.left ?? -1,
      handLeft: handRect?.left ?? -1,
      bannerText: document.querySelector('[data-testid="decade-start-banner"]')?.textContent?.trim(),
      bannerOverlap,
    };
  })()`);
  if (defaultCheck.style !== "on" || defaultCheck.rightLayout !== "on" || defaultCheck.bannerOverlap) {
    throw new Error(`Default L08 state check failed: ${JSON.stringify(defaultCheck)}`);
  }
  const defaultShotPath = join(rawDir, "raw-default-shizhounian.png");
  const shot1 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(defaultShotPath, Buffer.from(shot1.data, "base64"));

  // Dismiss start banner
  await evalJs(`document.querySelector('[data-testid="decade-start-banner"]')?.click()`);

  // 2. Open DecadeConfigModal & verify all 44 configs + 43 surfaces exist across tabs
  await evalJs(`document.querySelector('[data-testid="btn-open-decade-menu-modal"]')?.click()`);
  await sleep(150);
  const countCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const keys = [];
    for (const tab of ['layout', 'cards', 'marks', 'effects', 'system']) {
      document.querySelector('[data-testid="decade-cfg-tab-' + tab + '"]')?.click();
      await tick();
      for (const row of document.querySelectorAll('[data-testid^="decade-cfg-row-"]')) {
        keys.push(row.getAttribute('data-testid').replace('decade-cfg-row-', ''));
      }
    }
    document.querySelector('[data-testid="decade-cfg-tab-surfaces"]')?.click();
    await tick();
    const cssCount = document.querySelectorAll('.css-pill').length;
    document.querySelector('[data-testid="decade-cfg-tab-layout"]')?.click();
    await tick();
    return { cfgCount: keys.length, uniqueCount: new Set(keys).size, cssCount };
  })()`);
  if (countCheck.cfgCount !== 44 || countCheck.uniqueCount !== 44 || countCheck.cssCount !== 33) {
    throw new Error(`Modal 44/33 count mismatch: ${JSON.stringify(countCheck)}`);
  }
  const modalShotPath = join(rawDir, "raw-decade-config-modal.png");
  const shot2 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(modalShotPath, Buffer.from(shot2.data, "base64"));

  // 3. Apply real config changes in modal across layout / cards / marks / effects / system
  await evalJs(`(async () => {
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
    setSel('newDecadeStyle', 'off');
    setSel('decadeLayout', 'off');
    setCheck('rightLayout', false);
    setSel('borderLevel', 'five');
    setSel('rarityLong', 'on');
    setSel('loadingStyle', 'On');
    setSel('skill_control_gold', 'forever');

    document.querySelector('[data-testid="decade-cfg-tab-cards"]')?.click();
    await tick();
    setSel('cardBorder', 'four');
    setCheck('cardAlternateNameVisible', true);

    document.querySelector('[data-testid="decade-cfg-tab-marks"]')?.click();
    await tick();
    setCheck('campIdentityImageMode', false);
    setSel('playerMarkStyle', 'red');

    document.querySelector('[data-testid="decade-cfg-tab-effects"]')?.click();
    await tick();
    setSel('dynamicBackground', 'decade_jade');
    setCheck('EffectWood', true);
    setCheck('playerKillEffect', true);

    document.querySelector('[data-testid="decade-cfg-tab-system"]')?.click();
    await tick();
    setCheck('eruda', true);
  })()`);
  await sleep(120);

  // Verify Decision 38 JSON import rejection & valid JSON import on surfaces tab
  const importCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    document.querySelector('[data-testid="decade-cfg-tab-surfaces"]')?.click();
    await tick();
    const ta = document.querySelector('[data-testid="decade-cfg-json-textarea"]');
    const btn = document.querySelector('[data-testid="btn-decade-import-json"]');
    ta.value = '<script>alert(1)</script>';
    ta.dispatchEvent(new Event('input'));
    await tick();
    btn.click();
    await tick();
    const rejectedMsg = document.querySelector('[data-testid="decade-cfg-import-msg"]')?.textContent ?? '';
    ta.value = '{"cardBorder":123}';
    ta.dispatchEvent(new Event('input'));
    await tick();
    btn.click();
    await tick();
    const badEnumMsg = document.querySelector('[data-testid="decade-cfg-import-msg"]')?.textContent ?? '';
    return { rejectedMsg, badEnumMsg };
  })()`);
  await sleep(60);
  const acceptImportMsg = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const ta = document.querySelector('[data-testid="decade-cfg-json-textarea"]');
    const btn = document.querySelector('[data-testid="btn-decade-import-json"]');
    ta.value = '{"asyncMode":true}';
    ta.dispatchEvent(new Event('input'));
    await tick();
    btn.click();
    await tick();
    return document.querySelector('[data-testid="decade-cfg-import-msg"]')?.textContent ?? '';
  })()`);
  if (
    !importCheck.rejectedMsg.includes("DECADE_CONFIG_IMPORT_UNSAFE") ||
    !importCheck.badEnumMsg.includes("DECADE_CONFIG_IMPORT_INVALID_ENUM") ||
    !acceptImportMsg.includes("1 个键")
  ) {
    throw new Error(`JSON import check failed: ${JSON.stringify(importCheck)}, ok=${acceptImportMsg}`);
  }

  await evalJs(`document.querySelector('[data-testid="btn-close-decade-modal"]')?.click()`);
  await sleep(150);

  const liveAttrCheck = await evalJs(`(() => {
    const table = document.querySelector('.battle-table');
    const selfRect = document.querySelector('.seat-wrapper[data-position="0"]')?.getBoundingClientRect();
    const handRect = document.querySelector('.hand-dock')?.getBoundingClientRect();
    const campGlyphs = [...document.querySelectorAll('.seat-identity-stamp.camp-text-only')].map((e) => e.textContent?.trim());
    return {
      newDecadeStyle: table?.getAttribute('data-new-decade-style'),
      decadeLayout: table?.getAttribute('data-decade-layout'),
      rightLayout: table?.getAttribute('data-right-layout'),
      borderLevel: table?.getAttribute('data-border-level'),
      cardBorder: table?.getAttribute('data-card-border'),
      playerMarkStyle: table?.getAttribute('data-player-mark-style'),
      dynamicBackground: table?.getAttribute('data-dynamic-background'),
      loadingStyle: table?.getAttribute('data-loading-style'),
      skillGold: table?.getAttribute('data-skill-gold'),
      cardAlt: table?.getAttribute('data-card-alternate-name-visible'),
      campTextCount: campGlyphs.length,
      campTextValid: campGlyphs.every((g) => g && g.length === 1 && ['主', '忠', '反', '内', '猜'].includes(g)),
      dragonCount: document.querySelectorAll('[data-testid="seat-dragon-crest"]').length,
      erudaVisible: Boolean(document.querySelector('[data-testid="decade-eruda-bar"]')),
      selfLeft: selfRect?.left ?? -1,
      handLeft: handRect?.left ?? -1,
    };
  })()`);
  if (
    liveAttrCheck.newDecadeStyle !== "off" ||
    liveAttrCheck.decadeLayout !== "off" ||
    liveAttrCheck.rightLayout !== "off" ||
    liveAttrCheck.handLeft >= liveAttrCheck.selfLeft ||
    liveAttrCheck.campTextCount !== 5 ||
    !liveAttrCheck.campTextValid ||
    liveAttrCheck.dragonCount !== 5 ||
    !liveAttrCheck.erudaVisible
  ) {
    throw new Error(`Live DOM config projection failed: ${JSON.stringify(liveAttrCheck)}`);
  }

  // 4. Test tabletop chatBox + pauseHistory + JS effect surfaces
  await evalJs(`document.querySelector('[data-testid="btn-tabletop-chat"]')?.click()`);
  await sleep(80);
  await evalJs(`(() => {
    const input = document.querySelector('[data-testid="decade-chat-input"]');
    input.value = '十周年快捷语音';
    input.dispatchEvent(new Event('input'));
    document.querySelector('[data-testid="btn-decade-chat-send"]')?.click();
  })()`);
  await sleep(80);
  const chatVisible = await evalJs(`document.querySelector('[data-testid="decade-chat-box"]')?.textContent?.includes('十周年快捷语音')`);
  if (!chatVisible) throw new Error("Chat message failed to render");

  // Open pause history briefly then close
  await evalJs(`document.querySelector('[data-testid="btn-tabletop-pause"]')?.click()`);
  await sleep(80);
  const pauseRows = await evalJs(`document.querySelectorAll('[data-testid="decade-pause-history"] li').length`);
  if (pauseRows !== 5) throw new Error(`Pause history expected 5 seats, got ${pauseRows}`);
  await evalJs(`document.querySelector('[data-testid="decade-pause-history"] .mini-close')?.click()`);

  // Trigger JS effect surfaces + GuanXing preview via ExtensionSurfacePanel -> ui-suites
  await evalJs(`(() => {
    const drawer = document.querySelector('[data-testid="extension-surface-panel-drawer"]');
    if (drawer && !drawer.open) drawer.open = true;
    document.querySelector('[data-testid="tab-ui-suites"]')?.click();
  })()`);
  await sleep(100);
  await evalJs(`(() => {
    document.querySelector('[data-testid="btn-l08-apply-mark"]')?.click();
    document.querySelector('[data-testid="btn-l08-demo-effects"]')?.click();
    document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' });
  })()`);
  await sleep(180);
  const effectsCheck = await evalJs(`(() => {
    const ov = (a, b) => {
      if (!a || !b) return false;
      return Math.min(a.right, b.right) > Math.max(a.left, b.left) + 2 &&
        Math.min(a.bottom, b.bottom) > Math.max(a.top, b.top) + 2;
    };
    const chatRect = document.querySelector('[data-testid="decade-chat-box"]')?.getBoundingClientRect();
    const lipaiRect = document.querySelector('[data-testid="btn-handstrip-lipai"]')?.getBoundingClientRect();
    const tipBarRect = document.querySelector('[data-testid="tabletop-hand-tip-bar"]')?.getBoundingClientRect();
    const compareRect = document.querySelector('[data-testid="decade-effect-compare"]')?.getBoundingClientRect();
    const seatRects = [...document.querySelectorAll('.seat')].map((e) => e.getBoundingClientRect());
    const chatOverlapsSeat = seatRects.some((s) => ov(chatRect, s));
    const chatOverlapsLipai = ov(chatRect, lipaiRect);
    const chatOverlapsTipBar = ov(chatRect, tipBarRect);
    const chatOverlapsCompare = ov(chatRect, compareRect);

    const stageRect = document.querySelector('.arena-stage')?.getBoundingClientRect();
    const lineEl = document.querySelector('[data-testid="decade-effect-line"] line');
    const p1Rect = document.querySelector('.seat-wrapper[data-seat-player-id="1"] .seat')?.getBoundingClientRect();
    const p0Rect = document.querySelector('.seat-wrapper[data-seat-player-id="0"] .seat')?.getBoundingClientRect();
    let lineInsideP1 = false;
    let lineInsideP0 = false;
    if (stageRect && lineEl && p1Rect && p0Rect) {
      const x1 = stageRect.left + (Number(lineEl.getAttribute('x1')) / 100) * stageRect.width;
      const y1 = stageRect.top + (Number(lineEl.getAttribute('y1')) / 100) * stageRect.height;
      const x2 = stageRect.left + (Number(lineEl.getAttribute('x2')) / 100) * stageRect.width;
      const y2 = stageRect.top + (Number(lineEl.getAttribute('y2')) / 100) * stageRect.height;
      lineInsideP1 = x1 >= p1Rect.left && x1 <= p1Rect.right && y1 >= p1Rect.top && y1 <= p1Rect.bottom;
      lineInsideP0 = x2 >= p0Rect.left && x2 <= p0Rect.right && y2 >= p0Rect.top && y2 <= p0Rect.bottom;
    }

    return {
      kill: Boolean(document.querySelector('[data-testid="decade-effect-kill"]')),
      skill: Boolean(document.querySelector('[data-testid="decade-effect-skill"]')),
      compare: Boolean(compareRect),
      line: Boolean(lineEl),
      healAura: document.querySelectorAll('.seat.has-heal-aura').length,
      chatOverlapsSeat,
      chatOverlapsLipai,
      chatOverlapsTipBar,
      chatOverlapsCompare,
      lineInsideP1,
      lineInsideP0,
    };
  })()`);
  if (
    !effectsCheck.kill ||
    !effectsCheck.skill ||
    !effectsCheck.compare ||
    !effectsCheck.line ||
    effectsCheck.healAura < 1 ||
    effectsCheck.chatOverlapsSeat ||
    effectsCheck.chatOverlapsLipai ||
    effectsCheck.chatOverlapsTipBar ||
    effectsCheck.chatOverlapsCompare ||
    !effectsCheck.lineInsideP1 ||
    !effectsCheck.lineInsideP0
  ) {
    throw new Error(`JS surface / layout geometry checks failed: ${JSON.stringify(effectsCheck)}`);
  }
  const activeShotPath = join(rawDir, "raw-themed-surfaces-active.png");
  const shot3 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(activeShotPath, Buffer.from(shot3.data, "base64"));

  // Test GuanXing modal swap then close
  await evalJs(`document.querySelector('[data-testid="btn-l08-demo-guanxing"]')?.click()`);
  await sleep(100);
  const gxOpen = await evalJs(`document.querySelectorAll('[data-testid="decade-guanxing-dialog"] .gx-card').length`);
  if (gxOpen !== 3) throw new Error(`GuanXing expected 3 cards, got ${gxOpen}`);
  await evalJs(`document.querySelector('[data-testid="decade-guanxing-dialog"] .mini-close')?.click()`);
  await sleep(80);

  // 4b. Switch newDecadeStyle="on", rightLayout=true, aloneEquip=true; verify chat overlap=false + vertical alone-equip-dock above HandStrip with real self equip
  await evalJs(`document.querySelector('[data-testid="btn-open-decade-menu-modal"]')?.click()`);
  await sleep(100);
  await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    document.querySelector('[data-testid="decade-cfg-tab-layout"]')?.click();
    await tick();
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
    setSel('newDecadeStyle', 'on');
    await tick();
    setCheck('rightLayout', true);
    await tick();
    setCheck('aloneEquip', true);
    await tick();
    document.querySelector('[data-testid="btn-close-decade-modal"]')?.click();
  })()`);
  await sleep(140);

  const chatRightTrueCheck = await evalJs(`(() => {
    const ov = (a, b) => {
      if (!a || !b) return false;
      return Math.min(a.right, b.right) > Math.max(a.left, b.left) + 2 &&
        Math.min(a.bottom, b.bottom) > Math.max(a.top, b.top) + 2;
    };
    const chatRect = document.querySelector('[data-testid="decade-chat-box"]')?.getBoundingClientRect();
    const lipaiRect = document.querySelector('[data-testid="btn-handstrip-lipai"]')?.getBoundingClientRect();
    const seatRects = [...document.querySelectorAll('.seat')].map((e) => e.getBoundingClientRect());
    return {
      chatOverlapsSeat: seatRects.some((s) => ov(chatRect, s)),
      chatOverlapsLipai: ov(chatRect, lipaiRect),
    };
  })()`);
  if (chatRightTrueCheck.chatOverlapsSeat || chatRightTrueCheck.chatOverlapsLipai) {
    throw new Error(`Chat overlap under rightLayout=true failed: ${JSON.stringify(chatRightTrueCheck)}`);
  }
  await evalJs(`document.querySelector('[data-testid="decade-chat-box"] .mini-close')?.click()`);

  // Advance real WASM match until self equips a card (+1马 or autoStep) and verify alone-equip-dock stacks vertically above HandStrip
  let aloneEquipCheck = null;
  let lastStepDebug = null;
  for (let step = 0; step < 80; step++) {
    const dockNow = await evalJs(`(() => {
      const dock = document.querySelector('[data-testid="alone-equip-dock"]');
      const hw = document.querySelector('.decade-hand-wrap');
      if (!dock || !hw) return null;
      const dr = dock.getBoundingClientRect();
      const hr = hw.getBoundingClientRect();
      return {
        dockBottom: Math.round(dr.bottom),
        handTop: Math.round(hr.top),
        dockWidth: Math.round(dr.width),
        handWidth: Math.round(hr.width),
        portraitEquips: document.querySelectorAll('.seat-wrapper[data-position="0"] .equip-zone').length,
      };
    })()`);
    if (dockNow) {
      aloneEquipCheck = dockNow;
      break;
    }
    lastStepDebug = await evalJs(`(async () => {
      const choiceBtn = document.querySelector('.choice button:not(:disabled)');
      if (choiceBtn) {
        choiceBtn.click();
        return { act: 'choice', label: choiceBtn.textContent?.trim() };
      }
      const equipCard = document.querySelector('button.card.card-cat-equip:not(:disabled)');
      if (equipCard) {
        equipCard.click();
        await new Promise((r) => setTimeout(r, 80));
        const btn = document.querySelector('[aria-label="确认行动"]:not(:disabled), [aria-label="无目标直接打出"]:not(:disabled)');
        if (btn) { btn.click(); return { act: 'equip', kind: equipCard.getAttribute('data-kind') }; }
        return { act: 'equip-no-btn', kind: equipCard.getAttribute('data-kind') };
      }
      const endBtn = document.querySelector('[data-testid="btn-tabletop-end-phase"]:not(:disabled)');
      if (endBtn) {
        endBtn.click();
        return { act: 'end-phase' };
      }
      const stepBtn = [...document.querySelectorAll('.toolbar button, button')].find((b) => b.textContent?.trim() === '自动一步' && !b.disabled);
      stepBtn?.click();
      return { act: stepBtn ? 'auto-step' : 'busy', tip: document.querySelector('[data-testid="tabletop-hand-tip-bar"]')?.textContent?.trim() };
    })()`);
    await sleep(lastStepDebug?.act === 'equip' ? 280 : 160);
  }
  if (
    !aloneEquipCheck ||
    aloneEquipCheck.dockBottom > aloneEquipCheck.handTop + 2 ||
    aloneEquipCheck.dockWidth < 120 ||
    aloneEquipCheck.portraitEquips !== 0
  ) {
    throw new Error(`aloneEquip vertical stacking check failed: ${JSON.stringify({ aloneEquipCheck, lastStepDebug })}`);
  }

  // 5. Verify localStorage config persistence across full reload
  await send("Page.navigate", { url: `${baseUrl}/battle?players=5&seed=42&mode=identity&skipAnims=1` });
  for (let i = 0; i < 60; i++) {
    await sleep(150);
    const ok = await evalJs(`document.querySelectorAll('.seat').length === 5`);
    if (ok) break;
  }
  const persistedCheck = await evalJs(`(() => {
    const t = document.querySelector('.battle-table');
    return {
      borderLevel: t?.getAttribute('data-border-level'),
      dynamicBackground: t?.getAttribute('data-dynamic-background'),
    };
  })()`);
  if (persistedCheck.borderLevel !== "five" || persistedCheck.dynamicBackground !== "decade_jade") {
    throw new Error(`Config persistence failed across reload: ${JSON.stringify(persistedCheck)}`);
  }

  // 6. Generate compact <= 200 KB side-by-side thumbnail in docsDir via Canvas
  const b64_1 = readFileSync(defaultShotPath).toString("base64");
  const b64_2 = readFileSync(modalShotPath).toString("base64");
  const b64_3 = readFileSync(activeShotPath).toString("base64");
  const thumbDataUrl = await evalJs(`(async () => {
    const loadImg = (src) => new Promise((res, rej) => {
      const im = new Image();
      im.onload = () => res(im);
      im.onerror = rej;
      im.src = 'data:image/png;base64,' + src;
    });
    const [i1, i2, i3] = await Promise.all([loadImg(${JSON.stringify(b64_1)}), loadImg(${JSON.stringify(b64_2)}), loadImg(${JSON.stringify(b64_3)})]);
    const w = 480, h = 360;
    const canvas = document.createElement('canvas');
    canvas.width = w * 3;
    canvas.height = h + 32;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#0d1117';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const labels = ['Default (newDecadeStyle=on)', '44 Configs / 43 Surfaces Menu', 'Live Theme + Chat + Effects'];
    [i1, i2, i3].forEach((im, idx) => {
      ctx.drawImage(im, idx * w, 32, w, h);
      ctx.fillStyle = '#f5ebd4';
      ctx.font = 'bold 14px sans-serif';
      ctx.fillText(labels[idx], idx * w + 12, 22);
    });
    return canvas.toDataURL('image/jpeg', 0.82);
  })()`);
  const thumbPath = join(docsDir, "decade-configs-side-by-side.jpg");
  writeFileSync(thumbPath, Buffer.from(thumbDataUrl.split(",")[1], "base64"));

  const result = {
    defaultCheck,
    countCheck,
    liveAttrCheck,
    effectsCheck,
    chatRightTrueCheck,
    aloneEquipCheck,
    persistedCheck,
    rawSha256: {
      "raw-default-shizhounian.png": sha256File(defaultShotPath),
      "raw-decade-config-modal.png": sha256File(modalShotPath),
      "raw-themed-surfaces-active.png": sha256File(activeShotPath),
    },
    thumbBytes: statSync(thumbPath).size,
  };
  console.log("DECADE_CONFIGS_CDP_RESULT=" + JSON.stringify(result));
  if (result.thumbBytes > 200 * 1024) {
    throw new Error(`Thumbnail exceeded 200 KB: ${result.thumbBytes}`);
  }
  ws.close();
} finally {
  try { chrome.kill("SIGKILL"); } catch {}
  try { server.close(); } catch {}
  try { rmSync(udd, { recursive: true, force: true }); } catch {}
}
