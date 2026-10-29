// Actual UI + real Worker/WASM: capture authority passively; no engine mocks.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtempSync, openSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { extname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';

const [distArg, chromeBin, outArg] = process.argv.slice(2);
assert.ok(distArg && chromeBin && outArg, 'missing arguments');
const screenshot = join(resolve(outArg), "native-product.png");
const dist = resolve(distArg);
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
  await send('Page.addScriptToEvaluateOnNewDocument', { source: `
    (() => {
      const capture = window.__nativeProduct = { sent: {}, replies: {}, latest: null, frames: 0, rejected: [] };
      const OriginalWorker = window.Worker;
      window.Worker = class extends OriginalWorker {
        constructor(...args) {
          super(...args);
          this.addEventListener('message', ({data}) => {
            if (!data || !Number.isInteger(data.id)) return;
            capture.replies[data.id] = { ok: data.ok, error: data.error };
            if (data.ok === false) capture.rejected.push(data.error);
            if (data.data?.hash_hex && typeof data.data?.finished === 'boolean') {
              capture.latest = data.data;
              capture.frames++;
            }
          });
        }
        postMessage(...args) {
          const message = args[0];
          if (message && Number.isInteger(message.id)) capture.sent[message.type] = message.id;
          return super.postMessage(...args);
        }
      };
    })();
  ` });
  const current = () => evaluate('window.__nativeProduct?.latest ?? null');
  const clickRequest = async (label, type) => {
    const previous = await evaluate(`window.__nativeProduct.sent[${JSON.stringify(type)}] ?? 0`);
    await wait(`${label} enabled`, () => evaluate(`(() => {
      const button = [...document.querySelectorAll('button')].find(b => b.textContent.trim() === ${JSON.stringify(label)});
      return button && !button.disabled;
    })()`));
    assert.equal(await evaluate(`(() => {
      const button = [...document.querySelectorAll('button')].find(b => b.textContent.trim() === ${JSON.stringify(label)});
      button.click(); return true;
    })()`), true);
    const request = await wait(`${type} request sent`, () => evaluate(`(() => {
      const id = window.__nativeProduct.sent[${JSON.stringify(type)}];
      return id > ${previous} ? id : null;
    })()`));
    const reply = await wait(`${type} response`, () => evaluate(`window.__nativeProduct.replies[${request}] ?? null`));
    assert.equal(reply.ok, true, `${label}: ${reply.error}`);
    return current();
  };
  const checkPrivacy = (frame) => {
    assert.match(frame.hash_hex, /^[0-9a-f]{64}$/);
    for (const card of frame.view.known_cards) {
      assert.notEqual(card.zone, 'DrawPile', 'local player must not receive the draw pile');
      if (card.zone === 'Hand') assert.equal(card.owner, frame.view.observer, 'foreign hand leaked');
    }
  };
  const reports = [];
  for (const mode of ['prototype', 'identity']) {
    await send('Page.navigate', { url: origin + '/' });
    await wait('home controls', () => evaluate(`!!document.querySelector('[aria-label="对局模式"]')`));
    await evaluate(`(() => {
      const select = document.querySelector('[aria-label="对局模式"]');
      select.value = ${JSON.stringify(mode)}; select.dispatchEvent(new Event('change', {bubbles:true}));
    })()`);
    await clickRequest('本地开局', 'start');
    await evaluate(`(() => {
      const label = [...document.querySelectorAll('label')].find(x => x.textContent.includes('跳过动画'));
      const checkbox = label?.querySelector('input[type="checkbox"]');
      if (checkbox && !checkbox.checked) checkbox.click();
    })()`);
    let frame = await current();
    assert.equal(frame.view.seats.length, 4);
    checkPrivacy(frame);
    const saved = { hash: frame.hash_hex, revision: frame.revision };
    await clickRequest('存档', 'save');
    console.log(`PASS ${mode}_save_via_ui: revision=${saved.revision}`);
    if (!frame.finished) await clickRequest('自动一步', 'autoStep');
    frame = await clickRequest('读档', 'restore');
    assert.equal(frame.hash_hex, saved.hash, 'UI restore must return to the exact saved state');
    assert.equal(frame.revision, saved.revision, 'UI restore must retain revision');
    console.log(`PASS ${mode}_restore_exact_hash: ${saved.hash}`);
    let steps = 0;
    while (!frame.finished && steps < 2000) {
      frame = await clickRequest('自动一步', 'autoStep');
      checkPrivacy(frame);
      steps++;
    }
    assert.equal(frame.finished, true, `${mode}: no authoritative terminal state after ${steps} steps`);
    assert.ok(frame.view.outcome, 'terminal envelope must include its real outcome');
    const winner = frame.view.outcome.SingleWin?.player;
    const self = frame.view.seats.find(seat => seat.player === frame.view.observer);
    let expectedSelfWin;
    let expectedLabel;
    if (mode === 'prototype') {
      assert.ok(Number.isInteger(winner) && frame.view.seats.some(seat => seat.player === winner), 'winner must use the displayed local seat IDs');
      assert.ok(frame.view.seats.find(seat => seat.player === winner).alive, 'the announced winner must actually be alive');
      expectedSelfWin = winner === self.player;
      expectedLabel = `单挑胜者 P${winner}`;
    } else {
      assert.ok(self.identity, 'the local observer must receive its own identity');
      if ('RebelWin' in frame.view.outcome) {
        expectedSelfWin = self.identity === 'Rebel';
        expectedLabel = '反贼胜';
      } else if ('LoyalistWin' in frame.view.outcome) {
        expectedSelfWin = self.identity === 'Lord' || self.identity === 'Loyalist';
        expectedLabel = '主公方胜';
      } else if ('RenegadeWin' in frame.view.outcome) {
        expectedSelfWin = self.identity === 'Renegade';
        expectedLabel = '内奸胜';
      } else {
        assert.fail(`unexpected identity terminal result: ${JSON.stringify(frame.view.outcome)}`);
      }
    }
    const banner = await wait(`${mode} terminal banner rendered`, () => evaluate(`(() => {
      const element = document.querySelector('.outcome-banner');
      return element ? {text:element.innerText, win:element.classList.contains('win'), lose:element.classList.contains('lose')} : null;
    })()`));
    assert.ok(banner.text.includes(expectedLabel), JSON.stringify(banner));
    assert.equal(banner.win, expectedSelfWin, `${mode}: actual observer victory class`);
    assert.equal(banner.lose, !expectedSelfWin, `${mode}: actual observer defeat class`);
    assert.equal(banner.text.includes('🎉'), expectedSelfWin, `${mode}: actual observer victory icon`);
    console.log(`PASS ${mode}_actual_terminal_banner: ${JSON.stringify({banner, expectedSelfWin, observer:self.player, identity:self.identity})}`);
    assert.deepEqual(await evaluate('window.__nativeProduct.rejected'), []);
    reports.push({mode, seed:42, players:4, skipAnimations:true, steps, revision:frame.revision, hash:frame.hash_hex, outcome:frame.view.outcome, expectedSelfWin, banner, saved});
    console.log(`PASS ${mode}_authoritative_terminal: steps=${steps} revision=${frame.revision} hash=${frame.hash_hex}`);
  }
  writeFileSync(join(resolve(outArg), 'native-product-report.json'), JSON.stringify({reports, exceptions}, null, 2));
  await send('Page.captureScreenshot', { format: 'png' }).then((r) => writeFileSync(screenshot, Buffer.from(r.data, 'base64')));
  assert.deepEqual(exceptions, []);
  console.log('PASS browser_exceptions: 0');
  console.log(`run_dir=${runDir} origin=${origin}`);
} finally {
  ws?.close();
  chrome.kill('SIGTERM');
  await new Promise((resolve) => server.close(resolve));
}
