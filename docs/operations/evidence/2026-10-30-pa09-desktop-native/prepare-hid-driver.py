"""Prepare a screenshot-free real-HID driver; this script performs no UI input."""
from pathlib import Path
import json

root = Path(__file__).resolve().parents[4]
source = root / 'apps/web-client/scripts/web-frame-external.mjs'
text = source.read_text()

def replace_once(old, new):
    global text
    assert text.count(old) == 1, old[:100]
    text = text.replace(old, new, 1)

replace_once('import { spawn } from "node:child_process";', 'import { spawn } from "node:child_process";\nimport assert from "node:assert/strict";\nimport { createInterface } from "node:readline";')
replace_once('from "../src/engine/perfStats.ts";', 'from ' + json.dumps(str(root / 'apps/web-client/src/engine/perfStats.ts')) + ';')
replace_once('const args = parseArgs(process.argv.slice(2));', '''function screenPoint(frame, ready) {
  const { viewport, rect } = ready;
  assert.ok([frame.x, frame.y, frame.width, frame.height, viewport.width, viewport.height,
    rect.x, rect.y, rect.width, rect.height].every(Number.isFinite));
  assert.ok(Math.abs(frame.width - viewport.width) <= 2, "AX/CSS width mismatch");
  const inset = frame.height - viewport.height;
  assert.ok(inset >= 0 && inset <= 100, "unverified title/content inset");
  assert.ok(rect.width > 0 && rect.height > 0 && rect.x >= 0 && rect.y >= 0);
  assert.ok(rect.x + rect.width <= viewport.width && rect.y + rect.height <= viewport.height);
  // AX and DOM use logical pixels. DPR is recorded, never used as an implicit scale.
  return { x: frame.x + rect.x + rect.width / 2, y: frame.y + inset + rect.y + rect.height / 2 };
}
if (process.argv.includes("--self-check")) {
  const frame = { x: 100, y: 80, width: 1280, height: 820 };
  const ready = { viewport: { width: 1280, height: 788 }, rect: { x: 40, y: 50, width: 80, height: 30 }, dpr: 2 };
  assert.deepEqual(screenPoint(frame, ready), { x: 180, y: 177 });
  assert.deepEqual(screenPoint(frame, { ...ready, dpr: 1 }), { x: 180, y: 177 });
  assert.throws(() => screenPoint({ ...frame, width: 640 }, ready), /width mismatch/);
  assert.throws(() => screenPoint(frame, { ...ready, rect: { ...ready.rect, x: 1300 } }));
  console.log("HID coordinate self-check: 4 passed; no application or input invoked");
  process.exit(0);
}
const args = parseArgs(process.argv.slice(2));''')
replace_once('let reportData = null;', '''let reportData = null;
let hidReady = null;
let hidProcess = null;
let hidCalibration = null;
const hidTool = args["hid-tool"] ? resolve(String(args["hid-tool"])) : null;
if (hidTool) assert.equal(driver, "manual", "real-HID mode requires manual page driver");''')
replace_once('  const path = decodeURIComponent(new URL(req.url ?? "/", "http://x").pathname);', '''  if (req.url === "/hid-ready" && req.method === "POST") {
    let body = "";
    req.on("data", (chunk) => { body += chunk; if (body.length > 16384) req.destroy(); });
    req.on("end", () => {
      try { hidReady = JSON.parse(body); res.writeHead(204).end(); }
      catch { res.writeHead(400).end(); }
    });
    return;
  }
  if (req.url === "/pa09-hid-bootstrap.js") {
    const target = `${battleUrl}&report=/report`;
    const boot = `(() => {
      if (!location.hash.startsWith("#/battle")) location.replace(${JSON.stringify(target)});
      const clicks = [];
      addEventListener("click", (event) => {
        const button = event.target instanceof Element ? event.target.closest("button") : null;
        if (button?.textContent?.trim() === "自动一步") clicks.push({ trusted: event.isTrusted, timestamp: event.timeStamp });
      }, true);
      const original = window.fetch.bind(window);
      window.fetch = (input, init) => {
        if (typeof input === "string" && new URL(input, location.href).pathname === "/report" && typeof init?.body === "string") {
          const payload = JSON.parse(init.body);
          payload.hid_evidence = { clicks, tauri: typeof window.__TAURI_INTERNALS__?.invoke === "function" };
          return original(input, { ...init, body: JSON.stringify(payload) });
        }
        return original(input, init);
      };
      const timer = setInterval(() => {
        const button = [...document.querySelectorAll("button")].find((item) => item.textContent?.trim() === "自动一步");
        if (!button || button.disabled) return;
        const rect = button.getBoundingClientRect();
        if (!rect.width || !rect.height || !button.contains(document.elementFromPoint(rect.x + rect.width / 2, rect.y + rect.height / 2))) return;
        clearInterval(timer);
        original("/hid-ready", { method: "POST", body: JSON.stringify({ viewport: { width: innerWidth, height: innerHeight }, dpr: devicePixelRatio, rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height } }) });
      }, 100);
    })();`;
    res.writeHead(200, { "Content-Type": "text/javascript; charset=utf-8" }).end(boot);
    return;
  }
  const path = decodeURIComponent(new URL(req.url ?? "/", "http://x").pathname);''')
