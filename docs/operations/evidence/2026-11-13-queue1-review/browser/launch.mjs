// Upstream launch recipe: apps/web-client/scripts/tabletop-comparison-cdp.mjs.
// A temporary localhost CDP session for recorded, real upstream event probes.
import { spawn, execFileSync } from 'node:child_process';
import { createServer } from 'node:http';
import { readFileSync, writeFileSync, appendFileSync, existsSync, statSync, mkdtempSync, rmSync } from 'node:fs';
import { resolve, dirname, extname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import assert from 'node:assert/strict';

const evidence = dirname(fileURLToPath(import.meta.url));
const label = process.env.QUEUE1_LABEL || 'liuli';
assert(/^[a-z]+$/.test(label));
const upstream = resolve(evidence, '../../../../../../archive/upstream/noname-liuli/noname');
assert(existsSync(join(upstream, 'game/game.js')), upstream);
const profile = mkdtempSync(join(tmpdir(), 'noname-queue1-chrome-'));
const transcript = join(evidence, `session-${label}-${Date.now()}`);
const mime = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.jpg': 'image/jpeg', '.woff2': 'font/woff2', '.mp3': 'audio/mpeg', '.ogg': 'audio/ogg' };
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
  if (/spine/i.test(path)) { res.writeHead(404); res.end('Decision 35 guard'); return; }
  const file = resolve(upstream, '.' + (path === '/' ? '/index.html' : path));
  if (!file.startsWith(upstream + '/') || !existsSync(file) || statSync(file).isDirectory()) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { 'Content-Type': mime[extname(file)] || 'application/octet-stream', 'Cache-Control': 'no-store' });
  res.end(readFileSync(file));
});
await new Promise(r => server.listen(0, '127.0.0.1', r));
const chrome = spawn(process.env.CHROME_BIN || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', [
  '--headless=new', '--enable-webgl', '--ignore-gpu-blocklist', '--no-sandbox', '--autoplay-policy=no-user-gesture-required',
  `--user-data-dir=${profile}`, '--remote-debugging-port=0', '--window-size=1440,1080', 'about:blank',
], { stdio: 'ignore' });
const delay = ms => new Promise(r => setTimeout(r, ms));
let port;
for (let i = 0; i < 100; i++) {
  const active = join(profile, 'DevToolsActivePort');
  if (existsSync(active)) { port = Number(readFileSync(active, 'utf8').split('\n')[0]); break; }
  await delay(100);
}
assert(Number.isInteger(port), 'Chrome did not expose CDP');
const pages = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
const ws = new WebSocket(pages.find(p => p.type === 'page').webSocketDebuggerUrl);
await new Promise((r, j) => { ws.onopen = r; ws.onerror = j; });
let id = 0;
const pending = new Map();
const send = (method, params = {}) => {
  const requestId = ++id;
  return new Promise((resolve, reject) => {
    pending.set(requestId, { resolve, reject });
    ws.send(JSON.stringify({ id: requestId, method, params }));
  });
};
ws.onmessage = e => {
  const msg = JSON.parse(e.data);
  if (msg.id && pending.has(msg.id)) {
    const p = pending.get(msg.id); pending.delete(msg.id);
    msg.error ? p.reject(new Error(JSON.stringify(msg.error))) : p.resolve(msg.result);
  } else if (msg.method === 'Runtime.exceptionThrown' || msg.method === 'Log.entryAdded') {
    appendFileSync(transcript + '-events.jsonl', JSON.stringify(msg) + '\n');
  } else if (msg.method === 'Page.javascriptDialogOpening') {
    appendFileSync(transcript + '-events.jsonl', JSON.stringify(msg) + '\n');
    send('Page.handleJavaScriptDialog', { accept: true });
  }
};
await send('Runtime.enable');
await send('Page.enable');
await send('Log.enable');
const config = { dev: true, mode: 'identity', continue_name: ['caocao'], new_tutorial: true, show_splash: false,
  game_speed: 'vvfast', duration: 0, extensions: [], plays: [], theme: 'simple', layout: 'mobile',
  characters: ['standard'], cards: ['standard'], mode_config: { identity: { player_number: 4 } } };
await send('Page.addScriptToEvaluateOnNewDocument', { source: `
  window.alert = () => {}; window.confirm = () => true; window.prompt = () => null;
  window.bannedExtensions = []; window.__dirname = '';
  window.resolveLocalFileSystemURL = (u,s,e) => e && e(new Error('not a file system'));
  localStorage.removeItem('noname_inited');
  localStorage.setItem('noname_0.9_disable_extension','true');
  for (const k of ['directstart','noname_0.9_directstart','noname_0.9_nodb','nodb']) localStorage.setItem(k,'true');
  for (const k of ['noname_0.9_config','config']) localStorage.setItem(k,${JSON.stringify(JSON.stringify(config))});
` });
const url = process.env.QUEUE1_UPSTREAM_URL || `http://127.0.0.1:${server.address().port}/`;
await send('Page.navigate', { url });
const control = createServer(async (req, res) => {
  if (req.method !== 'POST') { res.writeHead(405); res.end(); return; }
  let body = ''; for await (const chunk of req) body += chunk;
  const at = new Date().toISOString();
  appendFileSync(transcript + '-requests.jsonl', JSON.stringify({ at, expression: body }) + '\n');
  try {
    const result = await send('Runtime.evaluate', { expression: body, awaitPromise: true, returnByValue: true });
    const reply = { at, result };
    appendFileSync(transcript + '-replies.jsonl', JSON.stringify(reply) + '\n');
    res.writeHead(200, { 'Content-Type': 'application/json' }); res.end(JSON.stringify(reply));
  } catch (error) { res.writeHead(500); res.end(String(error)); }
});
await new Promise(r => control.listen(0, '127.0.0.1', r));
const meta = { at: new Date().toISOString(), label, url, upstream, profile, upstreamPort: server.address().port,
  controlPort: control.address().port, cdpPort: port, chromePid: chrome.pid, transcript,
  head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim() };
writeFileSync(evidence + `/active-session-${label}.json`, JSON.stringify(meta, null, 2) + '\n');
console.log(JSON.stringify(meta));
async function close() {
  control.close(); server.close(); ws.close(); chrome.kill();
  await delay(500); rmSync(profile, { recursive: true, force: true }); process.exit(0);
}
process.on('SIGINT', close); process.on('SIGTERM', close);
