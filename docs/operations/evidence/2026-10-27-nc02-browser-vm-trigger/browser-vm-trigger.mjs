// Real product UI and native Worker, with a fixture preloaded as an installed copy.
// Usage: node browser-vm-trigger.mjs <dist> <repo> <chrome-bin> <screenshot>
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtempSync, openSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { extname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';

const [distArg, repoArg, chromeBin, screenshot] = process.argv.slice(2);
assert.ok(distArg && repoArg && chromeBin && screenshot, 'missing arguments');
const dist = resolve(distArg);
const fixture = join(resolve(repoArg), 'apps/web-client/scripts/fixtures/vm-trigger');
const pack = JSON.parse(readFileSync(join(fixture, 'package.npack.json'), 'utf8'));
pack.characters = JSON.parse(readFileSync(join(fixture, 'character.json'), 'utf8'));
const key = `${pack.manifest.namespace}/${pack.manifest.package_id}`;
const fingerprint = pack.manifest.logic_hash.map((n) => n.toString(16).padStart(2, '0')).join('');
const record = {
  key, dir: 'vm-fixture', packageId: pack.manifest.package_id, namespace: pack.manifest.namespace,
  version: pack.manifest.version, history: [pack.manifest.version], lifecycle: 'active',
  installedAt: '2026-01-01T00:00:00.000Z', updatedAt: '2026-01-01T00:00:00.000Z',
  logicFingerprint: fingerprint, nativeVersions: { [pack.manifest.version]: JSON.stringify(pack) },
};
const mime = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.json': 'application/json' };
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url ?? '/', 'http://localhost').pathname);
  const candidate = resolve(dist, '.' + path);
  if (!candidate.startsWith(dist + '/') && candidate !== dist) return res.writeHead(403).end();
  let file = candidate;
  try { if (!statSync(file).isFile()) file = join(dist, 'index.html'); }
  catch { file = join(dist, 'index.html'); }
  res.writeHead(200, { 'Content-Type': mime[extname(file)] ?? 'application/octet-stream' }).end(readFileSync(file));
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const origin = `http://127.0.0.1:${server.address().port}`;
const runDir = mkdtempSync(join(tmpdir(), 'nc02-browser-vm-'));
const chromeLog = openSync(join(runDir, 'chrome.log'), 'w');
const chrome = spawn(chromeBin, [
  '--headless=new', '--disable-gpu', '--no-sandbox', `--user-data-dir=${runDir}/profile`,
  '--remote-debugging-port=0', '--window-size=1280,1000', 'about:blank',
], { stdio: ['ignore', chromeLog, chromeLog] });
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const wait = async (description, fn) => {
  for (let i = 0; i < 200; i++) {
    const result = await fn();
    if (result) return result;
    if (chrome.exitCode !== null) throw new Error(`Chrome exited ${chrome.exitCode}`);
    await pause(100);
  }
  throw new Error(`timed out: ${description}`);
};
let ws;
try {
  const port = await wait('Chrome debug port', () => {
    try { return readFileSync(join(runDir, 'profile/DevToolsActivePort'), 'utf8').split('\n')[0]; }
    catch { return null; }
  });
  const target = await wait('Chrome page', async () => {
    try { return (await fetch(`http://127.0.0.1:${port}/json/list`).then((r) => r.json())).find((x) => x.type === 'page'); }
    catch { return null; }
  });
  ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
  let id = 0;
  const pending = new Map();
  const exceptions = [];
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const n = ++id;
    pending.set(n, { resolve, reject });
    ws.send(JSON.stringify({ id: n, method, params }));
  });
  ws.onmessage = ({ data }) => {
    const msg = JSON.parse(data);
    if (msg.id && pending.has(msg.id)) {
      const { resolve, reject } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
    } else if (msg.method === 'Runtime.exceptionThrown') exceptions.push(msg.params.exceptionDetails.text);
    else if (msg.method === 'Runtime.consoleAPICalled' && msg.params.type === 'error')
      exceptions.push(msg.params.args.map((x) => x.value ?? x.description).join(' '));
  };
  const evaluate = async (expression) => {
    const result = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
  };
  await send('Runtime.enable');
  await send('Page.enable');
  await send('Page.addScriptToEvaluateOnNewDocument', { source: `(() => {
    const Original = window.Worker;
    window.__vmReplies = [];
    window.Worker = class extends Original {
      constructor(...args) {
        super(...args);
        this.addEventListener('message', (event) => window.__vmReplies.push(event.data));
      }
    };
  })()` });
  await send('Page.navigate', { url: origin + '/' });
  await wait('initial page', () => evaluate(`location.origin === ${JSON.stringify(origin)} && document.readyState === 'complete'`));
  const snap = { installed: [record], contentLock: [key], matchFrozen: false };
  await evaluate(`localStorage.setItem('noname-next.extension-store.v1', ${JSON.stringify(JSON.stringify(snap))})`);
  await send('Page.navigate', { url: origin + '/battle?players=4&seed=42&skipAnims=1' });
  await wait('fixture candidate', () => evaluate(`!![...document.querySelectorAll('.general-select button')].find(x => x.getAttribute('aria-label')?.includes('选择 vm_fixture（'))`));
  console.log('PASS formal_content_lock_loaded: ' + key);
  const getReplies = () => evaluate('window.__vmReplies');
  const click = async (selector, label) => {
    const prior = (await getReplies()).length;
    const ok = await evaluate(`(() => { const b = [...document.querySelectorAll(${JSON.stringify(selector)})].find(x => ${label}); if (!b || b.disabled) return false; b.click(); return true; })()`);
    assert.equal(ok, true, 'button unavailable');
    return prior;
  };
  const choosePrior = await click('.general-select button', `x.getAttribute('aria-label')?.includes('选择 vm_fixture（')`);
  await wait('selection accepted', () => evaluate(`document.querySelector('.general-select .success')?.innerText.includes('vm_fixture')`));
  const selected = await getReplies();
  const chosen = selected.slice(choosePrior).find((m) => m.ok && m.data?.envelope?.view);
  assert.ok(chosen, 'missing chooseGeneral reply');
  assert.ok(chosen.data.envelope.view.seats[1].skills.some((s) => s.name === 'vm_fixture_phase_end_phase_end'));
  console.log('PASS ui_selected_vm_general: seat=1 module=vm_fixture_phase_end_phase_end');
  let reached = false;
  for (let i = 0; i < 120; i++) {
    const state = await evaluate(`({ phase: document.querySelector('.phase-badge')?.getAttribute('data-phase'), turn: document.querySelector('.turn-player')?.innerText, choice: !!document.querySelector('[aria-label="待选择"]'), hand: document.querySelector('.seat.self .meta')?.innerText, status: document.querySelector('.toolbar .status')?.innerText })`);
    console.log(`STEP ${i} ${JSON.stringify(state)}`);
    if (state.turn?.includes('P1') && state.phase === 'Discard' && !state.choice) { reached = true; break; }
    const prior = await click('.toolbar button', `x.textContent.trim() === '自动一步'`);
    await wait('automatic step reply', async () => (await getReplies()).length > prior);
    await wait('automatic step idle', () => evaluate(`![...document.querySelectorAll('.toolbar button')].find(x => x.textContent.trim() === '自动一步')?.disabled`));
  }
  assert.ok(reached, 'did not reach P1 discard without pending choice');
  const beforeReplies = await getReplies();
  const before = beforeReplies.findLast((m) => m.ok && m.data?.view)?.data;
  assert.equal(before.view.current_player, 1);
  assert.equal(before.view.phase, 'Discard');
  const beforeHash = before.hash_hex;
  const beforeHand = before.view.seats[1].hand_count;
  const beforeVisible = await evaluate(`document.querySelector('.seat.self .meta')?.innerText`);
  assert.match(beforeVisible, new RegExp('手牌\\s*' + beforeHand));
  const savePrior = await click('.toolbar button', `x.textContent.trim() === '存档'`);
  await wait('save reply', async () => (await getReplies()).length > savePrior);
  await wait('save UI status', () => evaluate(`document.querySelector('.toolbar .status')?.innerText.includes('已存档')`));
  const save = await evaluate(`JSON.parse(localStorage.getItem('noname-next.native-saves.v1'))[0]`);
  assert.deepEqual(save.contentLock, [key]);
  assert.equal(JSON.parse(save.packsJson)[0].modules[0].name, 'vm_fixture_phase_end_phase_end');
  console.log(`PASS ui_save_contains_vm_copy: phase=Discard hand=${beforeHand} hash=${beforeHash}`);
  const endPrior = await click('.toolbar button', `x.textContent.trim() === '结束阶段'`);
  await wait('end phase reply', async () => (await getReplies()).length > endPrior);
  await wait('end phase UI', () => evaluate(`document.querySelector('.phase-badge')?.getAttribute('data-phase') === 'End'`));
  const after = (await getReplies()).slice(endPrior).find((m) => m.ok && m.data?.view)?.data;
  assert.ok(after, 'missing end phase reply');
  assert.equal(after.view.current_player, 1);
  assert.equal(after.view.phase, 'End');
  assert.equal(after.view.seats[1].hand_count, beforeHand + 1);
  console.log('END_EVENTS ' + JSON.stringify(after.events));
  assert.ok(after.events.some((event) => event.PublicEvent?.code === 2));
  assert.match(await evaluate(`document.querySelector('.seat.self .meta')?.innerText`), new RegExp('手牌\\s*' + (beforeHand + 1)));
  console.log(`PASS ui_vm_end_effect: phase=Discard->End hand=${beforeHand}->${beforeHand + 1} hash=${after.hash_hex}`);
  const shot = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(screenshot, Buffer.from(shot.data, 'base64'));
  const loadPrior = await click('.toolbar button', `x.textContent.trim() === '读档'`);
  await wait('load reply', async () => (await getReplies()).length > loadPrior);
  await wait('restored discard UI', () => evaluate(`document.querySelector('.phase-badge')?.getAttribute('data-phase') === 'Discard'`));
  const restored = (await getReplies()).slice(loadPrior).find((m) => m.ok && m.data?.view)?.data;
  assert.equal(restored?.hash_hex, beforeHash);
  const replayPrior = await click('.toolbar button', `x.textContent.trim() === '结束阶段'`);
  await wait('replay reply', async () => (await getReplies()).length > replayPrior);
  await wait('replayed end UI', () => evaluate(`document.querySelector('.phase-badge')?.getAttribute('data-phase') === 'End'`));
  const replay = (await getReplies()).slice(replayPrior).find((m) => m.ok && m.data?.view)?.data;
  assert.equal(replay?.hash_hex, after.hash_hex);
  assert.equal(replay.view.seats[1].hand_count, beforeHand + 1);
  console.log(`PASS ui_restore_replay_same_hash: ${replay.hash_hex}`);
  // Control: same package and seed, but leave P1's default character without a VM skill.
  await send('Page.navigate', { url: origin + '/battle?players=4&seed=42&skipAnims=1' });
  await wait('control candidate', () => evaluate(`!![...document.querySelectorAll('.general-select button')].find(x => x.getAttribute('aria-label')?.includes('选择 vm_fixture（'))`));
  let controlReached = false;
  for (let i = 0; i < 120; i++) {
    const state = await evaluate(`({ phase: document.querySelector('.phase-badge')?.getAttribute('data-phase'), turn: document.querySelector('.turn-player')?.innerText, choice: !!document.querySelector('[aria-label="待选择"]') })`);
    if (state.turn?.includes('P1') && state.phase === 'Discard' && !state.choice) { controlReached = true; break; }
    const prior = await click('.toolbar button', `x.textContent.trim() === '自动一步'`);
    await wait('control step reply', async () => (await getReplies()).length > prior);
    await wait('control step idle', () => evaluate(`![...document.querySelectorAll('.toolbar button')].find(x => x.textContent.trim() === '自动一步')?.disabled`));
  }
  assert.ok(controlReached, 'control did not reach P1 discard');
  const controlBefore = (await getReplies()).findLast((m) => m.ok && m.data?.view)?.data;
  assert.equal(controlBefore.view.current_player, 1);
  assert.equal(controlBefore.view.phase, 'Discard');
  assert.equal(controlBefore.view.seats[1].skills.length, 0);
  const controlPrior = await click('.toolbar button', `x.textContent.trim() === '结束阶段'`);
  await wait('control end reply', async () => (await getReplies()).length > controlPrior);
  await wait('control end UI', () => evaluate(`document.querySelector('.phase-badge')?.getAttribute('data-phase') === 'End'`));
  const controlAfter = (await getReplies()).slice(controlPrior).find((m) => m.ok && m.data?.view)?.data;
  assert.equal(controlAfter.view.seats[1].hand_count, controlBefore.view.seats[1].hand_count);
  assert.ok(!controlAfter.events.some((event) => event.PublicEvent?.code === 2));
  console.log(`PASS control_without_vm_skill: phase=Discard->End hand=${controlBefore.view.seats[1].hand_count}->${controlAfter.view.seats[1].hand_count}`);
  assert.deepEqual(exceptions, []);
  console.log('PASS browser_exceptions: 0');
  console.log(`run_dir=${runDir} origin=${origin}`);
} finally {
  ws?.close();
  chrome.kill('SIGTERM');
  await new Promise((resolve) => server.close(resolve));
}
