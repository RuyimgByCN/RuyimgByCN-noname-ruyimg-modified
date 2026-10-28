// Real product UI and native Worker, with a VM-only fixture preloaded as an installed copy.
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
  await wait('general candidates', () => evaluate(`!!document.querySelector('.general-select button')`));
  const getReplies = () => evaluate('window.__vmReplies');
  const initial = (await getReplies()).findLast((m) => m.ok && m.data?.view)?.data;
  assert.ok(initial?.hash_hex, 'missing Worker start envelope');
  console.log(`PASS vm_only_product_start: key=${key} hash=${initial.hash_hex}`);
  const click = async (label) => {
    const prior = (await getReplies()).length;
    const ok = await evaluate(`(() => { const b = [...document.querySelectorAll('.toolbar button')].find(x => x.textContent.trim() === ${JSON.stringify(label)}); if (!b || b.disabled) return false; b.click(); return true; })()`);
    assert.equal(ok, true, `button unavailable: ${label}`);
    return prior;
  };
  const save = async (previousId = '') => {
    const prior = await click('存档');
    await wait('save reply', async () => (await getReplies()).length > prior);
    return wait('new saved slot', () => evaluate(`(() => { const s = JSON.parse(localStorage.getItem('noname-next.native-saves.v1') || '[]')[0]; return s && s.id !== ${JSON.stringify(previousId)} ? s : null; })()`));
  };
  const first = await save();
  assert.deepEqual(first.contentLock, [key]);
  const firstPack = JSON.parse(first.packsJson)[0];
  assert.equal(firstPack.modules[0].name, 'vm_fixture_phase_end_phase_end');
  assert.ok(!firstPack.characters && !firstPack.cards && !firstPack.pile);
  console.log('PASS vm_only_saved_copy');
  const stepPrior = await click('自动一步');
  await wait('automatic step reply', async () => (await getReplies()).length > stepPrior);
  await wait('step idle', () => evaluate(`![...document.querySelectorAll('.toolbar button')].find(x => x.textContent.trim() === '自动一步')?.disabled`));
  const step = (await getReplies()).slice(stepPrior).find((m) => m.ok && m.data?.view);
  assert.ok(step?.data?.hash_hex && step.data.hash_hex !== initial.hash_hex);
  const restorePrior = await click('读档');
  const restored = await wait('restored envelope', async () => (await getReplies()).slice(restorePrior).find((m) => m.ok && m.data?.view));
  assert.equal(restored.data.hash_hex, initial.hash_hex);
  await wait('restore idle', () => evaluate(`![...document.querySelectorAll('.toolbar button')].find(x => x.textContent.trim() === '存档')?.disabled`));
  console.log(`PASS vm_only_product_restore: hash=${restored.data.hash_hex}`);
  const afterRestore = await save(first.id);
  await evaluate(`(() => { const saves = JSON.parse(localStorage.getItem('noname-next.native-saves.v1')); const packs = JSON.parse(saves[0].packsJson); packs[0].modules[0].local_count++; saves[0].packsJson = JSON.stringify(packs); localStorage.setItem('noname-next.native-saves.v1', JSON.stringify(saves)); })()`);
  const rejectPrior = await click('读档');
  const rejected = await wait('bad module rejection', async () => (await getReplies()).slice(rejectPrior).find((m) => m.ok === false));
  assert.match(rejected.error, /module .*logic_hash mismatch/);
  await wait('rejection visible', () => evaluate(`document.querySelector('[role="alert"]')?.innerText.includes('logic_hash mismatch')`));
  const afterReject = await save(afterRestore.id);
  assert.equal(afterReject.hex, afterRestore.hex);
  console.log('PASS tampered_restore_rejected_without_mutation');
  const shot = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(screenshot, Buffer.from(shot.data, 'base64'));
  assert.deepEqual(exceptions, []);
  console.log('PASS browser_exceptions: 0');
  console.log(`run_dir=${runDir} origin=${origin}`);
} finally {
  ws?.close();
  chrome.kill('SIGTERM');
  await new Promise((resolve) => server.close(resolve));
}
