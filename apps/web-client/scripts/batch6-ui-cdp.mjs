import { createHash } from "node:crypto";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, extname, join } from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const webClientDir = join(__dirname, "..");
const repoRoot = join(webClientDir, "..", "..");
const distDir = join(webClientDir, "dist");
const evidenceDir = join(repoRoot, "docs", "operations", "evidence", "2026-11-21-ui-l24-l25-l29-l10");
const rawDir = join(repoRoot, "docs", "operations", "evidence-raw", "ui-batch6");
const chromePath = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

mkdirSync(evidenceDir, { recursive: true });
mkdirSync(rawDir, { recursive: true });

const pnpmCmd = process.platform === "win32" ? "pnpm.cmd" : "pnpm";
const buildRes = spawnSync(pnpmCmd, ["run", "build"], { cwd: webClientDir, stdio: "inherit" });
if (buildRes.status !== 0) process.exit(buildRes.status ?? 1);

const mime = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
  ".json": "application/json",
  ".png": "image/png",
  ".jpg": "image/jpeg",
};

const server = createServer((req, res) => {
  const reqPath = (req.url ?? "/").split("?")[0];
  let fullPath = join(distDir, reqPath === "/" ? "index.html" : reqPath);
  try {
    if (!statSync(fullPath).isFile()) fullPath = join(distDir, "index.html");
  } catch {
    fullPath = join(distDir, "index.html");
  }
  res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
  res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
  res.setHeader("Content-Type", mime[extname(fullPath)] ?? "application/octet-stream");
  res.end(readFileSync(fullPath));
});

await new Promise((r) => server.listen(0, "127.0.0.1", r));
const port = server.address().port;
const userDataDir = join("/tmp", `chrome-ui-batch6-${Date.now()}`);
const chromeProc = spawn(
  chromePath,
  [
    "--headless=new",
    "--no-sandbox",
    "--disable-gpu",
    "--remote-debugging-port=0",
    `--user-data-dir=${userDataDir}`,
    "about:blank",
  ],
  { stdio: ["ignore", "pipe", "pipe"] },
);

const wsUrl = await new Promise((resolve, reject) => {
  const timer = setTimeout(() => reject(new Error("Chrome WS timeout")), 10000);
  chromeProc.stderr.on("data", (chunk) => {
    const m = chunk.toString().match(/DevTools listening on (ws:\/\/[^\s]+)/);
    if (m) {
      clearTimeout(timer);
      resolve(m[1]);
    }
  });
});

