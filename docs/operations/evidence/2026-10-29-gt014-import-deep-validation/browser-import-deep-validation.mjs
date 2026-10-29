// GT-014 导入期深层校验（裁决第 12 项）产品路径：headless Chrome 经扩展页「导入」
// 按钮走真实 Worker `validatePackage` 深层校验——
//   1) 破坏指纹的两类变异（local_count+1、篡改触发事件名）在**导入期**被拒，
//      页面显示 Rust 原始错因，localStorage 逐字不变；
//   2) 合法包导入成功：列表出现「未签名」标记（本地验签暂缓），按钮不滞留「校验中…」。
// 复用 2026-10-28-gt014-local-vm-install 驱动的 CDP 骨架。不预置 localStorage。
// Usage: node browser-import-deep-validation.mjs <dist> <repo> <chrome-bin> <screenshot>
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
const runDir = mkdtempSync(join(tmpdir(), 'gt014-deep-'));
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
  await send('Page.navigate', { url: origin + '/extensions' });
  await wait('extensions page', () => evaluate(`location.origin === ${JSON.stringify(origin)} && !!document.querySelector('textarea[aria-label="扩展包 JSON"]')`));
  assert.equal(await evaluate(`localStorage.getItem('noname-next.extension-store.v1')`), null, 'store must start empty');
  const storeRaw = () => evaluate(`localStorage.getItem('noname-next.extension-store.v1')`);

  const importViaUi = async (json) => {
    await evaluate(`(() => {
      const box = document.querySelector('textarea[aria-label="扩展包 JSON"]');
      box.value = ${JSON.stringify(json)};
      box.dispatchEvent(new Event('input', { bubbles: true }));
      return true;
    })()`);
    await wait('import enabled', () => evaluate(`![...document.querySelectorAll('button')].find(x => x.textContent.trim() === '导入' || x.textContent.trim() === '校验中…')?.disabled`));
    await evaluate(`[...document.querySelectorAll('button')].find(x => x.textContent.trim() === '导入').click()`);
  };
  const rowState = () => evaluate(`(() => {
    const prefix = ${JSON.stringify(`${v1.manifest.package_id}（导入）`)};
    const row = [...document.querySelectorAll('.trow')].find(r => (r.children[0]?.textContent.trim() ?? '').startsWith(prefix));
    if (!row) return null;
    const cells = [...row.children].map(c => c.textContent.trim());
    const buttons = Object.fromEntries([...row.querySelectorAll('button')].map(b => [b.textContent.trim(), !b.disabled]));
    return { title: cells[0], version: cells[2], status: cells[3], buttons };
  })()`);

  // ---- ① 指纹变异：导入期被拒，页面显示 Rust 错因，存储逐字不变 ----
  for (const [name, mutate, pattern] of [
    ['corrupt_module_local_count', (p) => { p.modules[0].local_count += 1; }, /logic_hash mismatch/],
    ['tampered_trigger_event', (p) => { p.trigger_index[0].event = 'phase_draw'; }, /logic_hash mismatch/],
  ]) {
    const bad = structuredClone(v1);
    mutate(bad);
    await importViaUi(JSON.stringify(bad));
    const errorText = await wait(`${name} error shown`, () => evaluate(`document.querySelector('[role="alert"]')?.innerText ?? ''`));
    assert.ok(pattern.test(errorText), `${name}: expected ${pattern} in alert, got: ${errorText}`);
    assert.equal(await storeRaw(), null, `${name}: rejected import must not write the store`);
    console.log(`PASS ui_import_rejects_${name}: ${errorText}`);
  }

  // ---- ② 合法导入成功：未签名标记 + 通知文案；按钮不滞留「校验中…」 ----
  await importViaUi(JSON.stringify(v1));
  await wait('import notice', () => evaluate(`document.querySelector('[role="status"]')?.innerText.includes('已导入 ${key} @ 0.1.0') && document.querySelector('[role="status"]')?.innerText.includes('未签名')`));
  const row = await wait('imported row visible', () => rowState());
  assert.ok(row.title.includes('未签名'), `row title must carry the unsigned badge: ${row.title}`);
  assert.deepEqual({ version: row.version, status: row.status }, { version: '0.1.0', status: '已安装' });
  const buttonLabel = await evaluate(`[...document.querySelectorAll('button')].find(x => x.textContent.trim() === '导入' || x.textContent.trim() === '校验中…')?.textContent.trim()`);
  assert.equal(buttonLabel, '导入', 'import button must not be stuck on 校验中…');
  console.log(`PASS ui_import_valid_marks_unsigned: title=${row.title} status=${row.status}`);

  await send('Page.captureScreenshot', { format: 'png' }).then((r) => writeFileSync(screenshot, Buffer.from(r.data, 'base64')));
  assert.deepEqual(exceptions, []);
  console.log('PASS browser_exceptions: 0');
  console.log(`run_dir=${runDir} origin=${origin}`);
} finally {
  ws?.close();
  chrome.kill('SIGTERM');
  await new Promise((resolve) => server.close(resolve));
}
