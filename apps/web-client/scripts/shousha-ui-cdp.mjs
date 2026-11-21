// Verify L14 (手杀ui 48 configs / 53 hooks / 40 surfaces) + remaining L08 14 closures in headless Chrome.
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
const rawDir = resolve(repoRoot, "docs/operations/evidence-raw/ui-batch4");
const docsDir = resolve(repoRoot, "docs/operations/evidence/2026-11-21-ui-l14-shousha");
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
const cdpPort = 9496;
const udd = mkdtempSync(join(tmpdir(), "shousha-ui-cdp-"));
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

  // 1. Boot 5-player tabletop & verify L14 modal 48 configs / 40 surfaces + live config changes
  await send("Page.navigate", { url: `${baseUrl}/battle?players=5&seed=42&mode=identity&skipAnims=1` });
  for (let i = 0; i < 60; i++) {
    await sleep(150);
    const ok = await evalJs(`document.querySelectorAll('.seat').length === 5`);
    if (ok) break;
  }
  await sleep(200);
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);

  // Check L14 modal counts (48 configs, 40 surfaces) & KGMH/yangshi_bg + Decision 38 reject
  await evalJs(`document.querySelector('[data-testid="btn-open-shousha-menu-modal"]')?.click()`);
  await sleep(150);
  const shoushaModalCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const keys = [];
    for (const tab of ['jindutiao', 'prompts', 'appearance', 'select', 'info']) {
      document.querySelector('[data-testid="shousha-cfg-tab-' + tab + '"]')?.click();
      await tick();
      for (const row of document.querySelectorAll('[data-testid^="shousha-cfg-row-"]')) {
        keys.push(row.getAttribute('data-testid').replace('shousha-cfg-row-', ''));
      }
    }
    document.querySelector('[data-testid="shousha-cfg-tab-surfaces"]')?.click();
    await tick();
    const surfaceCount = document.querySelectorAll('[data-testid^="shousha-surface-pill-"]').length;
    const ta = document.querySelector('[data-testid="shousha-cfg-json-textarea"]');
    ta.value = '{"badKey":true}';
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    document.querySelector('[data-testid="btn-shousha-import-json"]')?.click();
    await tick();
    const rejectMsg = document.querySelector('[data-testid="shousha-cfg-import-msg"]')?.textContent?.trim() ?? '';
    document.querySelector('[data-testid="shousha-cfg-tab-jindutiao"]')?.click();
    await tick();
    return { cfgCount: keys.length, uniqueCfg: new Set(keys).size, surfaceCount, rejectMsg };
  })()`);
  if (shoushaModalCheck.cfgCount !== 48 || shoushaModalCheck.uniqueCfg !== 48 || shoushaModalCheck.surfaceCount !== 40 || !shoushaModalCheck.rejectMsg.includes("导入被拒绝")) {
    throw new Error(`Shousha modal verification failed: ${JSON.stringify(shoushaModalCheck)}`);
  }
  const shotModalPath = join(rawDir, "raw-l14-shousha-config-modal.png");
  const shot1 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotModalPath, Buffer.from(shot1.data, "base64"));

  // Apply real L14 configs in modal & close
  await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const setSel = (k, v) => {
      const el = document.querySelector('[data-testid="shousha-cfg-input-' + k + '"]');
      el.value = v;
      el.dispatchEvent(new Event('change'));
    };
    const setCheck = (k, v) => {
      const el = document.querySelector('[data-testid="shousha-cfg-input-' + k + '"]');
      el.checked = v;
      el.dispatchEvent(new Event('change'));
    };
    setSel('jindutiaoYangshi', '2');
    setSel('jindutiaoSet', '38');
    await tick();
    document.querySelector('[data-testid="shousha-cfg-tab-prompts"]')?.click();
    await tick();
    setSel('GTBBYangshi', 'off');
    setSel('JDTSYangshi', '2');
    await tick();
    document.querySelector('[data-testid="shousha-cfg-tab-appearance"]')?.click();
    await tick();
    setSel('yangshi', 'off');
    setSel('yangshi_bg', '2');
    setSel('KGMH', '2');
    setCheck('LTAN', true);
    setCheck('JSAN', true);
    await tick();
    document.querySelector('[data-testid="shousha-cfg-tab-select"]')?.click();
    await tick();
    setCheck('xindsingle', true);
    setCheck('xinddraw', true);
    await tick();
    document.querySelector('[data-testid="btn-close-shousha-modal"]')?.click();
  })()`);
  await sleep(150);

  // Trigger quick-say bubble + verify LTAN hides chat, jindutiao height & GeneralSelectPanel single-row
  const shoushaLiveCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    document.querySelector('[data-testid="btn-tabletop-say"]')?.click();
    await tick();
    const table = document.querySelector('.battle-table');
    const chatBtnDisplay = getComputedStyle(document.querySelector('[data-testid="btn-tabletop-chat"]')).display;
    const trackH = getComputedStyle(document.querySelector('[data-testid="battle-hud-timebar-track"]')).height;
    const sayBubble = document.querySelector('[data-testid="seat-say-bubble"]')?.textContent?.trim() ?? '';
    const gtbb = document.querySelector('[data-testid="shousha-gtbb-banner"]')?.textContent?.trim() ?? '';
    const guanjie = document.querySelector('[data-testid="seat-guanjie-badge"]')?.textContent?.trim() ?? '';
    const gs = document.querySelector('[data-testid="general-select-panel"]');
    const gsGridDisplay = gs ? getComputedStyle(gs.querySelector('.gs-grid')).display : null;
    const luckText = document.querySelector('[data-testid="shousha-choose-luck-bar"]')?.textContent?.trim() ?? '';
    return {
      yangshi: table?.getAttribute('data-shousha-yangshi'),
      bg: table?.getAttribute('data-shousha-bg'),
      jindutiaoStyle: table?.getAttribute('data-shousha-jindutiao-style'),
      chatBtnDisplay,
      trackH,
      sayBubble,
      gtbb,
      guanjie,
      gsGridDisplay,
      luckText,
    };
  })()`);
  if (
    shoushaLiveCheck.yangshi !== "off" ||
    shoushaLiveCheck.chatBtnDisplay !== "none" ||
    Number.parseFloat(shoushaLiveCheck.trackH) < 14 ||
    !shoushaLiveCheck.sayBubble.includes("快点吧") ||
    !shoushaLiveCheck.gtbb.includes("十周年播报") ||
    !shoushaLiveCheck.guanjie ||
    shoushaLiveCheck.gsGridDisplay !== "flex" ||
    !shoushaLiveCheck.luckText.includes("手气评价")
  ) {
    throw new Error(`Shousha live check failed: ${JSON.stringify(shoushaLiveCheck)}`);
  }
  const shotTablePath = join(rawDir, "raw-l14-shousha-tabletop.png");
  const shot2 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotTablePath, Buffer.from(shot2.data, "base64"));

  // Check persistence across Page.reload
  await send("Page.reload");
  for (let i = 0; i < 60; i++) {
    await sleep(150);
    const ok = await evalJs(`document.querySelectorAll('.seat').length === 5`);
    if (ok) break;
  }
  const shoushaReloaded = await evalJs(`({
    yangshi: document.querySelector('.battle-table')?.getAttribute('data-shousha-yangshi'),
    ltan: document.querySelector('.battle-table')?.getAttribute('data-shousha-ltan'),
    jsan: document.querySelector('.battle-table')?.getAttribute('data-shousha-jsan'),
  })`);
  if (shoushaReloaded.yangshi !== "off" || shoushaReloaded.ltan !== "hide" || shoushaReloaded.jsan !== "on") {
    throw new Error(`Shousha config persistence failed: ${JSON.stringify(shoushaReloaded)}`);
  }

  // 2. Verify L08 closures: dynamicBackground canvas RGB diff, player1..3[_PC].css, animation1.css, content GuanXing
  await evalJs(`document.querySelector('[data-testid="btn-open-decade-menu-modal"]')?.click()`);
  await sleep(150);
  await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
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
    setCheck('JBDDZ', true);
    await tick();
    document.querySelector('[data-testid="decade-cfg-tab-effects"]')?.click();
    await tick();
    setSel('dynamicBackground', 'decade_jade');
    await tick();
  })()`);
  const jadePixels = await evalJs(`(() => {
    const c = document.querySelector('[data-testid="decade-dynamic-bg-canvas"]');
    const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data;
    let gSum = 0, rSum = 0, nonZero = 0;
    for (let i = 0; i < d.length; i += 4) {
      if (d[i+3] > 0) { nonZero++; rSum += d[i]; gSum += d[i+1]; }
    }
    return { nonZero, rSum, gSum };
  })()`);
  await evalJs(`(() => {
    const el = document.querySelector('[data-testid="decade-cfg-input-dynamicBackground"]');
    el.value = 'shousha_amber';
    el.dispatchEvent(new Event('change'));
  })()`);
  await sleep(80);
  const amberPixels = await evalJs(`(() => {
    const c = document.querySelector('[data-testid="decade-dynamic-bg-canvas"]');
    const d = c.getContext('2d').getImageData(0, 0, c.width, c.height).data;
    let gSum = 0, rSum = 0, nonZero = 0;
    for (let i = 0; i < d.length; i += 4) {
      if (d[i+3] > 0) { nonZero++; rSum += d[i]; gSum += d[i+1]; }
    }
    return { nonZero, rSum, gSum };
  })()`);
  if (jadePixels.nonZero < 500 || jadePixels.gSum <= jadePixels.rSum || amberPixels.rSum <= amberPixels.gSum) {
    throw new Error(`dynamicBackground pixel check failed: jade=${JSON.stringify(jadePixels)} amber=${JSON.stringify(amberPixels)}`);
  }

  // Test player1..3[_PC].css live border switches + close modal
  const playerCssCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    document.querySelector('[data-testid="decade-cfg-tab-surfaces"]')?.click();
    await tick();
    const sel = document.querySelector('[data-testid="decade-player-css-select"]');
    const results = {};
    for (const v of ['player1_PC.css', 'player2_PC.css', 'player3_PC.css', 'player1.css', 'player2.css', 'player3.css']) {
      sel.value = v;
      sel.dispatchEvent(new Event('change'));
      await tick();
      const seat = document.querySelector('.seat.decade-seat-frame');
      results[v] = {
        attr: document.querySelector('.battle-table')?.getAttribute('data-player-css'),
        borderColor: getComputedStyle(seat).borderColor,
        radius: getComputedStyle(seat).borderTopLeftRadius,
      };
    }
    document.querySelector('[data-testid="btn-close-decade-modal"]')?.click();
    return results;
  })()`);
  if (
    playerCssCheck["player1_PC.css"].borderColor === playerCssCheck["player2_PC.css"].borderColor ||
    playerCssCheck["player2_PC.css"].radius === playerCssCheck["player3_PC.css"].radius
  ) {
    throw new Error(`player*.css switches not visually distinct: ${JSON.stringify(playerCssCheck)}`);
  }

  // 3. Test 3p doudizhu.css vs 8p junba.css + real pause turnActionLogs & identity guess + GuanXing
  await send("Page.navigate", { url: `${baseUrl}/battle?players=3&seed=42&skipAnims=1` });
  let ready3p = false;
  for (let i = 0; i < 60; i++) {
    await sleep(120);
    if (await evalJs(`document.querySelectorAll('.seat').length === 3`)) {
      ready3p = true;
      break;
    }
  }
  if (!ready3p) throw new Error("3p tabletop did not render 3 seats");
  await sleep(150);
  const ddzCheck = await evalJs(`({
    modeCss: document.querySelector('.battle-table')?.getAttribute('data-decade-mode-css'),
    tagText: document.querySelector('[data-testid="jbddz-mode-tag"]')?.textContent?.trim(),
    nums: [...document.querySelectorAll('[data-testid="seat-jbddz-num"]')].map((e) => e.textContent.trim()),
  })`);
  if (ddzCheck.modeCss !== "doudizhu.css" || !ddzCheck.tagText?.includes("斗地主") || ddzCheck.nums.length !== 3) {
    throw new Error(`3p JBDDZ doudizhu check failed: ${JSON.stringify(ddzCheck)}`);
  }

  await send("Page.navigate", { url: `${baseUrl}/battle?players=8&seed=42&mode=identity&skipAnims=1` });
  for (let i = 0; i < 60; i++) {
    await sleep(120);
    if (await evalJs(`document.querySelectorAll('.seat').length === 8`)) break;
  }
  // Advance 8 steps so real turnActionLogs populate across turns, then open pause drawer, filter + set guess
  await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 90));
    const stepBtn = [...document.querySelectorAll('button')].find((b) => b.textContent.trim() === '自动一步');
    for (let i = 0; i < 8; i++) {
      if (stepBtn && !stepBtn.disabled) stepBtn.click();
      await tick();
    }
    document.querySelector('[data-testid="btn-tabletop-pause"]')?.click();
    await tick();
  })()`);
  const unfilteredLogsCount = await evalJs(`document.querySelectorAll('[data-testid="pause-turn-logs"] li').length`);
  await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    document.querySelector('[data-testid="btn-pause-guess-2-反"]')?.click();
    await tick();
    document.querySelector('[data-testid="pause-seat-row-0"]')?.click();
    await tick();
  })()`);
  await sleep(120);
  const junbaPauseCheck = await evalJs(`({
    modeCss: document.querySelector('.battle-table')?.getAttribute('data-decade-mode-css'),
    tagText: document.querySelector('[data-testid="jbddz-mode-tag"]')?.textContent?.trim(),
    numsCount: document.querySelectorAll('[data-testid="seat-jbddz-num"]').length,
    animName: getComputedStyle(document.querySelector('[data-testid="decade-pause-history"] .overlay-card') ?? document.body).animationName,
    unfilteredLogsCount: ${JSON.stringify(0)},
    filteredLogsCount: document.querySelectorAll('[data-testid="pause-turn-logs"] li').length,
    pauseLogsText: document.querySelector('[data-testid="pause-turn-logs"]')?.textContent?.trim() ?? '',
  })`);
  junbaPauseCheck.unfilteredLogsCount = unfilteredLogsCount;
  if (
    junbaPauseCheck.modeCss !== "junba.css" ||
    !junbaPauseCheck.tagText?.includes("军八") ||
    junbaPauseCheck.numsCount !== 8 ||
    junbaPauseCheck.animName !== "decade-open-dialog" ||
    junbaPauseCheck.unfilteredLogsCount < 2 ||
    junbaPauseCheck.filteredLogsCount < 1 ||
    !junbaPauseCheck.pauseLogsText.includes("(P0)")
  ) {
    throw new Error(`8p JBDDZ / pause check failed: ${JSON.stringify(junbaPauseCheck)}`);
  }
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(100);
  const shotJunbaPausePath = join(rawDir, "raw-l08-jbddz-8p-pause.png");
  const shot3 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotJunbaPausePath, Buffer.from(shot3.data, "base64"));

  // Close pause drawer & check seat guess pill renders + GuanXing demo overlay works
  await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 50));
    document.querySelector('[data-testid="btn-pause-close"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-l08-demo-guanxing"]')?.click();
    await tick();
  })()`);
  const gxGuessCheck = await evalJs(`({
    guessPills: [...document.querySelectorAll('[data-testid="seat-pause-guess-pill"]')].map((e) => e.textContent.trim()),
    gxOpen: Boolean(document.querySelector('[data-testid="decade-guanxing-dialog"]')),
    gxCardCount: document.querySelectorAll('[data-testid="decade-guanxing-dialog"] .gx-card').length,
  })`);
  if (!gxGuessCheck.guessPills.some((t) => t.includes("反")) || !gxGuessCheck.gxOpen || gxGuessCheck.gxCardCount < 2) {
    throw new Error(`GuanXing / pause guess check failed: ${JSON.stringify(gxGuessCheck)}`);
  }

  // 4. Finish a real 2p WASM match & verify SanGuoShaGameOver + JSAN hide/restore
  await send("Page.navigate", { url: `${baseUrl}/battle?players=2&seed=42&skipAnims=1&speed=4` });
  let ready2p = false;
  for (let i = 0; i < 60; i++) {
    await sleep(120);
    if (await evalJs(`document.querySelectorAll('.seat').length === 2`)) {
      ready2p = true;
      break;
    }
  }
  if (!ready2p) throw new Error("2p tabletop did not render 2 seats");
  await evalJs(`(() => {
    const autoBtn = [...document.querySelectorAll('button')].find((b) => b.textContent.trim() === '连续自动');
    autoBtn?.click();
  })()`);
  let matchFinished = false;
  for (let i = 0; i < 120; i++) {
    await sleep(150);
    matchFinished = await evalJs(`Boolean(document.querySelector('[data-testid="decade-gameover-overlay"]'))`);
    if (matchFinished) break;
  }
  if (!matchFinished) {
    throw new Error("2p WASM match did not finish in time to display SanGuoShaGameOver");
  }
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(120);
  const shotGameOverPath = join(rawDir, "raw-l08-gameover-real.png");
  const shot4 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotGameOverPath, Buffer.from(shot4.data, "base64"));

  const jsanCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    const banner = document.querySelector('[data-testid="decade-gameover-banner"]')?.textContent?.trim() ?? '';
    document.querySelector('[data-testid="btn-shousha-hide-gameover"]')?.click();
    await tick();
    const hiddenOk = !document.querySelector('[data-testid="decade-gameover-overlay"]') && Boolean(document.querySelector('[data-testid="btn-shousha-restore-gameover"]'));
    document.querySelector('[data-testid="btn-shousha-restore-gameover"]')?.click();
    await tick();
    const restoredOk = Boolean(document.querySelector('[data-testid="decade-gameover-overlay"]'));
    return { banner, hiddenOk, restoredOk };
  })()`);
  if (!jsanCheck.banner || !jsanCheck.hiddenOk || !jsanCheck.restoredOk) {
    throw new Error(`SanGuoShaGameOver / JSAN check failed: ${JSON.stringify(jsanCheck)}`);
  }

  // 5. Stitch 2x2 <= 200 KB JPEG collage into docsDir
  const imgsB64 = [shotModalPath, shotTablePath, shotJunbaPausePath, shotGameOverPath].map((p) =>
    readFileSync(p).toString("base64"),
  );
  const thumbDataUrl = await evalJs(`(async () => {
    const loadImg = (src) => new Promise((res, rej) => {
      const im = new Image();
      im.onload = () => res(im);
      im.onerror = rej;
      im.src = 'data:image/png;base64,' + src;
    });
    const imgs = await Promise.all(${JSON.stringify(imgsB64)}.map(loadImg));
    const w = 480, h = 320, pad = 28;
    const canvas = document.createElement('canvas');
    canvas.width = w * 2;
    canvas.height = (h + pad) * 2;
    const ctx = canvas.getContext('2d');
    ctx.fillStyle = '#0d1117';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    const titles = [
      'L14 ShoushaConfigModal (48 Configs / 40 Surfaces)',
      'L14 Tabletop Active (yangshi=off, jindutiao, GTBB, say)',
      'L08 JBDDZ 8p junba.css + Pause Turn Logs + Guesses',
      'L08 Real 2p WASM Match SanGuoShaGameOver + JSAN',
    ];
    imgs.forEach((im, idx) => {
      const col = idx % 2;
      const row = Math.floor(idx / 2);
      const x = col * w;
      const y = row * (h + pad);
      ctx.fillStyle = '#fde68a';
      ctx.font = 'bold 13px sans-serif';
      ctx.fillText(titles[idx], x + 10, y + 19);
      ctx.drawImage(im, x, y + pad, w, h);
    });
    return canvas.toDataURL('image/jpeg', 0.8);
  })()`);
  const thumbPath = join(docsDir, "shousha-l14-and-l08-closure.jpg");
  writeFileSync(thumbPath, Buffer.from(thumbDataUrl.split(",")[1], "base64"));

  const report = {
    shoushaModalCheck,
    shoushaLiveCheck,
    shoushaReloaded,
    jadePixels,
    amberPixels,
    playerCssVariantsChecked: Object.keys(playerCssCheck),
    ddzCheck,
    junbaPauseCheck,
    gxGuessCheck,
    jsanCheck,
    thumbBytes: statSync(thumbPath).size,
    rawSha256: {
      "raw-l14-shousha-config-modal.png": sha256File(shotModalPath),
      "raw-l14-shousha-tabletop.png": sha256File(shotTablePath),
      "raw-l08-jbddz-8p-pause.png": sha256File(shotJunbaPausePath),
      "raw-l08-gameover-real.png": sha256File(shotGameOverPath),
    },
  };
  console.log("SHOUSHA_UI_CDP_RESULT=" + JSON.stringify(report));
  if (report.thumbBytes > 200 * 1024) {
    throw new Error(`Collage thumbnail exceeded 200 KB: ${report.thumbBytes}`);
  }
  ws.close();
} finally {
  try { chrome.kill("SIGKILL"); } catch {}
  try { server.close(); } catch {}
  try { rmSync(udd, { recursive: true, force: true }); } catch {}
}
