// Verify Batch UI-5 (L14 final 33 + L20 37 + L09 102) end-to-end in headless Chrome.
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
const rawDir = resolve(repoRoot, "docs/operations/evidence-raw/ui-batch5");
const docsDir = resolve(repoRoot, "docs/operations/evidence/2026-11-21-ui-l14-l20-l09");
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
const cdpPort = 9498;
const udd = mkdtempSync(join(tmpdir(), "ui-batch5-cdp-"));
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

  // 1. Open 5p battle & verify L14 remaining 33 closures (ZLLT detail crop, qiDongYe, XPJ, xindchoose_type, charactermenu, jindutiaotuoguan, mobile/PC CSS switch)
  await send("Page.navigate", { url: `${baseUrl}/battle?players=5&seed=42&mode=identity&skipAnims=1` });
  for (let i = 0; i < 60; i++) {
    await sleep(150);
    if (await evalJs(`document.querySelectorAll('.seat').length === 5`)) break;
  }
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(150);

  // Open Shousha Character Detail Card by clicking seat guanjie badge & verify ZLLT outcrop toggle
  await evalJs(`document.querySelector('[data-testid="seat-guanjie-badge"]')?.click()`);
  await sleep(150);
  const zlltDetailCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    const card = document.querySelector('[data-testid="shousha-char-detail-card"]');
    const beforeAttr = card?.getAttribute('data-zllt-outcrop');
    document.querySelector('[data-testid="btn-shousha-detail-toggle-zllt"]')?.click();
    await tick();
    const afterAttr = card?.getAttribute('data-zllt-outcrop');
    return { modalOpen: Boolean(card), beforeAttr, afterAttr };
  })()`);
  if (!zlltDetailCheck.modalOpen || zlltDetailCheck.beforeAttr === zlltDetailCheck.afterAttr) {
    throw new Error(`ZLLT detail crop check failed: ${JSON.stringify(zlltDetailCheck)}`);
  }
  const shotZlltPath = join(rawDir, "raw-l14-char-zllt-detail.png");
  const s1 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotZlltPath, Buffer.from(s1.data, "base64"));
  await evalJs(`document.querySelector('[data-testid="btn-close-shousha-detail"]')?.click()`);
  await sleep(80);

  // Configure qiDongYe, XPJ, yangshi=off, xindchoose_type=old, jindutiaotuoguan=true + jindutiaoST=80
  await evalJs(`document.querySelector('[data-testid="btn-open-shousha-menu-modal"]')?.click()`);
  await sleep(120);
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
    setSel('jindutiaoST', '80');
    setCheck('jindutiaotuoguan', true);
    await tick();
    document.querySelector('[data-testid="shousha-cfg-tab-appearance"]')?.click();
    await tick();
    setSel('yangshi', 'off');
    setSel('XPJ', 'on');
    setSel('qiDongYe', 'on');
    setSel('xindmenu', 'shizhounian');
    await tick();
    document.querySelector('[data-testid="shousha-cfg-tab-select"]')?.click();
    await tick();
    setSel('xindchoose_type', 'old');
    await tick();
    document.querySelector('[data-testid="btn-close-shousha-modal"]')?.click();
  })()`);
  await sleep(120);

  const l14QiDongOld = await evalJs(`({
    surfaces: document.querySelector('.battle-table')?.getAttribute('data-shousha-active-surfaces') ?? '',
    xpjText: document.querySelector('[data-testid="shousha-xpj-badge"]')?.textContent?.trim() ?? '',
    chooseRadius: getComputedStyle(document.querySelector('[data-testid="general-select-panel"]')).borderTopLeftRadius,
  })`);
  if (
    !l14QiDongOld.surfaces.includes("qidongye/layout_old.css") ||
    !l14QiDongOld.surfaces.includes("character/main2.css") ||
    !l14QiDongOld.xpjText.includes("配件·原版") ||
    l14QiDongOld.chooseRadius !== "4px"
  ) {
    throw new Error(`L14 qiDongYe/XPJ/chooseType check failed: ${JSON.stringify(l14QiDongOld)}`);
  }

  // Wait for jindutiaoST fast timer to trigger tuoguan indicator
  let tuoguanOk = false;
  for (let i = 0; i < 65; i++) {
    await sleep(100);
    if (await evalJs(`Boolean(document.querySelector('[data-testid="shousha-tuoguan-indicator"]'))`)) {
      tuoguanOk = true;
      break;
    }
  }
  if (!tuoguanOk) throw new Error("jindutiaotuoguan + jindutiaoST timer indicator did not appear");
  const shotXpjPath = join(rawDir, "raw-l14-xpj-qidongye-choose.png");
  const s2 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotXpjPath, Buffer.from(s2.data, "base64"));

  // Check mobile resize surface switching (card/main2.css + lbtn/main2.css + skill/main2.css vs _window.css)
  await send("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 1, mobile: true });
  await evalJs(`window.dispatchEvent(new Event('resize'))`);
  await sleep(100);
  const mobileSurfaces = await evalJs(`document.querySelector('.battle-table')?.getAttribute('data-shousha-active-surfaces') ?? ''`);
  await send("Emulation.setDeviceMetricsOverride", { width: 1440, height: 1080, deviceScaleFactor: 1, mobile: false });
  await evalJs(`window.dispatchEvent(new Event('resize'))`);
  await sleep(100);
  if (!mobileSurfaces.includes("lbtn/main2.css") || mobileSurfaces.includes("lbtn/main2_window.css")) {
    throw new Error(`Mobile Shousha surface switch failed: ${mobileSurfaces}`);
  }

  // 2. Verify L20 live events on tabletop (advance steps until damage/kill/card effects appear cleanly)
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(100);
  await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 80));
    const stepBtn = [...document.querySelectorAll('button')].find((b) => b.textContent.trim() === '自动一步');
    for (let i = 0; i < 16; i++) {
      if (stepBtn && !stepBtn.disabled) stepBtn.click();
      await tick();
    }
  })()`);
  await sleep(150);
  const l20Check = await evalJs(`({
    surfacesAttr: document.querySelector('.battle-table')?.getAttribute('data-l20-surfaces') ?? '',
    weapons: document.querySelectorAll('[data-testid="seat-l20-weapon"]').length,
    cardFx: document.querySelectorAll('[data-testid="seat-l20-card-fx"]').length,
    rankSweepSeats: document.querySelectorAll('.seat.has-l20-rank-sweep').length,
  })`);
  if (
    !l20Check.surfacesAttr.includes("extension.css") ||
    !l20Check.surfacesAttr.includes("group.css") ||
    !l20Check.surfacesAttr.includes("mark.css") ||
    l20Check.rankSweepSeats < 1
  ) {
    throw new Error(`L20 surfaces check failed: ${JSON.stringify(l20Check)}`);
  }
  const shotL20Path = join(rawDir, "raw-l20-effects-live.png");
  const s3 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotL20Path, Buffer.from(s3.data, "base64"));

  // 3. Verify L09 千幻聆音 (12 surfaces + all 82 configs + Decision 38 + tabletop skin button + reload persistence)
  await evalJs(`document.querySelector('[data-testid="btn-open-qhly-modal"]')?.click()`);
  await sleep(150);
  const qhlySheetCount = await evalJs(`document.querySelectorAll('[data-testid^="qhly-sheet-"]').length`);
  await evalJs(`document.querySelector('[data-testid="qhly-sheet-newui_shuimo.css"]')?.click()`);
  await sleep(60);
  const shuimoCardAttr = await evalJs(`document.querySelector('[data-testid="qhly-modal-card"]')?.getAttribute('data-qhly-sheet')`);
  await evalJs(`document.querySelector('[data-testid="btn-qhly-tab-configs"]')?.click()`);
  await sleep(120);

  const qhlyModalCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const keys = [];
    for (const tab of ['ui', 'window', 'decade', 'audio', 'theme_tune', 'actions']) {
      document.querySelector('[data-testid="qhly-cfg-tab-' + tab + '"]')?.click();
      await tick();
      for (const row of document.querySelectorAll('[data-testid^="qhly-cfg-row-"]')) {
        keys.push(row.getAttribute('data-testid').replace('qhly-cfg-row-', ''));
      }
    }
    const ta = document.querySelector('[data-testid="qhly-cfg-json-textarea"]');
    ta.value = '{"badQhly":1}';
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    document.querySelector('[data-testid="btn-qhly-import-json"]')?.click();
    await tick();
    const rejectMsg = document.querySelector('[data-testid="qhly-cfg-import-msg"]')?.textContent?.trim() ?? '';

    document.querySelector('[data-testid="qhly-cfg-tab-ui"]')?.click();
    await tick();
    const fsSel = document.querySelector('[data-testid="qhly-cfg-input-qhly_fontsize1"]');
    fsSel.value = '8';
    fsSel.dispatchEvent(new Event('change'));
    await tick();

    document.querySelector('[data-testid="qhly-cfg-tab-window"]')?.click();
    await tick();
    const sbChk = document.querySelector('[data-testid="qhly-cfg-input-qhly_skinButton"]');
    sbChk.checked = true;
    sbChk.dispatchEvent(new Event('change'));
    await tick();
    const kmChk = document.querySelector('[data-testid="qhly-cfg-input-qhly_keymarkopen"]');
    kmChk.checked = true;
    kmChk.dispatchEvent(new Event('change'));
    await tick();
    return {
      cfgCount: keys.length,
      uniqueCfg: new Set(keys).size,
      rejectMsg,
      cardFont: getComputedStyle(document.querySelector('[data-testid="qhly-modal-card"]')).fontSize,
    };
  })()`);
  if (
    qhlySheetCount !== 12 ||
    shuimoCardAttr !== "newui_shuimo.css" ||
    qhlyModalCheck.cfgCount !== 82 ||
    qhlyModalCheck.uniqueCfg !== 82 ||
    !qhlyModalCheck.rejectMsg.includes("导入被拒绝") ||
    Number.parseFloat(qhlyModalCheck.cardFont) < 14
  ) {
    throw new Error(`L09 Qhly modal check failed: ${JSON.stringify({ qhlySheetCount, shuimoCardAttr, qhlyModalCheck })}`);
  }
  const shotQhlyPath = join(rawDir, "raw-l09-qhly-82-configs.png");
  const s4 = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
  writeFileSync(shotQhlyPath, Buffer.from(s4.data, "base64"));

  await evalJs(`document.querySelector('[data-testid="btn-close-qhly-modal"]')?.click()`);
  await sleep(100);
  const tabletopSkinBtnCheck = await evalJs(`(async () => {
    const btns = [...document.querySelectorAll('[data-testid="btn-seat-qhly-skin"]')];
    const beforeLeft = btns[0] ? getComputedStyle(btns[0]).left : null;
    btns[0]?.click();
    await new Promise((r) => setTimeout(r, 40));
    const afterLeft = btns[0] ? getComputedStyle(btns[0]).left : null;
    return { count: btns.length, beforeLeft, afterLeft };
  })()`);
  if (tabletopSkinBtnCheck.count !== 5 || tabletopSkinBtnCheck.beforeLeft === tabletopSkinBtnCheck.afterLeft) {
    throw new Error(`Tabletop qhly skin button check failed: ${JSON.stringify(tabletopSkinBtnCheck)}`);
  }

  // Reload persistence check for L09 + L14
  await send("Page.reload");
  for (let i = 0; i < 60; i++) {
    await sleep(150);
    if (await evalJs(`document.querySelectorAll('.seat').length === 5`)) break;
  }
  const reloadCheck = await evalJs(`({
    qhlySkinBtn: document.querySelector('.battle-table')?.getAttribute('data-qhly-skin-btn'),
    xpj: document.querySelector('.battle-table')?.getAttribute('data-shousha-xpj'),
    skinBtnsRendered: document.querySelectorAll('[data-testid="btn-seat-qhly-skin"]').length,
  })`);
  if (reloadCheck.qhlySkinBtn !== "on" || reloadCheck.xpj !== "on" || reloadCheck.skinBtnsRendered !== 5) {
    throw new Error(`Batch UI-5 reload persistence check failed: ${JSON.stringify(reloadCheck)}`);
  }

  // 4. Generate 2x2 <= 200 KB JPEG collage
  const imgsB64 = [shotZlltPath, shotXpjPath, shotL20Path, shotQhlyPath].map((p) =>
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
      'L14 Char Detail & ZLLT Outcrop Crop Toggle',
      'L14 XPJ + qiDongYe + xindchoose_type + Timer Tuoguan',
      'L20 46 Hooks / 3 CSS Surfaces Live Effects',
      'L09 Qhly 82 Configs + 12 Themes + Skin Button',
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
  const thumbPath = join(docsDir, "ui-batch5-l14-l20-l09.jpg");
  writeFileSync(thumbPath, Buffer.from(thumbDataUrl.split(",")[1], "base64"));

  const report = {
    zlltDetailCheck,
    l14QiDongOld,
    tuoguanOk,
    mobileSurfacesIncludesMain2: mobileSurfaces.includes("lbtn/main2.css"),
    l20Check,
    qhlySheetCount,
    shuimoCardAttr,
    qhlyModalCheck,
    tabletopSkinBtnCheck,
    reloadCheck,
    thumbBytes: statSync(thumbPath).size,
    rawSha256: {
      "raw-l14-char-zllt-detail.png": sha256File(shotZlltPath),
      "raw-l14-xpj-qidongye-choose.png": sha256File(shotXpjPath),
      "raw-l20-effects-live.png": sha256File(shotL20Path),
      "raw-l09-qhly-82-configs.png": sha256File(shotQhlyPath),
    },
  };
  console.log("UI_BATCH5_CDP_RESULT=" + JSON.stringify(report));
  if (report.thumbBytes > 200 * 1024) {
    throw new Error(`Thumbnail exceeded 200 KB: ${report.thumbBytes}`);
  }
  ws.close();
} finally {
  try { chrome.kill("SIGKILL"); } catch {}
  try { server.close(); } catch {}
  try { rmSync(udd, { recursive: true, force: true }); } catch {}
}
