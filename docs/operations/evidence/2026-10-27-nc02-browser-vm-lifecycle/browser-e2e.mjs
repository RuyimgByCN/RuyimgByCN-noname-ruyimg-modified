// Isolated real Chrome -> product settings panel -> real native package server.
// Usage: node browser-e2e.mjs <ui-origin> <server-origin> <chrome-bin> <run-dir> <screenshot>
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { openSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const [ui, server, chromeBin, runDir, screenshot] = process.argv.slice(2);
assert.ok(ui && server && chromeBin && runDir && screenshot, 'missing arguments');
const profile = join(runDir, 'chrome-profile');
const chromeLog = openSync(join(runDir, 'chrome.log'), 'w');
const chrome = spawn(chromeBin, [
  '--headless=new', '--disable-gpu', '--no-sandbox',
  `--user-data-dir=${profile}`, '--remote-debugging-port=0',
  '--window-size=1280,1000', 'about:blank',
], { stdio: ['ignore', chromeLog, chromeLog] });
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const wait = async (description, fn) => {
  for (let i = 0; i < 100; i++) {
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
    try { return readFileSync(join(profile, 'DevToolsActivePort'), 'utf8').split('\n')[0]; }
    catch { return null; }
  });
  const target = await wait('Chrome page', async () => {
    try {
      const list = await fetch(`http://127.0.0.1:${port}/json/list`).then((r) => r.json());
      return list.find((item) => item.type === 'page');
    } catch { return null; }
  });
  ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
  let id = 0;
  const pending = new Map();
  const exceptions = [];
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const messageId = ++id;
    pending.set(messageId, { resolve, reject });
    ws.send(JSON.stringify({ id: messageId, method, params }));
  });
  ws.onmessage = ({ data }) => {
    const message = JSON.parse(data);
    if (message.id && pending.has(message.id)) {
      const { resolve, reject } = pending.get(message.id);
      pending.delete(message.id);
      message.error ? reject(new Error(JSON.stringify(message.error))) : resolve(message.result);
    } else if (message.method === 'Page.javascriptDialogOpening') {
      void send('Page.handleJavaScriptDialog', { accept: true });
    } else if (message.method === 'Runtime.exceptionThrown') {
      exceptions.push(message.params.exceptionDetails.text);
    } else if (message.method === 'Runtime.consoleAPICalled' && message.params.type === 'error') {
      exceptions.push(message.params.args.map((arg) => arg.value ?? arg.description).join(' '));
    }
  };
  const evaluate = async (expression) => {
    const result = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
    return result.result.value;
  };
  const body = () => evaluate('document.body.innerText');
  const action = async (label, marker) => {
    const clicked = await evaluate(`(() => {
      const fieldset = [...document.querySelectorAll('fieldset')].find(x => x.querySelector('legend')?.textContent.includes('原生扩展包'));
      const button = [...fieldset.querySelectorAll('button')].find(x => x.textContent.trim() === ${JSON.stringify(label)});
      if (!button || button.disabled) return false;
      button.click(); return true;
    })()`);
    assert.equal(clicked, true, `missing ${label} button`);
    await wait(marker, async () => (await body()).includes(marker));
    console.log(`PASS browser_${label}: ${marker}`);
  };
  const request = (path, value) => evaluate(`(async () => {
    const response = await fetch(${JSON.stringify(server)} + ${JSON.stringify(path)}, {
      method: ${JSON.stringify(value === undefined ? 'GET' : 'POST')},
      headers: {'content-type': 'application/json'},
      ${value === undefined ? '' : `body: JSON.stringify(${JSON.stringify(value)}),`}
    });
    return { status: response.status, body: await response.text() };
  })()`);
  await send('Runtime.enable');
  await send('Page.enable');
  await send('Page.navigate', { url: ui });
  await wait('product page', () => evaluate(`location.origin === ${JSON.stringify(new URL(ui).origin)} && document.readyState === 'complete'`));
  await evaluate(`localStorage.setItem('noname-next.native-server', ${JSON.stringify(server)})`);
  await wait('settings link', () => evaluate(`!!document.querySelector('a[href="/settings"]')`));
  await evaluate(`document.querySelector('a[href="/settings"]').click()`);
  await wait('installed VM fixture in settings', async () => (await body()).includes('official/vm-fixture@0.1.0'));
  const list = await request('/native/packages');
  assert.equal(list.status, 200);
  assert.deepEqual(JSON.parse(list.body), [{ namespace: 'official', package_id: 'vm-fixture', version: '0.1.0' }]);
  console.log('PASS browser_installed_vm_fixture: official/vm-fixture@0.1.0');
  const game = await request('/native/games', { packages: ['vm-fixture'], generals: ['vm_fixture'], seed: 42, seats: 4, max_steps: 2000 });
  assert.equal(game.status, 200, game.body);
  const summary = JSON.parse(game.body);
  assert.equal(summary.catalog_generals, summary.base_catalog_generals + 1);
  assert.equal(summary.finished, true);
  assert.equal(summary.stalled, false);
  console.log(`PASS server_vm_general_game: steps=${summary.steps} finished=${summary.finished}`);
  const match = await request('/native/matches', { packages: ['vm-fixture'], seed: 42, seats: 4 });
  assert.equal(match.status, 200, match.body);
  const archive = JSON.parse(match.body).archive_hex;
  const restore = () => request('/native/matches/restore', { packages: ['vm-fixture'], archive_hex: archive });
  assert.equal((await restore()).status, 200);
  console.log('PASS server_same_package_restore');
  await action('停用', 'disable 完成');
  const disabled = await restore();
  assert.equal(disabled.status, 400, disabled.body);
  console.log(`PASS disabled_package_refuses_restore: HTTP ${disabled.status}`);
  await action('激活', 'activate 完成');
  assert.equal((await restore()).status, 200);
  console.log('PASS reactivated_package_restore');
  await action('扫描安装', '安装扫描完成');
  await evaluate(`document.querySelectorAll('fieldset').forEach(el => { if (el.querySelector('legend')?.textContent.includes('原生扩展包')) el.scrollIntoView(); })`);
  const shot = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(screenshot, Buffer.from(shot.data, 'base64'));
  await action('卸载', 'uninstall 完成');
  assert.deepEqual(JSON.parse((await request('/native/packages')).body), []);
  const uninstalled = await restore();
  assert.equal(uninstalled.status, 400, uninstalled.body);
  console.log(`PASS uninstalled_package_refuses_restore: HTTP ${uninstalled.status}`);
  await action('扫描安装', '安装扫描完成');
  assert.equal((await restore()).status, 200);
  console.log('PASS reinstalled_same_package_restore');
  assert.deepEqual(exceptions, [], `browser exceptions: ${exceptions.join('; ')}`);
  console.log('PASS browser_exceptions: 0');
} finally {
  ws?.close();
  chrome.kill('SIGTERM');
}
