// Real browser, built web client and native WASM Worker; observe replies without modifying them.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtempSync, readFileSync, rmSync, statSync } from 'node:fs';
import { extname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';

const [distArg, chromeBin] = process.argv.slice(2);
assert.ok(distArg && chromeBin, 'missing dist or browser');
const dist = resolve(distArg);
const mime = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.json': 'application/json' };
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url ?? '/', 'http://localhost').pathname);
  let file = resolve(dist, '.' + path);
  if (!file.startsWith(dist + '/') && file !== dist) return res.writeHead(403).end();
  try { if (!statSync(file).isFile()) file = join(dist, 'index.html'); }
  catch { file = join(dist, 'index.html'); }
  res.writeHead(200, { 'Content-Type': mime[extname(file)] ?? 'application/octet-stream' }).end(readFileSync(file));
});
await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const profile = mkdtempSync(join(tmpdir(), 'nc03-event-browser-'));
const chrome = spawn(chromeBin, [
  '--headless=new', '--disable-gpu', '--no-sandbox', `--user-data-dir=${profile}`,
  '--remote-debugging-port=0', '--window-size=1280,1000', 'about:blank',
], { stdio: 'ignore' });
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
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
    try { return readFileSync(join(profile, 'DevToolsActivePort'), 'utf8').split('\n')[0]; }
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
    else if (msg.method === 'Runtime.consoleAPICalled' && msg.params.type === 'error') {
      const message = msg.params.args.map((x) => x.value ?? x.description).join(' ');
      if (!message.includes('engine worker error')) exceptions.push(message);
    }
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
    window.__fault = null;
    window.__faultInjected = false;
    window.__replies = [];
    window.Worker = class extends Original {
      constructor(...args) {
        super(...args);
        this.addEventListener('messageerror', () => { window.__messageErrorEventSeen = true; });
        this.addEventListener('message', event => {
          if (event.data.ok && event.data.data?.view) window.__replies.push(event.data.data.revision);
        });
      }
      postMessage(message, ...args) {
        if (message.type === 'autoStep' && window.__fault) {
          const fault = window.__fault;
          window.__fault = null;
          queueMicrotask(() => {
            window.__faultMeta = {fault, handler: typeof this.onmessageerror, hasProperty: 'onmessageerror' in this};
            this.dispatchEvent(fault === 'error'
              ? new ErrorEvent('error', {message: 'injected worker fault'})
              : new MessageEvent('messageerror'));
            window.__faultInjected = true;
          });
          return;
        }
        return super.postMessage(message, ...args);
      }
    };
  })()` });
  let failures = 0;
  for (const fault of ['error', 'messageerror']) {
    await send('Page.navigate', { url: origin + '/battle?players=4&seed=42&skipAnims=1&faultcase=' + fault });
    await wait('initial native match', () => evaluate(`location.search.includes('faultcase=${fault}') && !!document.querySelector('.toolbar .status')?.innerText.includes('native')`));
    const before = await evaluate('window.__replies.at(-1)');
    await evaluate(`window.__faultInjected = false; window.__fault = '${fault}'; [...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步').click()`);
    await wait('injected fault', () => evaluate('window.__faultInjected'));
    await pause(150);
    const state = await evaluate(`({
      busy: [...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步')?.disabled,
      alert: document.querySelector('.toolbar .error')?.innerText ?? null,
      meta: window.__faultMeta, eventSeen: window.__messageErrorEventSeen ?? false
    })`);
    if (!state.busy && state.alert?.includes('结果未知')) {
      console.log(`PASS ${fault}_visible: ${state.alert}`);
      const count = await evaluate('window.__replies.length');
      await evaluate("[...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步').click()");
      await wait('real retry response', () => evaluate(`window.__replies.length > ${count}`));
      await wait('idle after retry', () => evaluate("!![...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步' && !b.disabled)"));
      const after = await evaluate('window.__replies.at(-1)');
      if (after > before) console.log(`PASS ${fault}_retry: real Worker revision ${before} -> ${after}`);
      else { failures++; console.log(`FAIL ${fault}_retry: revision ${before} -> ${after}`); }
    } else {
      failures += 2;
      console.log(`FAIL ${fault}_visible: busy=${state.busy}; alert=${state.alert}; meta=${JSON.stringify(state.meta)}; eventSeen=${state.eventSeen}`);
      console.log(`FAIL ${fault}_retry: request remains pending`);
    }
  }
  assert.deepEqual(exceptions, []);
  console.log(`RESULT pass=${4 - failures} fail=${failures} unexpected_exceptions=${exceptions.length}`);
  if (failures) process.exitCode = 1;
} finally {
  ws?.close();
  if (chrome.exitCode === null) {
    const exited = new Promise((resolve) => chrome.once('exit', resolve));
    chrome.kill('SIGTERM');
    await Promise.race([exited, pause(2000)]);
  }
  await new Promise((resolve) => server.close(resolve));
  rmSync(profile, { recursive: true, force: true });
}
