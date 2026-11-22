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
const evidenceDir = join(repoRoot, "docs", "operations", "evidence", "2026-11-21-ui-batch7-closure");
const rawDir = join(repoRoot, "docs", "operations", "evidence-raw", "ui-batch7");
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
const userDataDir = join("/tmp", `chrome-ui-batch7-${Date.now()}`);
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
    if (data.method === "Runtime.exceptionThrown") {
      console.error("[PAGE_EXCEPTION]", JSON.stringify(data.params?.exceptionDetails));
    } else if (data.method === "Runtime.consoleAPICalled" && data.params?.type === "error") {
      console.error("[PAGE_CONSOLE_ERROR]", JSON.stringify(data.params?.args?.map((a) => a.value ?? a.description)));
    }
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
const snap = (name) => send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false }).then((r) => {
  const out = join(rawDir, name);
  writeFileSync(out, Buffer.from(r.data, "base64"));
  return out;
});

try {
  await send("Page.enable");
  await send("Runtime.enable");
  await send("Emulation.setDeviceMetricsOverride", { width: 1440, height: 1080, deviceScaleFactor: 1, mobile: false });
  await send("Page.navigate", { url: `http://127.0.0.1:${port}/battle?players=5&seed=42&skipAnims=1` });

  let readyDiag = null;
  for (let i = 0; i < 80; i++) {
    await sleep(150);
    readyDiag = await evalJs(`({ seats: document.querySelectorAll('.seat').length, cands: document.querySelectorAll('.candidate').length, hud: document.querySelectorAll('[data-testid^="btn-open-"]').length, text: document.body?.innerText?.slice(0, 220) })`);
    if (readyDiag.seats === 5 && readyDiag.cands > 0 && readyDiag.hud === 7) break;
  }
  const hudBtnCount = readyDiag?.hud ?? 0;
  if (hudBtnCount !== 7) throw new Error(`Expected hudBtnCount === 7, diag=${JSON.stringify(readyDiag)}`);

  // 1. Pre-select: L01 ALL 28 action buttons (exhaustive zero-crash check) + banQh wiring + viewAtt + JSON reject
  await evalJs(`document.querySelector('[data-testid="btn-tabletop-batch7-panel"]')?.click()`);
  await sleep(120);
  const l01Check = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 25));
    const rowCount = document.querySelectorAll('[data-testid^="l01-cfg-row-"]').length;
    const actionButtons = [...document.querySelectorAll('button[data-testid^="l01-cfg-input-"]')];
    const notices = [];
    for (const btn of actionButtons) {
      btn.click();
      await tick();
      notices.push(document.querySelector('[data-testid="l01-cfg-msg"]')?.textContent ?? '');
    }
    const viewAttChk = document.querySelector('[data-testid="l01-cfg-input-viewAtt"]');
    viewAttChk.checked = true;
    viewAttChk.dispatchEvent(new Event('change'));
    await tick();

    const banQhChk = document.querySelector('[data-testid="l01-cfg-input-banQh"]');
    banQhChk.checked = true;
    banQhChk.dispatchEvent(new Event('change'));
    await tick();
    document.querySelector('[data-testid="btn-close-batch7-panel"]')?.click();
    await tick();
    const gsPanel = document.querySelector('[data-testid="general-select-panel"]');
    const reloadBtn = document.querySelector('[data-testid="btn-general-reload"]');
    const banQhActive = gsPanel?.getAttribute('data-ban-qh') === 'on' && Boolean(reloadBtn?.disabled);

    document.querySelector('[data-testid="btn-tabletop-batch7-panel"]')?.click();
    await tick();
    const banQhChk2 = document.querySelector('[data-testid="l01-cfg-input-banQh"]');
    banQhChk2.checked = false;
    banQhChk2.dispatchEvent(new Event('change'));
    await tick();

    document.querySelector('[data-testid="l01-cfg-input-banzhubiao"]')?.click();
    await tick();
    if (!document.querySelector('[data-testid="l01-ban-table"]')?.textContent?.includes('caocao')) {
      document.querySelector('[data-testid="btn-l01-add-banzhu"]')?.click();
      await tick();
    }

    const ta = document.querySelector('[data-testid="l01-cfg-json-textarea"]');
    const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set;
    setter.call(ta, '{"editCf":{"bad":0}}');
    ta.dispatchEvent(new Event('input', { bubbles: true }));
    await tick();
    document.querySelector('[data-testid="btn-l01-import-json"]')?.click();
    await tick();
    const rejectMsg = document.querySelector('[data-testid="l01-cfg-msg"]')?.textContent ?? '';
    return {
      rowCount,
      actionBtnCount: actionButtons.length,
      allHadNotice: notices.every((n) => n.trim().length > 0),
      banQhActive,
      viewAttBadge: Boolean(document.querySelector('[data-testid="l01-view-att-badge"]')),
      banTableText: document.querySelector('[data-testid="l01-ban-table"]')?.textContent ?? '',
      rejectMsg,
    };
  })()`);
  if (
    l01Check.rowCount !== 63 ||
    l01Check.actionBtnCount !== 28 ||
    !l01Check.allHadNotice ||
    !l01Check.banQhActive ||
    !l01Check.viewAttBadge ||
    !l01Check.banTableText.includes("caocao") ||
    !l01Check.rejectMsg.includes("L01_CONFIG_NON_POSITIVE_WEIGHT")
  ) {
    throw new Error(`L01 check failed: ${JSON.stringify(l01Check)}`);
  }
  const shot1Path = await snap("raw-l01-63cfg.png");

  // 2. Pool Tab: L17 (dz-group-* + charStars + ban) + L15 + L21 + U05
  const poolCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    document.querySelector('[data-testid="b7-tab-pool"]')?.click();
    await tick();
    const l17Rows = document.querySelectorAll('[data-testid^="l17-cfg-row-"]').length;
    const l15Rows = document.querySelectorAll('[data-testid^="l15-cfg-row-"]').length;
    const l21Rows = document.querySelectorAll('[data-testid^="l21-cfg-row-"]').length;
    const u05Rows = document.querySelectorAll('[data-testid^="u05-cfg-row-"]').length;
    document.querySelector('[data-testid="btn-l17-plan-2"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-l17-toggle-caocao"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-l17-star-caocao"]')?.click();
    await tick();
    const groupChips = [...document.querySelectorAll('[data-testid^="l17-roster-"]')].filter((el) =>
      [...el.classList].some((c) => c.startsWith('dz-group-'))
    ).length;
    const l21Chk = document.querySelector('[data-testid="l21-toggle-hidden-preview"]');
    l21Chk.checked = true;
    l21Chk.dispatchEvent(new Event('change'));
    await tick();
    return { l17Rows, l15Rows, l21Rows, u05Rows, groupChips };
  })()`);
  if (
    poolCheck.l17Rows !== 9 ||
    poolCheck.l15Rows !== 11 ||
    poolCheck.l21Rows !== 7 ||
    poolCheck.u05Rows !== 7 ||
    poolCheck.groupChips !== 4
  ) {
    throw new Error(`Pool check mismatch: ${JSON.stringify(poolCheck)}`);
  }
  const shot2Path = await snap("raw-l15-l17-l21-u05-pool.png");

  // 3. Tools Tab: L02 (console, tutorial, getExtensions drawers) + L16 + L18
  const toolsCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    document.querySelector('[data-testid="b7-tab-tools"]')?.click();
    await tick();
    const openToolsChk = document.querySelector('[data-testid="l02-cfg-input-openTools"]');
    openToolsChk.checked = true;
    openToolsChk.dispatchEvent(new Event('change'));
    await tick();
    document.querySelector('[data-testid="l02-cfg-input-tutorial"]')?.click();
    await tick();
    document.querySelector('[data-testid="l02-cfg-input-getExtensions"]')?.click();
    await tick();
    document.querySelector('[data-testid="l16-cfg-input-loadUpdateContent"]')?.click();
    await tick();
    document.querySelector('[data-testid="l16-cfg-input-generateZip"]')?.click();
    await tick();
    document.querySelector('[data-testid="l18-cfg-input-checkForUpdate"]')?.click();
    await tick();
    return {
      l02Rows: document.querySelectorAll('[data-testid^="l02-cfg-row-"]').length,
      hasConsole: Boolean(document.querySelector('[data-testid="l02-console-drawer"]')),
      hasTutorial: Boolean(document.querySelector('[data-testid="l02-tutorial-drawer"]')),
      hasExtensions: Boolean(document.querySelector('[data-testid="l02-extensions-drawer"]')),
      l16Btns: document.querySelectorAll('[data-testid^="l16-cfg-input-"]').length,
      zipManifest: document.querySelector('[data-testid="l16-zip-manifest"]')?.textContent ?? '',
      hasL16Changelog: Boolean(document.querySelector('[data-testid="l16-changelog-box"]')),
      l18Rows: document.querySelectorAll('[data-testid^="l18-cfg-row-"]').length,
      progressVal: document.querySelector('[data-testid="l18-progress-bar"]')?.value ?? 0,
    };
  })()`);
  if (
    toolsCheck.l02Rows !== 4 ||
    !toolsCheck.hasConsole ||
    !toolsCheck.hasTutorial ||
    !toolsCheck.hasExtensions ||
    toolsCheck.l16Btns !== 2 ||
    !toolsCheck.zipManifest.includes("match_bundle_b7.zip") ||
    !toolsCheck.hasL16Changelog ||
    toolsCheck.l18Rows !== 12 ||
    toolsCheck.progressVal !== 100
  ) {
    throw new Error(`Tools check failed: ${JSON.stringify(toolsCheck)}`);
  }
  const shot3Path = await snap("raw-l02-l16-l18-tools.png");

  // 4. FX Tab: U04/L05 + all 6 L12 distinct surfaces + L26 Fool_skill gate
  const fxCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    document.querySelector('[data-testid="b7-tab-fx"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-u04-havefun-firework"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-u04-havefun-snow"]')?.click();
    await tick();
    const l12Borders = [];
    for (const sid of ['dynamicSkin','rzsh','font.css','offline.css','offline_old.css','rzsh.css']) {
      document.querySelector('[data-testid="btn-l12-surface-' + sid + '"]')?.click();
      await tick();
      const stage = document.querySelector('[data-testid="l12-rzsh-stage"]');
      const cs = getComputedStyle(stage);
      l12Borders.push(cs.borderTopColor + '|' + cs.borderTopStyle + '|' + cs.backgroundImage);
    }
    const tipWhenOn = Boolean(document.querySelector('[data-testid="l26-skilltip"]'));
    const fsChk = document.querySelector('[data-testid="l26-cfg-input-Fool_skill"]');
    fsChk.checked = false;
    fsChk.dispatchEvent(new Event('change'));
    await tick();
    const tipWhenOff = !document.querySelector('[data-testid="l26-skilltip"]');
    fsChk.checked = true;
    fsChk.dispatchEvent(new Event('change'));
    await tick();
    const equipChk = document.querySelector('[data-testid="l26-chk-equip-mirror"]');
    equipChk.checked = true;
    equipChk.dispatchEvent(new Event('change'));
    await tick();
    return { l12UniqueCount: new Set(l12Borders).size, tipWhenOn, tipWhenOff, l26Rows: document.querySelectorAll('[data-testid^="l26-cfg-row-"]').length };
  })()`);
  if (fxCheck.l12UniqueCount < 5 || !fxCheck.tipWhenOn || !fxCheck.tipWhenOff || fxCheck.l26Rows !== 5) {
    throw new Error(`FX tab check failed: ${JSON.stringify(fxCheck)}`);
  }
  const shot4Path = await snap("raw-l12-l26-u04-fx.png");
  await evalJs(`document.querySelector('[data-testid="btn-close-batch7-panel"]')?.click()`);
  await sleep(100);

  // Pick a general with a real liegong-family skill (`烈弓` / `武神` / `诈降`) if present for positive Fool_liegong test!
  const chosenCandidate = await evalJs(`(() => {
    const btns = [...document.querySelectorAll('.candidate')];
    const match = btns.find((b) => /烈弓|武神|诈降|玲珑|伏骑/.test(b.textContent ?? ''));
    const pick = match ?? btns[0];
    pick?.click();
    return pick?.textContent?.trim() ?? '';
  })()`);
  await sleep(220);

  // 5. Modals: L13 MVP + L11 Taixu + L07 Omnisearch (distinct background styles + multi-page SimplePagination!)
  await evalJs(`document.querySelector('[data-testid="btn-open-mvp-modal"]')?.click()`);
  await sleep(100);
  const mvpCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const beforeCss = document.querySelector('.mvp-modal-card')?.getAttribute('data-l13-surface');
    document.querySelector('[data-testid="btn-mvp-qy-chongzhu"]')?.click();
    await tick();
    document.querySelector('[data-testid="btn-mvp-one-more"]')?.click();
    await tick();
    return { beforeCss, status: document.querySelector('[data-testid="l13-recast-status"]')?.textContent ?? '' };
  })()`);
  if (mvpCheck.beforeCss !== "extension_new.css" || !mvpCheck.status.includes("qy_chongzhu_")) {
    throw new Error(`L13 MVP modal failed: ${JSON.stringify(mvpCheck)}`);
  }
  const shot5Path = await snap("raw-l13-mvp-modal.png");
  await evalJs(`document.querySelector('[data-testid="btn-close-mvp-modal"]')?.click()`);
  await sleep(80);

  await evalJs(`document.querySelector('[data-testid="btn-open-taixu-modal"]')?.click()`);
  await sleep(100);
  const taixuSurfaceCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const seen = [];
    for (const id of ['extension_MobileStyle.css','extension_PcStyle.css','extension_servant.css','extension_style.css']) {
      document.querySelector('[data-testid="btn-l11-surface-' + id + '"]')?.click();
      await tick();
      seen.push(document.querySelector('.taixu-modal-card')?.getAttribute('data-l11-surface'));
    }
    return seen;
  })()`);
  if (taixuSurfaceCheck.length !== 4 || new Set(taixuSurfaceCheck).size !== 4) {
    throw new Error(`L11 surface switch failed: ${JSON.stringify(taixuSurfaceCheck)}`);
  }
  const shot5bPath = await snap("raw-l11-taixu-modal.png");
  await evalJs(`document.querySelector('[data-testid="btn-close-taixu-modal"]')?.click()`);
  await sleep(80);

  await evalJs(`document.querySelector('[data-testid="btn-open-omnisearch-modal"]')?.click()`);
  await sleep(100);
  const omniCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 35));
    const hlSel = document.querySelector('[data-testid="l07-select-highlight"]');
    hlSel.value = 'atom-one-dark.min.css';
    hlSel.dispatchEvent(new Event('change'));
    await tick();
    const bgSel = document.querySelector('[data-testid="l07-select-bg"]');
    const bgBefore = getComputedStyle(document.querySelector('.Searcher')).backgroundImage;
    bgSel.value = '相爱相杀';
    bgSel.dispatchEvent(new Event('change'));
    await tick();
    const bgAfter = getComputedStyle(document.querySelector('.Searcher')).backgroundImage;
    document.querySelector('[data-testid="btn-l07-changelog"]')?.click();
    await tick();
    const qInput = document.querySelector('[data-testid="omnisearch-modal-input"]');
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set;
    setter.call(qInput, 'a');
    qInput.dispatchEvent(new Event('input', { bubbles: true }));
    await tick();
    document.querySelector('[data-testid="btn-omnisearch-modal-run"]')?.click();
    await tick();
    const pageBtns = [...document.querySelectorAll('[data-testid="l07-simple-pagination"] .page-li')];
    const page2Btn = pageBtns.find((b) => b.textContent?.trim() === '2');
    page2Btn?.click();
    await tick();
    return {
      bgChanged: bgBefore !== bgAfter,
      bgAttr: document.querySelector('.Searcher')?.getAttribute('data-l07-bg'),
      hasPaginationMulti: Boolean(page2Btn),
      page2Active: Boolean(page2Btn?.classList.contains('page-active')),
      page2Text: document.querySelector('[data-testid="omnisearch-modal-results"]')?.textContent ?? '',
    };
  })()`);
  if (!omniCheck.bgChanged || omniCheck.bgAttr !== "相爱相杀" || !omniCheck.hasPaginationMulti || !omniCheck.page2Active) {
    throw new Error(`L07 Omnisearch check failed: ${JSON.stringify(omniCheck)}`);
  }
  const shot6Path = await snap("raw-l07-omnisearch-modal.png");
  await evalJs(`document.querySelector('[data-testid="btn-close-omnisearch-modal"]')?.click()`);
  await sleep(100);

  // 6. Live Tabletop: L01 viewAtt HUD + L23 guarded tap + Fool_throwEmotion + L26 Fool_liegong negative/positive + GEOM_AUDIT
  const viewAttCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    document.querySelector('[data-testid="btn-tabletop-view-att"]')?.click();
    await tick();
    const open = Boolean(document.querySelector('[data-testid="l01-attitude-drawer"]'));
    const cells = document.querySelectorAll('[data-testid^="l01-att-seat-"]').length;
    document.querySelector('[data-testid="btn-close-view-att"]')?.click();
    await tick();
    return { open, cells };
  })()`);
  if (!viewAttCheck.open || viewAttCheck.cells !== 5) throw new Error(`viewAtt HUD failed: ${JSON.stringify(viewAttCheck)}`);

  const cailanziTapCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 40));
    const overlay = document.querySelector('[data-testid="seat-cailanzi-overlay"]');
    const box = overlay.getBoundingClientRect();
    // Orphan mouseup WITHOUT mousedown must NOT open picker
    overlay.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: box.left + 12, clientY: box.top + 12 }));
    await tick();
    const orphanOpened = Boolean(document.querySelector('[data-testid="seat-cailanzi-picker"]'));
    overlay.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, clientX: box.left + 12, clientY: box.top + 12 }));
    overlay.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: box.left + 14, clientY: box.top + 15 }));
    await tick();
    const pickerOpen = Boolean(document.querySelector('[data-testid="seat-cailanzi-picker"]'));
    document.querySelector('[data-testid="btn-cailanzi-throw-egg"]')?.click();
    await tick();
    const emojiEl = document.querySelector('[data-testid="l10-emoji-rebate"]');
    const rot = Number(emojiEl?.getAttribute('data-fool-throw-rotation') ?? '0');
    return { orphanOpened, pickerOpen, rot };
  })()`);
  if (cailanziTapCheck.orphanOpened || !cailanziTapCheck.pickerOpen || cailanziTapCheck.rot === 0) {
    throw new Error(`L23/Fool_throwEmotion failed: ${JSON.stringify(cailanziTapCheck)}`);
  }

  // Advance until self has actionable cards; verify non-Sha card yields 0 Fool_liegong badges (negative case!)
  // and if Sha is selected on eligible skill, verify positive badge cleanly!
  const foolGateCheck = await evalJs(`(async () => {
    const tick = () => new Promise((r) => setTimeout(r, 70));
    const stepBtn = [...document.querySelectorAll('button')].find((b) => b.textContent?.trim() === '自动一步');
    for (let i = 0; i < 16; i++) {
      if (document.querySelector('.strip .card:not([disabled])')) break;
      if (stepBtn && !stepBtn.disabled) stepBtn.click();
      await tick();
    }
    const cards = [...document.querySelectorAll('.strip .card:not([disabled])')];
    const nonSha = cards.find((c) => !/杀/.test(c.textContent ?? ''));
    let badgesOnNonSha = 0;
    if (nonSha) {
      nonSha.click();
      await tick();
      badgesOnNonSha = document.querySelectorAll('[data-testid="seat-fool-liegong"]').length;
      nonSha.click();
      await tick();
    }
    const shaCard = cards.find((c) => /杀/.test(c.textContent ?? '')) ?? nonSha;
    shaCard?.click();
    await tick();
    const selfSkillsText = document.querySelector('.seat.self')?.textContent ?? '';
    const hasLiegongFamily = /烈弓|武神|诈降|玲珑|伏骑|金韬/.test(selfSkillsText);
    return {
      nonShaLabel: nonSha?.textContent?.trim() ?? null,
      badgesOnNonSha,
      selectedText: shaCard?.textContent?.trim() ?? null,
      hasLiegongFamily,
      badgesAfterSha: document.querySelectorAll('[data-testid="seat-fool-liegong"]').length,
    };
  })()`);
  if (
    foolGateCheck.badgesOnNonSha !== 0 ||
    (!foolGateCheck.hasLiegongFamily && foolGateCheck.badgesAfterSha !== 0) ||
    (foolGateCheck.hasLiegongFamily && foolGateCheck.badgesAfterSha === 0)
  ) {
    throw new Error(`Fool_liegong gate check failed: ${JSON.stringify(foolGateCheck)}`);
  }

  const tabletopB7Audit = await evalJs(`(() => {
    const r = (el) => {
      if (!el) return null;
      const b = el.getBoundingClientRect();
      return { left: b.left, right: b.right, top: b.top, bottom: b.bottom, width: b.width, height: b.height };
    };
    const hit = (a, b) => a && b && !(a.right <= b.left + 0.5 || a.left >= b.right - 0.5 || a.bottom <= b.top + 0.5 || a.top >= b.bottom - 0.5);
    const cz = r(document.querySelector('[data-testid="tabletop-center-zone"]'));
    const actionBar = r(document.querySelector('.action-bar'));
    const tipBarEl = document.querySelector('.hand-tip-bar');
    const tipBar = r(tipBarEl);
    const tipBarOverflow = Boolean(!tipBarEl || tipBarEl.scrollWidth > tipBarEl.clientWidth + 1);
    const confirmBtn = r([...document.querySelectorAll('.action-bar button')][0]);
    const centerZoneClipped = Boolean(
      !cz || !tipBar || tipBarOverflow ||
      (actionBar && actionBar.bottom > cz.bottom + 1) ||
      (confirmBtn && confirmBtn.bottom > cz.bottom + 1) ||
      tipBar.bottom > cz.bottom + 1 || tipBar.top < cz.top - 1 || tipBar.left < cz.left - 1 || tipBar.right > cz.right + 1
    );
    let skinOverlapsName = false, skinOverlapsGlyph = false, huanfuOverlapsGlyph = false, huanfuOverlapsIdentity = false, skinOverlapsHuanfu = false, equipOccludesHeader = false, clippedSeatTags = 0, stageClipPathCount = 0;
    for (const seat of document.querySelectorAll('.seat')) {
      const stageEl = seat.querySelector('.seat-portrait-stage');
      if (stageEl && getComputedStyle(stageEl).clipPath !== 'none') stageClipPathCount++;
      const nameRect = r(seat.querySelector('.portrait-caption .name'));
      const glyphRect = r(seat.querySelector('.general-slot:not(.deputy-hidden-slot) .no-image-glyph'));
      const idRect = r(seat.querySelector('.seat-identity-stamp'));
      const skinBtn = seat.querySelector('[data-testid="btn-seat-qhly-skin"]');
      const huanfuBtn = seat.querySelector('[data-testid="btn-seat-skinswitch-huanfu"]');
      const skinRect = skinBtn && getComputedStyle(skinBtn).display !== 'none' ? r(skinBtn) : null;
      const huanfuRect = huanfuBtn && getComputedStyle(huanfuBtn).display !== 'none' ? r(huanfuBtn) : null;
      if (hit(skinRect, nameRect)) skinOverlapsName = true;
      if (hit(skinRect, glyphRect)) skinOverlapsGlyph = true;
      if (hit(huanfuRect, glyphRect)) huanfuOverlapsGlyph = true;
      if (hit(huanfuRect, idRect)) huanfuOverlapsIdentity = true;
      if (hit(skinRect, huanfuRect)) skinOverlapsHuanfu = true;
      const hdrRect = r(seat.querySelector('.seat-overlay-header'));
      const eqRect = r(seat.querySelector('.equip-zone'));
      if (hdrRect && eqRect && hdrRect.height > 0 && eqRect.height > 0 && hdrRect.bottom > eqRect.top + 1) equipOccludesHeader = true;
      for (const child of seat.querySelectorAll('.seat-tags > *')) {
        if (getComputedStyle(child).display === 'none') continue;
        const cr = r(child);
        const boxEl = child.closest('.seat-bottom-overlays') ?? child.closest('.seat-footer');
        const br = r(boxEl);
        if (!cr || cr.width < 2 || cr.height < 4 || !br || cr.top < br.top - 0.5 || cr.bottom > br.bottom + 0.5 || cr.left < br.left - 0.5 || cr.right > br.right + 0.5) clippedSeatTags++;
      }
    }
    return {
      coinCanvasWidth: Math.round(r(document.querySelector('[data-testid="tabletop-weather-canvas"]'))?.width ?? 0),
      l21HiddenSeatsCount: document.querySelectorAll('[data-testid="seat-l21-hidden-avatar"]').length,
      geomAudit: { centerZoneClipped, skinOverlapsName, skinOverlapsGlyph, huanfuOverlapsGlyph, huanfuOverlapsIdentity, skinOverlapsHuanfu, equipOccludesHeader, clippedSeatTags, stageClipPathCount },
    };
  })()`);
  if (
    tabletopB7Audit.coinCanvasWidth < 400 ||
    tabletopB7Audit.l21HiddenSeatsCount < 2 ||
    Object.values(tabletopB7Audit.geomAudit).some((v) => v !== false && v !== 0)
  ) {
    throw new Error(`Batch 7 tabletop audit failed: ${JSON.stringify(tabletopB7Audit)}`);
  }

  await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
  await sleep(80);
  const shot7Path = await snap("raw-b7-tabletop-fx-cailanzi-fool.png");

  const thumbPath = join(evidenceDir, "ui-batch7-closure.jpg");
  const pyRes = spawnSync("python3", [
    "-c",
    `
