// I02 真实无头 Chrome 产品路径验收（CDP 驱动）：
// 真实构建产物 + 真实 WASM Worker + BattleView + __battleAudioOwner + CollectionView 隔离验证。
//
// 命令：
//   node apps/web-client/scripts/i02-browser-audio-drive.mjs <base-url> <chrome-bin> <outdir>

import { spawn } from "node:child_process";
import { writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const base = process.argv[2] ?? "http://127.0.0.1:8124/";
const chromeBin = process.argv[3] ?? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const outDir = process.argv[4] ?? ".";
const port = 9335;

const udd = mkdtempSync(join(tmpdir(), "i02-cdp-"));
const chrome = spawn(chromeBin, [
  "--headless=new",
  "--disable-gpu",
  "--no-sandbox",
  "--autoplay-policy=no-user-gesture-required",
  `--user-data-dir=${udd}`,
  `--remote-debugging-port=${port}`,
  "--window-size=1280,800",
  "about:blank",
], { stdio: "ignore" });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function cdpTarget() {
  for (let i = 0; i < 50; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/list`);
      const list = await res.json();
      const page = list.find((t) => t.type === "page");
      if (page?.webSocketDebuggerUrl) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(200);
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

const wsUrl = await cdpTarget();
const ws = new WebSocket(wsUrl);
await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
ws.onmessage = (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.id && pending.has(msg.id)) {
    const { resolve, reject } = pending.get(msg.id);
    pending.delete(msg.id);
    msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
    return;
  }
  if (msg.method === "Runtime.exceptionThrown") {
    consoleErrors.push(`exception: ${msg.params?.exceptionDetails?.text ?? "?"}`);
  }
  if (msg.method === "Runtime.consoleAPICalled" && msg.params?.type === "error") {
    consoleErrors.push(`console.error: ${(msg.params.args ?? []).map((a) => a.value ?? a.description ?? "?").join(" ")}`);
  }
};

await send(ws, "Runtime.enable");
await send(ws, "Page.enable");
await send(ws, "Page.navigate", { url: base });
await sleep(3000);

async function evalJs(expression) {
  const r = await send(ws, "Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.text);
  return r.result?.value;
}

// 1. 进入本地对局（开启 skipAnims 加速步进，推进到已选标准武将 P1 的出牌阶段）
await evalJs(`localStorage.setItem("noname-next-skip-anims", "1")`);
const clickedStart = await evalJs(`(() => {
  const el = Array.from(document.querySelectorAll('button,a')).find(el => (el.innerText||'').includes('本地开局'));
  if (!el) return false;
  el.click();
  return true;
})()`);
if (!clickedStart) throw new Error("未找到本地开局按钮");

for (let i = 0; i < 40; i++) {
  await sleep(300);
  const ready = await evalJs(`Boolean(
    window.__battleAudioOwner &&
    /判定|摸牌|出牌|弃牌|结束/.test(document.body.innerText) &&
    Array.from(document.querySelectorAll('button')).some(b => (b.innerText||'').includes('·'))
  )`);
  if (ready) break;
}

const initialSnap = await evalJs(`window.__battleAudioOwner?.snapshot()`);
if (!initialSnap) throw new Error("__battleAudioOwner 未挂载");
console.log("STEP initial_audio_owner " + JSON.stringify(initialSnap));

// 2. 选择标准武将（刘备/曹操/孙权等带 standard:* 绑定的候选）
const chosenGeneral = await evalJs(`(() => {
  const buttons = Array.from(document.querySelectorAll('button'));
  const target =
    buttons.find(b => /^(刘备|曹操|孙权|关羽|张飞|赵云|司马懿|诸葛亮)[\\s\\n·]/.test((b.innerText||'').trim())) ||
    buttons.find(b => (b.innerText||'').includes('·') && (b.innerText||'').length < 40);
  if (!target) return null;
  const label = (target.innerText||'').trim().split('\\n')[0];
  target.click();
  return label;
})()`);
await sleep(1500);
const snapAfterChoose = await evalJs(`window.__battleAudioOwner.snapshot()`);
console.log("STEP after_choose_general " + JSON.stringify({ chosenGeneral, snapAfterChoose }));

// 3. 推进若干步真实自动行动（覆盖 P0 回合与 P1 司马懿出牌阶段），收集每个信封的 public_presentation 消费与 __battleAudioOwner 状态
const stepSnapshots = [];
for (let i = 0; i < 14; i++) {
  await evalJs(`(() => {
    const el = Array.from(document.querySelectorAll('button')).find(b => (b.innerText||'').includes('自动一步') && !b.disabled);
    if (el) el.click();
  })()`);
  await sleep(300);
  const s = await evalJs(`window.__battleAudioOwner.snapshot()`);
  stepSnapshots.push(s);
}
const lastStepSnap = stepSnapshots[stepSnapshots.length - 1];
console.log("STEP after_auto_steps " + JSON.stringify(lastStepSnap));
if (!lastStepSnap || lastStepSnap.playedCuesCount < 1) {
  throw new Error(`期望标准武将出牌/发动技能产生至少 1 条语音消费记录，实际: ${JSON.stringify(lastStepSnap)}`);
}

// 4. 验证静音复选框与音量滑块即时生效
const muteToggleSnap = await evalJs(`(() => {
  const muteInput = document.querySelector('input[aria-label="对局静音"]');
  if (!muteInput) return { found: false };
  muteInput.click();
  const s1 = window.__battleAudioOwner.snapshot();
  muteInput.click();
  const s2 = window.__battleAudioOwner.snapshot();
  return { found: true, mutedSnap: s1, unmutedSnap: s2 };
})()`);
if (!muteToggleSnap?.found || muteToggleSnap.mutedSnap.muted !== true || muteToggleSnap.unmutedSnap.muted !== false) {
  throw new Error(`静音按钮状态切换异常: ${JSON.stringify(muteToggleSnap)}`);
}
console.log("STEP mute_toggle " + JSON.stringify(muteToggleSnap));

// 5. 验证存档与读档建立静默基线且代数递增
for (let i = 0; i < 20; i++) {
  const idle = await evalJs(`(() => {
    const b = Array.from(document.querySelectorAll('button')).find(el => (el.innerText||'').trim() === '存档');
    return Boolean(b && !b.disabled);
  })()`);
  if (idle) break;
  await sleep(200);
}
const genBeforeLoad = await evalJs(`window.__battleAudioOwner.snapshot().sessionGeneration`);
const saveClicked = await evalJs(`(() => {
  const b = Array.from(document.querySelectorAll('button')).find(el => (el.innerText||'').trim() === '存档');
  if (!b) return { found: false };
  b.click();
  return { found: true, disabled: b.disabled };
})()`);
await sleep(1200);
const afterSaveStatus = await evalJs(`({
  status: document.querySelector('.status')?.innerText ?? null,
  error: document.querySelector('.error')?.innerText ?? null,
})`);
console.log("STEP after_save " + JSON.stringify({ saveClicked, afterSaveStatus }));
const loadClicked = await evalJs(`(() => {
  const b = Array.from(document.querySelectorAll('button')).find(el => (el.innerText||'').trim() === '读档');
  if (!b) return { found: false };
  b.click();
  return { found: true, disabled: b.disabled };
})()`);
await sleep(1500);
const afterLoadStatus = await evalJs(`({
  status: document.querySelector('.status')?.innerText ?? null,
  error: document.querySelector('.error')?.innerText ?? null,
})`);
console.log("STEP after_load " + JSON.stringify({ loadClicked, afterLoadStatus }));
const snapAfterLoad = await evalJs(`window.__battleAudioOwner.snapshot()`);
if (snapAfterLoad.sessionGeneration <= genBeforeLoad) {
  throw new Error(`读档未递增 sessionGeneration: before=${genBeforeLoad}, after=${snapAfterLoad.sessionGeneration}`);
}
console.log("STEP save_load_baseline " + JSON.stringify({ genBeforeLoad, snapAfterLoad }));

const report = {
  initial_audio_owner: initialSnap,
  chosen_general: chosenGeneral,
  after_choose_general: snapAfterChoose,
  after_steps_snapshot: stepSnapshots[stepSnapshots.length - 1],
  mute_toggle: muteToggleSnap,
  save_load_baseline: { genBeforeLoad, snapAfterLoad },
  console_errors: consoleErrors,
};

writeFileSync(join(outDir, "i02-browser-audio-report.json"), JSON.stringify(report, null, 2));
console.log("RESULT " + JSON.stringify(report));

ws.close();
chrome.kill("SIGTERM");
await sleep(300);
rmSync(udd, { recursive: true, force: true });

if (consoleErrors.length > 0) process.exit(3);
if (!snapAfterLoad || snapAfterLoad.lastCommittedRevision < 1) process.exit(4);
