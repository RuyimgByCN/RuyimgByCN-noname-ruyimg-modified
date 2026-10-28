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
    window.__requests = [];
    window.__replies = [];
    window.__rejectEnd = false;
    window.__rejectAuto = false;
    window.__injectedAuto = false;
    window.__injectCardProjection = false;
    window.Worker = class extends Original {
      constructor(...args) {
        super(...args);
        this.addEventListener('message', event => {
          const type = window.__requests.findLast(x => x.id === event.data.id)?.type;
          if (type === 'autoStep' && window.__rejectAuto && !window.__injectedAuto && event.data.ok) {
            event.data.data.result = { kind: 'answer', outcome: JSON.stringify({
              outcome: 'rejected', reason: 'InvalidChoice'
            }) };
            window.__injectedAuto = true;
          }
          if (type === 'endPhase' && window.__injectCardProjection && event.data.ok) {
            event.data.data.view.known_cards.push({id: 4294967295, def: 1, suit: 'Spade', rank: 1, owner: 1, zone: 'Hand', kind: 'Sha'});
            event.data.data.legal_actions.push({PlayCard: {card: 4294967295, targets: []}});
          }
          if (type) window.__replies.push({type, ok: event.data.ok, result: event.data.data?.result});
        });
      }
      postMessage(message, ...args) {
        window.__requests.push({id: message.id, type: message.type});
        if (message.type === 'endPhase' && window.__rejectEnd) {
          message = {...message, actor: (message.actor + 1) % 4};
        }
        return super.postMessage(message, ...args);
      }
    };
  })()` });
  await send('Page.navigate', { url: origin + '/battle?players=4&seed=42&skipAnims=1' });
  await wait('initial native match', () => evaluate("!!document.querySelector('.toolbar .status')?.innerText.includes('native')"));
  for (let step = 0; step < 100; step++) {
    if (await evaluate("!![...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '结束阶段' && !b.disabled)")) break;
    await wait('enabled auto-step', () => evaluate("!![...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步' && !b.disabled)"));
    const prior = await evaluate("window.__replies.filter(x => x.type === 'autoStep').length");
    await evaluate("[...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步').click()");
    await wait('step reply', () => evaluate(`window.__replies.filter(x => x.type === 'autoStep').length > ${prior}`));
    await wait('idle after step', () => evaluate("!![...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步' && !b.disabled)"));
  }
  assert.equal(await evaluate("!![...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '结束阶段' && !b.disabled)"), true, 'no end phase action for observer');
  const before = await evaluate("[...document.querySelectorAll('.match-stats .stat-value')].slice(0, 2).map(x => x.innerText)");
  await evaluate("window.__rejectEnd = true; window.__injectCardProjection = true; [...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '结束阶段').click()");
  await wait('rejected response', () => evaluate("window.__replies.some(x => x.type === 'endPhase')"));
  await wait('idle after rejection', () => evaluate("!![...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步' && !b.disabled)"));
  await pause(80);
  const rejection = await evaluate("window.__replies.findLast(x => x.type === 'endPhase')");
  const status = await evaluate("document.querySelector('.toolbar .status')?.innerText");
  const alert = await evaluate("document.querySelector('.toolbar .error')?.innerText ?? null");
  const after = await evaluate("[...document.querySelectorAll('.match-stats .stat-value')].slice(0, 2).map(x => x.innerText)");
  assert.equal(rejection.ok, true);
  assert.equal(rejection.result.outcome, 'rejected');
  assert.equal(rejection.result.reason, 'NotYourTurn');
  assert.deepEqual(after, before);
  let failures = 0;
  if (alert?.includes('还没有轮到该玩家') && status.includes('还没有轮到该玩家')) {
    console.log('PASS native_rejected_end_phase: real Worker NotYourTurn shown; revision/hash unchanged');
  } else {
    failures++;
    console.log(`FAIL native_rejected_end_phase: reason=${rejection.result.reason}; status=${status}; alert=${alert}`);
  }

  await wait('injected card', () => evaluate("!!document.querySelector('.hand button:not(:disabled)')"));
  await evaluate("document.querySelector('.hand button:not(:disabled)').click()");
  await wait('enabled play', () => evaluate("!!document.querySelector('[aria-label=\"无目标直接打出\"]:not(:disabled)')"));
  await evaluate("document.querySelector('[aria-label=\"无目标直接打出\"]').click()");
  await wait('play response or clone error', () => evaluate("window.__replies.some(x => x.type === 'playCard') || document.querySelector('.toolbar .error')?.innerText.includes('could not be cloned')"));
  await wait('idle after play', () => evaluate("!![...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '自动一步' && !b.disabled)"));
  const play = await evaluate(`({
    reply: window.__replies.findLast(x => x.type === 'playCard') ?? null,
    alert: document.querySelector('.toolbar .error')?.innerText ?? null,
    log: document.querySelector('[aria-label="事件日志"]')?.innerText ?? ''
  })`);
  if (play.reply?.ok && play.reply.result.outcome === 'rejected'
      && play.alert?.startsWith('操作被拒绝：') && !play.log.includes('出牌 #4294967295')) {
    console.log(`PASS reactive_play_reaches_worker: real Worker reason=${play.reply.result.reason}; projection injected, no array unwrapping`);
  } else {
    failures++;
    console.log(`FAIL reactive_play_reaches_worker: reply=${JSON.stringify(play.reply)}; alert=${play.alert}`);
  }

  await evaluate("window.__rejectAuto = true; [...document.querySelectorAll('.toolbar button')].find(b => b.textContent.trim() === '连续自动').click()");
  await wait('injected auto response', () => evaluate('window.__injectedAuto'));
  await pause(350);
  const auto = await evaluate(`({
    count: window.__requests.filter(x => x.type === 'autoStep').length,
    injectedAt: window.__replies.filter(x => x.type === 'autoStep').findIndex(x => x.result?.kind === 'answer' && x.result.outcome?.includes('InvalidChoice')),
    alert: document.querySelector('.toolbar .error')?.innerText ?? null,
    stopButton: [...document.querySelectorAll('.toolbar button')].some(b => b.textContent.trim() === '停止自动')
  })`);
  const autoReplies = await evaluate("window.__replies.filter(x => x.type === 'autoStep').map(x => x.result)");
  const postInjection = autoReplies.length - auto.injectedAt - 1;
  if (auto.alert?.includes('选择无效') && !auto.stopButton && postInjection === 0) {
    console.log('PASS continuous_auto_rejection: injected nested InvalidChoice visible; no subsequent steps');
  } else {
    failures++;
    console.log(`FAIL continuous_auto_rejection: alert=${auto.alert}; running=${auto.stopButton}; subsequent_replies=${postInjection}`);
  }
  assert.deepEqual(exceptions, []);
  console.log(`RESULT pass=${3 - failures} fail=${failures} exceptions=${exceptions.length}`);
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
