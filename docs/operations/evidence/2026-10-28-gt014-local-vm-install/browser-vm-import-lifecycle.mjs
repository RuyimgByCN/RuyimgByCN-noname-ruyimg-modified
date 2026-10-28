// GT-014 产品路径：headless Chrome 经扩展管理页「导入扩展包」正式入口安装含 VM 模块的
// 作者产物 → 启用 → 开局选将 → 真实触发 → 存读档 → UI 导入新版本升级 → 回退 → UI 错误分支。
// 不预置 localStorage；所有状态只经页面 UI 产生。
// Usage: node browser-vm-import-lifecycle.mjs <dist> <repo> <chrome-bin> <screenshot>
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtempSync, openSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { extname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';

const [distArg, repoArg, chromeBin, screenshot] = process.argv.slice(2);
assert.ok(distArg && repoArg && chromeBin && screenshot, 'missing arguments');
const dist = resolve(distArg);
const repo = resolve(repoArg);
const loadFixture = (dir) => {
  const root = join(repo, 'apps/web-client/scripts/fixtures', dir);
  const pack = JSON.parse(readFileSync(join(root, 'package.npack.json'), 'utf8'));
  pack.characters = JSON.parse(readFileSync(join(root, 'character.json'), 'utf8'));
  return pack;
};
const v1 = loadFixture('vm-trigger');
const v2 = loadFixture('vm-trigger-v2');
const key = `${v1.manifest.namespace}/${v1.manifest.package_id}`;
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.json': 'application/json', '.png': 'image/png', '.svg': 'image/svg+xml' };
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url ?? '/', 'http://localhost').pathname);
  let file = join(dist, path);
  try { if (!statSync(file).isFile()) throw new Error(); } catch { file = join(dist, 'index.html'); }
  res.setHeader('content-type', types[extname(file)] ?? 'application/octet-stream');
  res.setHeader('cross-origin-opener-policy', 'same-origin');
  res.setHeader('cross-origin-embedder-policy', 'require-corp');
  res.end(readFileSync(file));
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const origin = `http://127.0.0.1:${server.address().port}`;
const runDir = mkdtempSync(join(tmpdir(), 'gt014-browser-vm-'));
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
  assert.equal(await evaluate(`localStorage.getItem('noname-next.extension-store.v1')`), null, 'store must start empty');

  // ---- 扩展管理页：经正式导入入口安装 v0.1.0 并启用 ----
  const openExtensions = async () => {
    await send('Page.navigate', { url: origin + '/extensions' });
    await wait('extensions page', () => evaluate(`!!document.querySelector('textarea[aria-label="扩展包 JSON"]')`));
  };
  const importViaUi = async (json) => {
    await evaluate(`(() => {
      const box = document.querySelector('textarea[aria-label="扩展包 JSON"]');
      box.value = ${JSON.stringify(json)};
      box.dispatchEvent(new Event('input', { bubbles: true }));
      return true;
    })()`);
    await wait('import enabled', () => evaluate(`![...document.querySelectorAll('button')].find(x => x.textContent.trim() === '导入')?.disabled`));
    await evaluate(`[...document.querySelectorAll('button')].find(x => x.textContent.trim() === '导入').click()`);
  };
  const rowState = () => evaluate(`(() => {
    const row = [...document.querySelectorAll('.trow')].find(r => r.children[0]?.textContent.trim() === 'vm-fixture（导入）');
    if (!row) return null;
    const cells = [...row.children].map(c => c.textContent.trim());
    const buttons = Object.fromEntries([...row.querySelectorAll('button')].map(b => [b.textContent.trim(), !b.disabled]));
    return { key: cells[1], version: cells[2], status: cells[3], buttons };
  })()`);
  const rowClick = async (label) => assert.equal(await evaluate(`(() => {
    const row = [...document.querySelectorAll('.trow')].find(r => r.children[0]?.textContent.trim() === 'vm-fixture（导入）');
    const b = [...row.querySelectorAll('button')].find(x => x.textContent.trim() === ${JSON.stringify(label)});
    if (!b || b.disabled) return false; b.click(); return true;
  })()`), true, `${label} unavailable`);
  await openExtensions();
  assert.equal(await rowState(), null, 'no imported row before import');
  await importViaUi(JSON.stringify(v1));
  await wait('import status', () => evaluate(`document.querySelector('[role="status"]')?.innerText.includes('已导入 ${key} @ 0.1.0')`));
  let row = await rowState();
  assert.deepEqual({ key: row.key, version: row.version, status: row.status }, { key, version: '0.1.0', status: '已安装' });
  assert.equal(row.buttons['安装'], false, 'install button is for catalog packages only');
  assert.equal(row.buttons['升级'], false, 'catalog upgrade does not apply to imported packages');
  assert.equal(row.buttons['启用'], true);
  console.log(`PASS ui_import_installs_vm_package: ${key}@0.1.0 status=${row.status}`);
  await rowClick('启用');
  await wait('activated', async () => (await rowState())?.status === '已启用');
  assert.ok(await evaluate(`[...document.querySelectorAll('.lock-list li')].some(li => li.textContent.includes('${key} @ 0.1.0'))`), 'content lock lists the package');
  const stored = await evaluate(`JSON.parse(localStorage.getItem('noname-next.extension-store.v1'))`);
  assert.deepEqual(stored.contentLock, [key]);
  assert.deepEqual(JSON.parse(stored.installed[0].nativeVersions['0.1.0']), v1);
  console.log('PASS ui_activate_enters_content_lock');

  // ---- 对局：选将 → 真实触发 → 存读档重放（存档携带安装副本） ----
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
  const chosen = (await getReplies()).slice(choosePrior).find((m) => m.ok && m.data?.envelope?.view);
  assert.ok(chosen, 'missing chooseGeneral reply');
  assert.ok(chosen.data.envelope.view.seats[1].skills.some((s) => s.name === 'vm_fixture_phase_end_phase_end'));
  console.log('PASS ui_selected_vm_general: seat=1 module=vm_fixture_phase_end_phase_end');
  let reached = false;
  for (let i = 0; i < 120; i++) {
    const state = await evaluate(`({ phase: document.querySelector('.phase-badge')?.getAttribute('data-phase'), turn: document.querySelector('.turn-player')?.innerText, choice: !!document.querySelector('[aria-label="待选择"]') })`);
    if (state.turn?.includes('P1') && state.phase === 'Discard' && !state.choice) { reached = true; break; }
    const prior = await click('.toolbar button', `x.textContent.trim() === '自动一步'`);
    await wait('automatic step reply', async () => (await getReplies()).length > prior);
    await wait('automatic step idle', () => evaluate(`![...document.querySelectorAll('.toolbar button')].find(x => x.textContent.trim() === '自动一步')?.disabled`));
  }
  assert.ok(reached, 'did not reach P1 discard without pending choice');
  const before = (await getReplies()).findLast((m) => m.ok && m.data?.view)?.data;
  assert.equal(before.view.current_player, 1);
  assert.equal(before.view.phase, 'Discard');
  const beforeHash = before.hash_hex;
  const beforeHand = before.view.seats[1].hand_count;
  const savePrior = await click('.toolbar button', `x.textContent.trim() === '存档'`);
  await wait('save reply', async () => (await getReplies()).length > savePrior);
  await wait('save UI status', () => evaluate(`document.querySelector('.toolbar .status')?.innerText.includes('已存档')`));
  const save = await evaluate(`JSON.parse(localStorage.getItem('noname-next.native-saves.v1'))[0]`);
  assert.deepEqual(save.contentLock, [key]);
  assert.deepEqual(JSON.parse(save.packsJson), [v1]);
  console.log(`PASS ui_save_contains_installed_copy: hand=${beforeHand} hash=${beforeHash}`);
  const endPrior = await click('.toolbar button', `x.textContent.trim() === '结束阶段'`);
  await wait('end phase reply', async () => (await getReplies()).length > endPrior);
  await wait('end phase UI', () => evaluate(`document.querySelector('.phase-badge')?.getAttribute('data-phase') === 'End'`));
  const after = (await getReplies()).slice(endPrior).find((m) => m.ok && m.data?.view)?.data;
  assert.equal(after.view.seats[1].hand_count, beforeHand + 1);
  assert.ok(after.events.some((event) => event.PublicEvent?.code === 2));
  console.log(`PASS ui_vm_end_effect: hand=${beforeHand}->${beforeHand + 1} hash=${after.hash_hex}`);
  const shot = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(screenshot, Buffer.from(shot.data, 'base64'));
  const loadPrior = await click('.toolbar button', `x.textContent.trim() === '读档'`);
  await wait('load reply', async () => (await getReplies()).length > loadPrior);
  await wait('restored discard UI', () => evaluate(`document.querySelector('.phase-badge')?.getAttribute('data-phase') === 'Discard'`));
  const restored = (await getReplies()).slice(loadPrior).find((m) => m.ok && m.data?.view)?.data;
  assert.equal(restored?.hash_hex, beforeHash);
  const replayPrior = await click('.toolbar button', `x.textContent.trim() === '结束阶段'`);
  await wait('replay reply', async () => (await getReplies()).length > replayPrior);
  const replay = (await getReplies()).slice(replayPrior).find((m) => m.ok && m.data?.view)?.data;
  assert.equal(replay?.hash_hex, after.hash_hex);
  console.log(`PASS ui_restore_replay_same_hash: ${replay.hash_hex}`);

  // ---- 扩展管理页：UI 导入 v0.1.1 升级（保留旧副本）→ 回退 → 错误分支不改写 ----
  await openExtensions();
  await importViaUi(JSON.stringify(v2));
  await wait('upgrade status', () => evaluate(`document.querySelector('[role="status"]')?.innerText.includes('已导入 ${key} @ 0.1.1')`));
  row = await rowState();
  assert.deepEqual({ version: row.version, status: row.status, rollback: row.buttons['回退'] }, { version: '0.1.1', status: '已启用', rollback: true });
  const upgraded = await evaluate(`JSON.parse(localStorage.getItem('noname-next.extension-store.v1')).installed[0]`);
  assert.deepEqual(upgraded.history, ['0.1.0', '0.1.1']);
  assert.deepEqual(JSON.parse(upgraded.nativeVersions['0.1.0']), v1);
  assert.deepEqual(JSON.parse(upgraded.nativeVersions['0.1.1']), v2);
  assert.ok(await evaluate(`[...document.querySelectorAll('.lock-list li')].some(li => li.textContent.includes('${key} @ 0.1.1'))`));
  console.log('PASS ui_import_upgrade_keeps_history_and_old_copy');
  await rowClick('回退');
  await wait('rolled back', async () => (await rowState())?.version === '0.1.0');
  assert.ok(await evaluate(`[...document.querySelectorAll('.lock-list li')].some(li => li.textContent.includes('${key} @ 0.1.0'))`));
  console.log('PASS ui_rollback_to_previous_version');
  const storeBefore = await evaluate(`localStorage.getItem('noname-next.extension-store.v1')`);
  await importViaUi('{not json');
  await wait('invalid json alert', () => evaluate(`document.querySelector('[role="alert"]')?.innerText.includes('合法 JSON')`));
  const corrupt = structuredClone(v1);
  corrupt.modules[0].entry_pc = corrupt.modules[0].code.length;
  await importViaUi(JSON.stringify(corrupt));
  await wait('corrupt module alert', () => evaluate(`document.querySelector('[role="alert"]')?.innerText.includes('入口或预算字段无效')`));
  const conflict = structuredClone(v1);
  conflict.modules[0].caps_mask += 1;
  await importViaUi(JSON.stringify(conflict));
  await wait('conflict alert', () => evaluate(`document.querySelector('[role="alert"]')?.innerText.includes('同版本包内容冲突')`));
  assert.equal(await evaluate(`localStorage.getItem('noname-next.extension-store.v1')`), storeBefore, 'rejected imports leave the store untouched');
  row = await rowState();
  assert.deepEqual({ version: row.version, status: row.status }, { version: '0.1.0', status: '已启用' });
  console.log('PASS ui_rejected_imports_leave_store_untouched');
  await rowClick('停用');
  await wait('disabled', async () => (await rowState())?.status === '已停用');
  await rowClick('卸载');
  await wait('uninstalled', async () => (await rowState()) === null);
  assert.deepEqual((await evaluate(`JSON.parse(localStorage.getItem('noname-next.extension-store.v1'))`)).installed, []);
  console.log('PASS ui_disable_and_uninstall');
  assert.deepEqual(exceptions, []);
  console.log('PASS browser_exceptions: 0');
  console.log(`run_dir=${runDir} origin=${origin}`);
} finally {
  ws?.close();
  chrome.kill('SIGTERM');
  await new Promise((resolve) => server.close(resolve));
}