old = '''      const boot = `<script>location.replace(${JSON.stringify(
        `${battleUrl}&report=http://${req.headers.host}/report`,
      )})</script>`;'''
replace_once(old, '''      const boot = hidTool ? '<script src="/pa09-hid-bootstrap.js"></script>' : `<script>location.replace(${JSON.stringify(
        `${battleUrl}&report=http://${req.headers.host}/report`,
      )})</script>`;''')
replace_once('let child = null;', '''let child = null;
process.once("exit", () => {
  if (child && app !== "safari" && app !== "firefox") { try { child.kill("SIGTERM"); } catch {} }
  if (hidProcess) { try { hidProcess.kill("SIGTERM"); } catch {} }
  server.close();
});
process.once("SIGINT", () => process.exit(130));
process.once("SIGTERM", () => process.exit(143));''')
replace_once('// ---- 等报告 ----', '''// ---- Optional real system input; no screenshots or permission-request APIs ----
if (hidTool) {
  assert.equal(app, "tauri");
  assert.ok(child?.pid && existsSync(hidTool));
  const untilReady = Date.now() + 20000;
  while (!hidReady && Date.now() < untilReady) await sleep(100);
  assert.ok(hidReady, "no real enabled-button geometry received");
  hidProcess = spawn(hidTool, [String(child.pid)], { stdio: ["pipe", "pipe", "inherit"] });
  const lines = createInterface({ input: hidProcess.stdout })[Symbol.asyncIterator]();
  const readReply = async () => {
    let timeout;
    try {
      const item = await Promise.race([lines.next(), new Promise((_, reject) => {
        timeout = setTimeout(() => reject(new Error("HID controller reply timed out")), 5000);
      })]);
      assert.ok(!item.done, "HID controller ended before reply");
      const value = JSON.parse(item.value);
      assert.ok(!value.error, value.error);
      return value;
    } finally { clearTimeout(timeout); }
  };
  const first = await readReply();
  assert.equal(first.pid, child.pid);
  const point = screenPoint(first.frame, hidReady);
  hidCalibration = { frame: first.frame, ready: hidReady, point, gap_ms: gapMs };
  writeFileSync(join(outDir, "hid-calibration.json"), JSON.stringify(hidCalibration, null, 2));
  for (let sequence = 0; sequence < steps && !reportData; sequence++) {
    hidProcess.stdin.write(JSON.stringify({ ...point, sequence }) + "\\n");
    const posted = await readReply();
    assert.equal(posted.posted, true);
    assert.equal(posted.sequence, sequence);
    await sleep(gapMs);
  }
  hidProcess.stdin.end();
}
// ---- 等报告 ----''')
replace_once('const clickSteps = reportData.interactions.filter((r) => r.type === "autoStep" && r.input_kind === "click");', '''const clickSteps = reportData.interactions.filter((r) => r.type === "autoStep" && r.input_kind === "click");
if (hidTool) {
  assert.equal(reportData.hid_evidence?.tauri, true);
  assert.equal(reportData.hid_evidence?.clicks.length, steps, "button clicks lost/duplicated");
  assert.ok(reportData.hid_evidence.clicks.every((item) => item.trusted === true), "untrusted click");
  assert.equal(clickSteps.length, steps, "trusted clicks did not all produce measured autoStep requests");
}
writeFileSync(join(outDir, "raw-report.json"), JSON.stringify(reportData, null, 2));''')
output = Path(__file__).with_name('hid-web-frame-external.mjs')
try:
    config = json.loads((root / 'apps/desktop-client/tauri.conf.json').read_text())
    capability = json.loads((root / 'apps/desktop-client/capabilities/main.json').read_text())
except (OSError, json.JSONDecodeError) as error:
    raise SystemExit(f'Cannot prepare release fixture configuration: {error}') from error
capability.pop('$schema', None)
capability['identifier'] = 'pa09-local-hid-fixture'
capability['remote'] = {'urls': ['http://localhost:5173/*']}
config['app']['windows'][0]['url'] = 'http://localhost:5173'
config['app']['security']['capabilities'] = [capability]
output.write_text(text)
Path(__file__).with_name('hid-release-config.json').write_text(json.dumps(config, ensure_ascii=False, indent=2) + '\n')
print(output)