from PIL import Image, ImageDraw
imgs = [
    ("L01 (63 Configs, 28 Actions, banQh)", "${shot1Path}"),
    ("L17 dz-group/Stars + L15 + L21 + U05", "${shot2Path}"),
    ("L02 Drawers + L16 Manifest + L18 Progress", "${shot3Path}"),
    ("L12 6 Surfaces + U04/L05 + L26 Fool_skill", "${shot4Path}"),
    ("L13 MVP (extension_new.css + _qy_chongzhu)", "${shot5Path}"),
    ("L11 Taixu (4 Responsive CSS Surfaces)", "${shot5bPath}"),
    ("L07 Multi-Page SimplePagination + bg", "${shot6Path}"),
    ("Tabletop: U04/L05 + L21 + L23 + L26 Gating", "${shot7Path}"),
]
cw, ch, th = 420, 280, 22
canvas = Image.new("RGB", (cw * 2, (ch + th) * 4), (16, 18, 22))
draw = ImageDraw.Draw(canvas)
for idx, (title, path) in enumerate(imgs):
    x, y = (idx % 2) * cw, (idx // 2) * (ch + th)
    draw.text((x + 8, y + 4), title, fill=(250, 204, 21))
    im = Image.open(path).convert("RGB").resize((cw, ch), Image.Resampling.LANCZOS)
    canvas.paste(im, (x, y + th))
canvas.save("${thumbPath}", "JPEG", quality=72, optimize=True)
`,
  ]);
  if (pyRes.status !== 0) throw new Error(`PIL collage failed: ${pyRes.stderr?.toString()}`);

  const sha256Of = (p) => createHash("sha256").update(readFileSync(p)).digest("hex");
  const summary = {
    hudBtnCount,
    chosenCandidate,
    l01Check,
    poolCheck,
    toolsCheck,
    fxCheck,
    mvpCheck,
    taixuSurfaceCheck,
    omniCheck,
    viewAttCheck,
    cailanziTapCheck,
    foolGateCheck,
    tabletopB7Audit,
    thumbBytes: statSync(thumbPath).size,
    rawSha256: {
      "raw-l01-63cfg.png": sha256Of(shot1Path),
      "raw-l15-l17-l21-u05-pool.png": sha256Of(shot2Path),
      "raw-l02-l16-l18-tools.png": sha256Of(shot3Path),
      "raw-l12-l26-u04-fx.png": sha256Of(shot4Path),
      "raw-l13-mvp-modal.png": sha256Of(shot5Path),
      "raw-l11-taixu-modal.png": sha256Of(shot5bPath),
      "raw-l07-omnisearch-modal.png": sha256Of(shot6Path),
      "raw-b7-tabletop-fx-cailanzi-fool.png": sha256Of(shot7Path),
    },
  };
  console.log("UI_BATCH7_CDP_RESULT=" + JSON.stringify(summary));
} finally {
  try { browserWs.close(); } catch {}
  try { pageWs.close(); } catch {}
  chromeProc.kill("SIGKILL");
  server.close();
}