const require = createRequire(import.meta.url);
const { WebSocket } = require("ws");
const browserWs = new WebSocket(wsUrl);
let msgId = 1;
const pending = new Map();
const rpc = (ws, method, params = {}) =>
  new Promise((resolve, reject) => {
    const id = msgId++;
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
const wireWs = (ws) => {
  ws.on("message", (buf) => {
    const data = JSON.parse(buf.toString());
    if (data.id && pending.has(data.id)) {
      const { resolve, reject } = pending.get(data.id);
      pending.delete(data.id);
      if (data.error) reject(new Error(data.error.message));
      else resolve(data.result);
    }
  });
};
wireWs(browserWs);
await new Promise((r) => browserWs.on("open", r));
const { targetId } = await rpc(browserWs, "Target.createTarget", { url: `http://127.0.0.1:${port}/battle` });
const targetInfo = await rpc(browserWs, "Target.getTargets");
const pageTarget = targetInfo.targetInfos.find((t) => t.targetId === targetId);
const pageWs = new WebSocket(pageTarget.webSocketDebuggerUrl ?? wsUrl.replace(/\/devtools\/browser.*/, `/devtools/page/${targetId}`));
wireWs(pageWs);
await new Promise((r) => pageWs.on("open", r));

const send = (method, params = {}) => rpc(pageWs, method, params);
const evalJs = async (expression) => {
  const r = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  if (r.exceptionDetails) {
    throw new Error(`Eval error: ${JSON.stringify(r.exceptionDetails.exception?.description || r.exceptionDetails)}`);
  }
  return r.result?.value;
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

try {
  await send("Page.enable");
  await send("Runtime.enable");
  await send("Emulation.setDeviceMetricsOverride", { width: 1440, height: 1080, deviceScaleFactor: 1, mobile: false });
  await send("Page.navigate", { url: `http://127.0.0.1:${port}/battle?players=5&seed=42&skipAnims=1` });

  for (let i = 0; i < 60; i++) {
    await sleep(150);
    if (await evalJs(`document.querySelectorAll('.seat').length === 5 && document.querySelectorAll('.candidate').length > 0`)) break;
  }

  // 0. Capture L27 e.css GeneralSelectPanel BEFORE selecting candidate dismisses it
  const l27PreSelectCheck = await evalJs(`(() => {
    const gs = document.querySelector('[data-testid="general-select-panel"]');
    const header = document.querySelector('[data-testid="l27-ecss-header"]');
    const track = gs?.querySelector('.l27-jindutiao-track');
    return {
      hasEcssClass: Boolean(gs?.classList.contains('l27-ecss-styled')),
      headerText: header?.textContent?.trim() ?? '',
      hasTrack: Boolean(track),
      candidateCount: document.querySelectorAll('.candidate').length,
    };
  })()`);
  if (!l27PreSelectCheck.hasEcssClass || !l27PreSelectCheck.headerText || !l27PreSelectCheck.hasTrack) {
    throw new Error(`L27 e.css pre-select check failed: ${JSON.stringify(l27PreSelectCheck)}`);
  }
  await evalJs(`document.querySelector('[data-testid="general-select-panel"]')?.scrollIntoView({ block: 'start' })`);
  await sleep(100);
  const shot0Path = join(rawDir, "raw-l27-ecss-select.png");
  writeFileSync(shot0Path, Buffer.from((await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false })).data, "base64"));

  await evalJs(`document.querySelector('.candidate')?.click()`);
  await sleep(180);

  // 1. L09 44 configs + real subpage switch + QhlyMiniWin + L20
  await evalJs(`document.querySelector('[data-testid="btn-open-qhly-modal"]')?.click()`);
  await sleep(120);
  const l09Batch6Check = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    document.querySelector('[data-testid="btn-qhly-tab-configs"]')?.click();
    await tick();
    const setChk = (k, v) => {
      const el = document.querySelector('[data-testid="qhly-cfg-input-' + k + '"]');
      el.checked = v;
      el.dispatchEvent(new Event('change'));
    };
    document.querySelector('[data-testid="qhly-cfg-tab-window"]')?.click();
    await tick();
    setChk('qhly_skinButton', true);
    await tick();
    setChk('qhly_smallwiningame', true);
    await tick();
    setChk('qhly_skillingame', true);
    await tick();
    setChk('qhly_recordWin', true);
    await tick();
    setChk('qhly_editmode', true);
    await tick();
    setChk('qhly_dragButton', true);
    await tick();
    document.querySelector('[data-testid="qhly-cfg-tab-decade"]')?.click();
    await tick();
    setChk('qhly_dom2image', true);
    await tick();
    document.querySelector('[data-testid="btn-qhly-tab-inspect"]')?.click();
    await tick();
    const subTexts = {};
    for (const sp of ['skill', 'skin', 'config', 'introduce']) {
      document.querySelector('[data-testid="btn-qhly-subpage-' + sp + '"]')?.click();
      await tick();
      subTexts[sp] = document.querySelector('[data-testid="qhly-subpage-body"]')?.textContent?.trim() ?? '';
    }
    document.querySelector('[data-testid="btn-qhly-dom2image"]')?.click();
    await tick();
    return {
      resolverText: document.querySelector('[data-testid="qhly-asset-resolver-box"]')?.textContent ?? '',
      recordWinText: document.querySelector('[data-testid="qhly-record-win-box"]')?.textContent ?? '',
      dom2ImgText: document.querySelector('[data-testid="qhly-dom2image-status"]')?.textContent ?? '',
      editBox: Boolean(document.querySelector('[data-testid="qhly-editmode-box"]')),
      subTexts,
    };
  })()`);
  if (
    !l09Batch6Check.recordWinText.includes("胜") ||
    !l09Batch6Check.dom2ImgText.includes("已生成") ||
    !l09Batch6Check.editBox ||
    !l09Batch6Check.subTexts.skill.includes("[技能页]") ||
    !l09Batch6Check.subTexts.skin.includes("[皮肤页]") ||
    !l09Batch6Check.subTexts.config.includes("[配置页]") ||
    !l09Batch6Check.subTexts.introduce.includes("[简介页]")
  ) {
    throw new Error(`L09 Batch 6 closure check failed: ${JSON.stringify(l09Batch6Check)}`);
  }
  await evalJs(`document.querySelector('[data-testid="btn-close-qhly-modal"]')?.click()`);
  await sleep(80);
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(100);
  const miniWinCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 45));
    document.querySelector('[data-testid="btn-seat-qhly-skin"]')?.click();
    await tick();
    const miniOpen = Boolean(document.querySelector('[data-testid="qhly-mini-skin-win"]'));
    return {
      miniOpen,
      ingameSkills: document.querySelectorAll('[data-testid="seat-qhly-ingame-skills"]').length,
    };
  })()`);
  if (!miniWinCheck.miniOpen || miniWinCheck.ingameSkills < 1) {
    throw new Error(`L09 miniWin / ingameSkills check failed: ${JSON.stringify(miniWinCheck)}`);
  }
  const shot1Path = join(rawDir, "raw-l09-l20-closure.png");
  writeFileSync(shot1Path, Buffer.from((await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false })).data, "base64"));
  await evalJs(`document.querySelector('[data-testid="btn-close-qhly-mini"]')?.click()`);
  await sleep(60);

  // 2. L24 皮肤切换 (14 configs + 10 surfaces + 22 hooks + Decision 38)
  await evalJs(`document.querySelector('[data-testid="btn-open-skinswitch-modal"]')?.click()`);
  await sleep(120);
  const l24Check = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    document.querySelector('[data-testid="btn-skinswitch-tab-configs"]')?.click();
    await tick();
    const rowCount = document.querySelectorAll('[data-testid^="skinswitch-cfg-row-"]').length;
    const surfCount = document.querySelectorAll('[data-testid^="skinswitch-surface-"]').length;
    document.querySelector('[data-testid="skinswitch-surface-light-modal.css"]')?.click();
    await tick();
    const lightAttr = document.querySelector('[data-testid="skinswitch-modal-card"]')?.getAttribute('data-skinswitch-surface');
    const setSsChk = async (k, v) => {
      const el = document.querySelector('[data-testid="skinswitch-cfg-input-' + k + '"]');
      el.checked = v;
      el.dispatchEvent(new Event('change'));
      await tick();
    };
    await setSsChk('showEditMenu', true);
    await setSsChk('showPreviewDynamicMenu', true);
    await setSsChk('hideHuanFu', false);
    await setSsChk('cugDynamicBg', true);
    document.querySelector('[data-testid="btn-skinswitch-gxnr"]')?.click();
    await tick();
    document.querySelector('[data-testid="skinswitch-cfg-input-genDynamicSkin"]')?.click();
    await tick();
    document.querySelector('[data-testid="skinswitch-cfg-input-genDyTempFile"]')?.click();
    await tick();
    const ta = document.querySelector('[data-testid="skinswitch-cfg-json-textarea"]');
    ta.value = '{"badSs":1}';
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    document.querySelector('[data-testid="btn-skinswitch-import-json"]')?.click();
    await tick();
    return {
      rowCount,
      surfCount,
      lightAttr,
      gxnrDialog: Boolean(document.querySelector('[data-testid="skinswitch-gxnr-dialog"]')),
      snippet: Boolean(document.querySelector('[data-testid="skinswitch-generated-snippet"]')),
      templateSnippet: Boolean(document.querySelector('[data-testid="skinswitch-generated-template"]')),
      rejectMsg: document.querySelector('[data-testid="skinswitch-cfg-msg"]')?.textContent ?? '',
    };
  })()`);
  if (
    l24Check.rowCount !== 14 ||
    l24Check.surfCount !== 10 ||
    l24Check.lightAttr !== "light-modal.css" ||
    !l24Check.gxnrDialog ||
    !l24Check.snippet ||
    !l24Check.templateSnippet ||
    !l24Check.rejectMsg.includes("导入被拒绝")
  ) {
    throw new Error(`L24 SkinSwitch check failed: ${JSON.stringify(l24Check)}`);
  }
  const shot2Path = join(rawDir, "raw-l24-skinswitch-14cfg-10surf.png");
  writeFileSync(shot2Path, Buffer.from((await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false })).data, "base64"));
  await evalJs(`document.querySelector('[data-testid="btn-close-skinswitch-modal"]')?.click()`);
  await sleep(80);

  // 3. L25 祖安武将 (68 configs + 40 hooks + L10/L27-29 toggles)
  await evalJs(`document.querySelector('[data-testid="btn-tabletop-zuan-panel"]')?.click()`);
  await sleep(120);
  const l25Check = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const keys = [];
    for (const t of ['info_dividers', 'gameplay_modes', 'visual_fx', 'interaction_audio']) {
      document.querySelector('[data-testid="zuan-tab-' + t + '"]')?.click();
      await tick();
      for (const r of document.querySelectorAll('[data-testid^="zuan-cfg-row-"]')) {
        keys.push(r.getAttribute('data-testid').replace('zuan-cfg-row-', ''));
      }
    }
    const ta = document.querySelector('[data-testid="zuan-cfg-json-textarea"]');
    ta.value = '{"badZuan":true}';
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    document.querySelector('[data-testid="btn-zuan-import-json"]')?.click();
    await tick();
    const rejectMsg = document.querySelector('[data-testid="zuan-cfg-msg"]')?.textContent ?? '';

    document.querySelector('[data-testid="zuan-tab-hooks_patch"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-zuan-hook-_bingleBigScreen"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-zuan-hook-_zuanboss"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-zuan-hook-_haoyunlai"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-zuan-hook-_relifeskill"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-zuan-hook-_seadreammodeskill"]')?.click();
    await tick();
    return {
      cfgCount: keys.length,
      uniqueCfg: new Set(keys).size,
      rejectMsg,
      hookCount: document.querySelectorAll('[data-testid^="btn-zuan-hook-"]').length,
    };
  })()`);
  if (l25Check.cfgCount !== 68 || l25Check.uniqueCfg !== 68 || l25Check.hookCount !== 40 || !l25Check.rejectMsg.includes("导入被拒绝")) {
    throw new Error(`L25 Zuan modal check failed: ${JSON.stringify(l25Check)}`);
  }
  const shot3Path = join(rawDir, "raw-l25-zuan-68cfg-40hooks.png");
  writeFileSync(shot3Path, Buffer.from((await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false })).data, "base64"));
  await evalJs(`document.querySelector('[data-testid="btn-close-zuan-panel"]')?.click()`);
  await sleep(100);

  // 4. L27 e.css + L28 thunder.css + L29 + L10 live tabletop (advance to actionable hand card for cardPhantom)
  await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 70));
    const stepBtn = [...document.querySelectorAll('button')].find((b) => b.textContent.trim() === '自动一步');
    for (let i = 0; i < 12; i++) {
      if (document.querySelector('.strip .card:not([disabled])')) break;
      if (stepBtn && !stepBtn.disabled) stepBtn.click();
      await tick();
    }
  })()`);
  await evalJs(`document.querySelector('[data-testid="btn-l10-emoji"]')?.click()`);
  await sleep(80);
  const tabletopBatch6Check = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    const actionable = document.querySelector('.strip .card:not([disabled])');
    actionable?.click();
    await tick();
    const r = (el) => el?.getBoundingClientRect() ?? null;
    const hit = (a, b) => Boolean(a && b && !(a.right <= b.left + 0.5 || a.left >= b.right - 0.5 || a.bottom <= b.top + 0.5 || a.top >= b.bottom - 0.5));
    const cz = r(document.querySelector('[data-testid="tabletop-center-zone"]'));
    const actionBar = r(document.querySelector('.action-bar'));
    const tipBarEl = document.querySelector('.hand-tip-bar');
    const tipBar = r(tipBarEl);
    const tipBarOverflow = Boolean(!tipBarEl || tipBarEl.scrollWidth > tipBarEl.clientWidth + 1);
    const tipChildren = tipBarEl ? [...tipBarEl.children].map((el) => r(el)).filter((b) => b && b.width > 0) : [];
    let tipBarChildOverlap = false;
    for (let i = 0; i < tipChildren.length; i++) {
      for (let j = i + 1; j < tipChildren.length; j++) {
        if (hit(tipChildren[i], tipChildren[j])) tipBarChildOverlap = true;
      }
    }
    const confirmBtn = r([...document.querySelectorAll('.action-bar button')][0]);
    const centerZoneClipped = Boolean(
      !cz ||
      !actionBar ||
      !tipBar ||
      !confirmBtn ||
      tipBarOverflow ||
      tipBarChildOverlap ||
      actionBar.bottom > cz.bottom + 1 ||
      confirmBtn.bottom > cz.bottom + 1 ||
      tipBar.bottom > cz.bottom + 1 ||
      tipBar.top < cz.top - 1 ||
      tipBar.left < cz.left - 1 ||
      tipBar.right > cz.right + 1
    );

    let skinOverlapsName = false;
    let skinOverlapsGlyph = false;
    let huanfuOverlapsGlyph = false;
    let huanfuOverlapsIdentity = false;
    let skinOverlapsHuanfu = false;
    let equipOccludesHeader = false;
    let clippedSeatTags = 0;
    let stageClipPathCount = 0;
    let portraitClipPathCount = 0;
    let huanfuVisibleCount = 0;
    const seatOverlapDebug = [];
    for (const [idx, seat] of [...document.querySelectorAll('.seat')].entries()) {
      const stageEl = seat.querySelector('.seat-portrait-stage');
      if (stageEl && getComputedStyle(stageEl).clipPath !== 'none') {
        stageClipPathCount++;
        seatOverlapDebug.push({ idx, kind: 'stage-clip-path', clipPath: getComputedStyle(stageEl).clipPath });
      }
      const portraitEl = seat.querySelector('.general-slot:not(.deputy-hidden-slot) .portrait');
      if (portraitEl && getComputedStyle(portraitEl).clipPath !== 'none') {
        portraitClipPathCount++;
      }
      const nameRect = r(seat.querySelector('.portrait-caption .name'));
      const glyphRect = r(seat.querySelector('.general-slot:not(.deputy-hidden-slot) .no-image-glyph'));
      const idRect = r(seat.querySelector('.seat-identity-stamp'));
      const skinBtn = seat.querySelector('[data-testid="btn-seat-qhly-skin"]');
      const huanfuBtn = seat.querySelector('[data-testid="btn-seat-skinswitch-huanfu"]');
      const skinRect = skinBtn && getComputedStyle(skinBtn).display !== 'none' ? r(skinBtn) : null;
      const huanfuRect = huanfuBtn && getComputedStyle(huanfuBtn).display !== 'none' ? r(huanfuBtn) : null;
      if (huanfuRect && huanfuRect.width > 0) huanfuVisibleCount++;
      if (hit(skinRect, nameRect)) {
        skinOverlapsName = true;
        seatOverlapDebug.push({ idx, kind: 'skin-name', name: seat.querySelector('.portrait-caption .name')?.textContent, skinRect, nameRect });
      }
      if (hit(skinRect, glyphRect)) {
        skinOverlapsGlyph = true;
        seatOverlapDebug.push({ idx, kind: 'skin-glyph', skinRect, glyphRect });
      }
      if (hit(huanfuRect, glyphRect)) {
        huanfuOverlapsGlyph = true;
        seatOverlapDebug.push({ idx, kind: 'huanfu-glyph', huanfuRect, glyphRect });
      }
      if (hit(huanfuRect, idRect)) huanfuOverlapsIdentity = true;
      if (hit(skinRect, huanfuRect)) {
        skinOverlapsHuanfu = true;
        seatOverlapDebug.push({ idx, kind: 'skin-huanfu', skinRect, huanfuRect });
      }
      const hdrRect = r(seat.querySelector('.seat-overlay-header'));
      const eqRect = r(seat.querySelector('.equip-zone'));
      if (hdrRect && eqRect && hdrRect.height > 0 && eqRect.height > 0 && hdrRect.bottom > eqRect.top + 1) {
        equipOccludesHeader = true;
      }
      for (const child of seat.querySelectorAll('.seat-tags > *')) {
        if (getComputedStyle(child).display === 'none') continue;
        const cr = r(child);
        const boxEl = child.closest('.seat-bottom-overlays') ?? child.closest('.seat-footer');
        const br = r(boxEl);
        if (!cr || cr.width < 2 || cr.height < 4 || !br || cr.top < br.top - 0.5 || cr.bottom > br.bottom + 0.5 || cr.left < br.left - 0.5 || cr.right > br.right + 0.5) {
          clippedSeatTags++;
          seatOverlapDebug.push({ idx, kind: 'clipped-tag', text: child.textContent?.trim(), cr, br });
        }
      }
    }

    let foldedNameClipped = false;
    const cardEls = [...document.querySelectorAll('.strip > button.card')];
    for (let i = 0; i < cardEls.length - 1; i++) {
      const nRect = r(cardEls[i].querySelector('.card-name'));
      const nextRect = r(cardEls[i + 1]);
      if (nRect && nextRect && nRect.right > nextRect.left + 0.5) foldedNameClipped = true;
    }

    const phantomEl = document.querySelector('[data-testid="l10-card-phantom"]');
    const phantomRect = r(phantomEl);
    const l29Btn = document.querySelector('[data-testid="btn-l29-main1-shengjian"]');

    return {
      l28Thunder: Boolean(document.querySelector('[data-testid="l28-thunder-yuqi"]')),
      l29Main1Btn: Boolean(l29Btn?.classList.contains('is-longxue-special')),
      l10Shichangshi: Boolean(document.querySelector('[data-testid="l10-shichangshi-badge"]')),
      l10Emoji: document.querySelector('[data-testid="l10-emoji-rebate"]')?.textContent ?? '',
      l10Phantom: phantomEl?.textContent?.trim() ?? '',
      l10PhantomCardHeight: Math.round(phantomRect?.height ?? 0),
      l24QuickEdit: Boolean(document.querySelector('[data-testid="btn-skinswitch-quick-edit"]')),
      l24QuickPreview: Boolean(document.querySelector('[data-testid="btn-skinswitch-quick-preview"]')),
      l25BingleBanner: Boolean(document.querySelector('[data-testid="zuan-bingle-big-screen"]')),
      l25RelifeSeat: Boolean(document.querySelector('[data-testid="seat-zuan-relife"]')),
      l25SeaDream: document.querySelector('.battle-table')?.getAttribute('data-zuan-seadream'),
      cugBgAttr: document.querySelector('.battle-table')?.getAttribute('data-skinswitch-cug-bg'),
      geomAudit: {
        centerZoneClipped,
        skinOverlapsName,
        skinOverlapsGlyph,
        huanfuOverlapsGlyph,
        huanfuOverlapsIdentity,
        skinOverlapsHuanfu,
        equipOccludesHeader,
        clippedSeatTags,
        stageClipPathCount,
        portraitClipPathCount,
        foldedNameClipped,
        huanfuVisibleCount,
        seatOverlapDebug,
      },
    };
  })()`);
  if (
    !tabletopBatch6Check.l28Thunder ||
    !tabletopBatch6Check.l29Main1Btn ||
    !tabletopBatch6Check.l10Shichangshi ||
    !tabletopBatch6Check.l10Emoji ||
    !tabletopBatch6Check.l10Phantom.includes("幻影出牌") ||
    tabletopBatch6Check.l10PhantomCardHeight < 90 ||
    !tabletopBatch6Check.l24QuickEdit ||
    !tabletopBatch6Check.l24QuickPreview ||
    !tabletopBatch6Check.l25BingleBanner ||
    !tabletopBatch6Check.l25RelifeSeat ||
    tabletopBatch6Check.l25SeaDream !== "on" ||
    tabletopBatch6Check.cugBgAttr !== "on" ||
    tabletopBatch6Check.geomAudit.centerZoneClipped ||
    tabletopBatch6Check.geomAudit.skinOverlapsName ||
    tabletopBatch6Check.geomAudit.skinOverlapsGlyph ||
    tabletopBatch6Check.geomAudit.huanfuOverlapsGlyph ||
    tabletopBatch6Check.geomAudit.huanfuOverlapsIdentity ||
    tabletopBatch6Check.geomAudit.skinOverlapsHuanfu ||
    tabletopBatch6Check.geomAudit.equipOccludesHeader ||
    tabletopBatch6Check.geomAudit.clippedSeatTags > 0 ||
    tabletopBatch6Check.geomAudit.stageClipPathCount > 0 ||
    tabletopBatch6Check.geomAudit.portraitClipPathCount < 5 ||
    tabletopBatch6Check.geomAudit.foldedNameClipped ||
    tabletopBatch6Check.geomAudit.huanfuVisibleCount < 5
  ) {
    throw new Error(`Batch 6 tabletop check failed: ${JSON.stringify(tabletopBatch6Check)}`);
  }
  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(80);
  const shot4Path = join(rawDir, "raw-l27-l28-l29-l10-tabletop.png");
  writeFileSync(shot4Path, Buffer.from((await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false })).data, "base64"));

  const thumbPath = join(evidenceDir, "ui-batch6-l24-l25-l29-l10.jpg");
  const pyRes = spawnSync("python3", [
    "-c",
    `
from PIL import Image, ImageDraw
imgs = [
    ("L27 e.css GeneralSelectPanel", "${shot0Path}"),
    ("L09 44 Configs + SubPages + MiniWin + L20", "${shot1Path}"),
    ("L24 SkinSwitch 14 Configs / 10 Surfaces", "${shot2Path}"),
    ("L25 Zuan 68 Configs / 40 Hooks + L10/L27-29", "${shot3Path}"),
    ("L28/L29 main1/L10 CardFold+Phantom/L24 [动皮]", "${shot4Path}"),
]
cw, ch, th = 480, 320, 24
canvas = Image.new("RGB", (cw * 2, (ch + th) * 3), (16, 18, 22))
draw = ImageDraw.Draw(canvas)
for idx, (title, path) in enumerate(imgs):
    x, y = (idx % 2) * cw, (idx // 2) * (ch + th)
    draw.text((x + 10, y + 6), title, fill=(250, 204, 21))
    im = Image.open(path).convert("RGB").resize((cw, ch), Image.Resampling.LANCZOS)
    canvas.paste(im, (x, y + th))
canvas.save("${thumbPath}", "JPEG", quality=80, optimize=True)
`,
  ]);
  if (pyRes.status !== 0) throw new Error(`PIL collage failed: ${pyRes.stderr?.toString()}`);

  const sha256Of = (p) => createHash("sha256").update(readFileSync(p)).digest("hex");
  const summary = {
    l27PreSelectCheck,
    l09Batch6Check,
    miniWinCheck,
    l24Check,
    l25Check,
    tabletopBatch6Check,
    thumbBytes: statSync(thumbPath).size,
    rawSha256: {
      "raw-l27-ecss-select.png": sha256Of(shot0Path),
      "raw-l09-l20-closure.png": sha256Of(shot1Path),
      "raw-l24-skinswitch-14cfg-10surf.png": sha256Of(shot2Path),
      "raw-l25-zuan-68cfg-40hooks.png": sha256Of(shot3Path),
      "raw-l27-l28-l29-l10-tabletop.png": sha256Of(shot4Path),
    },
  };
  console.log("UI_BATCH6_CDP_RESULT=" + JSON.stringify(summary));
} finally {
  try { browserWs.close(); } catch {}
  try { pageWs.close(); } catch {}
  chromeProc.kill("SIGKILL");
  server.close();
}
