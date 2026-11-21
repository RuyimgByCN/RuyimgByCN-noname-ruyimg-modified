// Verify 2-8 player desktop, tablet (768x1024 / 1024x768) & mobile (390x844) layout in headless Chrome without height overrides.
import { execFileSync, spawn } from "node:child_process";
import { createServer } from "node:http";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, extname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const webClientDir = resolve(scriptsDir, "..");
const repoRoot = resolve(webClientDir, "../..");
const outDir = resolve(repoRoot, process.env.TABLETOP_EVIDENCE_DIR || "docs/operations/evidence-raw/ui-batch1");
mkdirSync(outDir, { recursive: true });

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
  if (!fp.startsWith(distDir) || !existsSync(fp) || statSync(fp).isDirectory()) fp = resolve(distDir, "index.html");
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
const cdpPort = 9493;
const udd = mkdtempSync(join(tmpdir(), "tabletop-resp-"));
const chrome = spawn(
  chromeBin,
  ["--headless=new", "--no-sandbox", `--user-data-dir=${udd}`, `--remote-debugging-port=${cdpPort}`, "--window-size=1440,1080", "about:blank"],
  { stdio: "ignore" },
);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let msgId = 0;
const pending = new Map();

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
    if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
    return r.result?.value;
  };
  await send("Runtime.enable");
  await send("Page.enable");

  async function verifyCase({ players, width, height, mobile, shotName, checkMobileFit = false }) {
    await send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile });
    const modeQuery = players >= 4 ? "&mode=identity" : "";
    await send("Page.navigate", { url: `${baseUrl}/battle?players=${players}&seed=42${modeQuery}&skipAnims=1` });
    for (let i = 0; i < 60; i++) {
      await sleep(150);
      const ready = await evalJs(`document.querySelectorAll('.seat').length === ${players} && Boolean(document.querySelector('.hand-dock'))`);
      if (ready) break;
    }
    await sleep(250);
    // Scroll natural BattleBoard into view without overriding .battle-table height
    await evalJs(`document.querySelector('.battle-table')?.scrollIntoView({ block: 'start' })`);
    await sleep(120);

    const check = await evalJs(`(() => {
      const ov = (a, b) => {
        const x = Math.min(a.right, b.right) - Math.max(a.left, b.left);
        const y = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
        return x > 2 && y > 2 ? Math.round(x) + 'x' + Math.round(y) : null;
      };
      const seats = [...document.querySelectorAll('.seat')].map((el) => {
        const r = el.getBoundingClientRect();
        return { el, pos: el.getAttribute('data-position'), left: r.left, right: r.right, top: r.top, bottom: r.bottom, w: r.width, h: r.height };
      });
      const hand = document.querySelector('.hand-dock')?.getBoundingClientRect();
      const center = document.querySelector('[data-testid="tabletop-center-zone"]')?.getBoundingClientRect();
      const centerParts = [...document.querySelectorAll('.center-table-cards, .prompt-and-controls')].map((el) => el.getBoundingClientRect());
      const hud = document.querySelector('[data-testid="battle-shousha-table-hud"]')?.getBoundingClientRect();
      const overlaps = [];
      for (let i = 0; i < seats.length; i++) {
        if (seats[i].w < 40 || seats[i].h < 40) overlaps.push('zero-size:' + i);
        for (let j = i + 1; j < seats.length; j++) {
          const hit = ov(seats[i], seats[j]);
          if (hit) overlaps.push(seats[i].pos + '<->' + seats[j].pos + '(' + hit + ')');
        }
        if (hand) {
          const hit = ov(seats[i], hand);
          if (hit) overlaps.push('hand<->' + seats[i].pos + '(' + hit + ')');
        }
        if (center) {
          const hit = ov(seats[i], center);
          if (hit) overlaps.push('center<->' + seats[i].pos + '(' + hit + ')');
        }
        for (const slotEl of seats[i].el.querySelectorAll('.general-slot:not(.deputy-hidden-slot)')) {
          const nm = slotEl.querySelector('.name')?.getBoundingClientRect();
          const gl = slotEl.querySelector('.no-image-glyph')?.getBoundingClientRect();
          if (nm && gl) {
            const hit = ov(nm, gl);
            if (hit) overlaps.push('portrait-name<->glyph@' + seats[i].pos + '(' + hit + ')');
          }
          const rv = slotEl.querySelector('.reveal-pill')?.getBoundingClientRect();
          const ccEl = seats[i].el.querySelector('.seat-card-count');
          if (rv && ccEl && getComputedStyle(ccEl).visibility !== 'hidden') {
            const hit = ov(rv, ccEl.getBoundingClientRect());
            if (hit) overlaps.push('reveal<->count@' + seats[i].pos + '(' + hit + ')');
          }
        }
        const metaEl = seats[i].el.querySelector('.meta');
        if (metaEl && metaEl.getBoundingClientRect().height > 24) {
          overlaps.push('meta-wrap@' + seats[i].pos + '(' + Math.round(metaEl.getBoundingClientRect().height) + 'px)');
        }
      }
      if (hand && center) {
        const hit = ov(center, hand);
        if (hit) overlaps.push('center<->hand(' + hit + ')');
        for (const p of centerParts) {
          const ph = ov(p, hand);
          if (ph) overlaps.push('centerPart<->hand(' + ph + ')');
        }
      }
      const hudTop = hud ? Math.round(hud.top) : -1;
      const handBottom = hand ? Math.round(hand.bottom) : 9999;
      const topbarRect = document.querySelector('.topbar')?.getBoundingClientRect();
      const topbarClipped = Boolean(
        topbarRect && getComputedStyle(document.querySelector('.topbar')).position === 'sticky' &&
        hud && topbarRect.bottom > hud.top + 2
      );
      return {
        players: seats.length,
        stamps: document.querySelectorAll('[data-testid="seat-identity-stamp"]').length,
        overflowX: document.documentElement.scrollWidth > window.innerWidth,
        scrollWidth: document.documentElement.scrollWidth,
        innerWidth: window.innerWidth,
        hudTop,
        handBottom,
        topbarClipped,
        overlaps,
      };
    })()`);
    console.log(`[MATRIX ${players}p ${width}x${height} mobile=${mobile}]`, JSON.stringify(check));
    if (
      check.players !== players ||
      check.stamps !== players ||
      check.overlaps.length > 0 ||
      check.overflowX ||
      check.hudTop < 0 ||
      check.topbarClipped ||
      (checkMobileFit && check.handBottom > height)
    ) {
      throw new Error(`Layout verification failed for ${players}p ${width}x${height}: ${JSON.stringify(check)}`);
    }
    if (shotName) {
      const shot = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
      const shotPath = join(outDir, shotName);
      writeFileSync(shotPath, Buffer.from(shot.data, "base64"));
      console.log(`Saved ${shotPath}`);
    }
  }

  for (const players of [2, 3, 4, 5, 6, 7, 8]) {
    await verifyCase({
      players,
      width: 1440,
      height: 1080,
      mobile: false,
      shotName: players === 8 ? "tabletop-8p-desktop.png" : undefined,
    });
  }
  await verifyCase({ players: 5, width: 768, height: 1024, mobile: false });
  await verifyCase({ players: 8, width: 768, height: 1024, mobile: false });
  await verifyCase({ players: 8, width: 1024, height: 768, mobile: false });
  await verifyCase({
    players: 5,
    width: 390,
    height: 844,
    mobile: true,
    checkMobileFit: true,
    shotName: "tabletop-5p-mobile.png",
  });
  await verifyCase({ players: 8, width: 390, height: 844, mobile: true });
  ws.close();
  console.log("ALL 2-8 PLAYER DESKTOP, TABLET & MOBILE LAYOUT CHECKS PASSED!");
} finally {
  try { chrome.kill("SIGKILL"); } catch {}
  try { server.close(); } catch {}
  try { rmSync(udd, { recursive: true, force: true }); } catch {}
}
