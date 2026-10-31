// Actual UI + real Worker/WASM: capture authority passively; no engine mocks.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtempSync, openSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { extname, join, resolve, basename } from 'node:path';
import { tmpdir } from 'node:os';

const [distArg, chromeBin, outArg] = process.argv.slice(2);
assert.ok(distArg && chromeBin && outArg, 'missing arguments');
const crypto = await import('node:crypto');
const requested=[];
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
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
  const bytes=readFileSync(file); requested.push({path,sha256:sha(bytes),bytes:bytes.length}); res.end(bytes);
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
          super(...args); capture.worker=this;
          this.addEventListener('message', ({data}) => {
            if (!data || !Number.isInteger(data.id)) return;
            capture.replies[data.id] = { ok: data.ok, error: data.error, data:data.data };
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

  const out=resolve(outArg);const reports=[];
  await send('Page.navigate',{url:origin+'/battle?players=3&skipAnims=1'});
  await wait('app connected',()=>current());
  const slot=async(name)=>{
    const hex=readFileSync(join(out,name+'.ngsa')).toString('hex');
    await evaluate(`localStorage.setItem('noname-next.native-saves.v1',JSON.stringify([{id:'probe',label:'NC10 formal fixture',hex:${JSON.stringify(hex)},players:3,seed:2306,revision:1,contentLock:[],lockSeed:'',savedAt:'2026-10-31T11:25:00Z'}]))`);
    const frame=await clickRequest('读档','restore');assert.match(frame.hash_hex,/^[0-9a-f]{64}$/);return frame;
  };
  // Every UI fixture is loaded through the real production restore button.
  let frame=await slot('park-0');assert.equal(frame.choice.source_skill,'遗计');assert.equal(frame.choice.allow_cancel,true);
  await wait('own optional UI',()=>evaluate(`document.querySelector('.choice h2')?.textContent==='是否发动【遗计】？'`));
  await send('Page.captureScreenshot',{format:'png'}).then(r=>writeFileSync(join(out,'own-optional.png'),Buffer.from(r.data,'base64')));const before=frame.revision;frame=await clickRequest('选择 0','resolveChoice');assert.equal(frame.revision,before+1);assert.equal(frame.result.outcome,'pending');
  reports.push({case:'UI-own-confirm',frame});
  frame=await slot('park-0');const cancelBefore=frame.revision;frame=await clickRequest('取消','resolveChoice');assert.equal(frame.revision,cancelBefore+1);assert.ok(['accepted','pending'].includes(frame.result.outcome));assert.equal(frame.choice,null);assert.equal(frame.hash_hex,JSON.parse(readFileSync(join(out,'native-probe-results.json')))['own-cancel'].hash);reports.push({case:'UI-optional-cancel',frame});
  frame=await slot('shan');const cardToken=frame.choice.options.find(x=>x.label_key.startsWith('card:')).token;const action=frame.legal_actions.find(x=>x.ResolveChoice&&!x.ResolveChoice.cancelled&&x.ResolveChoice.selections.includes(cardToken));assert.ok(action);const cardLabel='选择 '+action.ResolveChoice.selections.join(', ');
  const shanBefore=frame.revision;frame=await clickRequest(cardLabel,'resolveChoice');assert.equal(frame.revision,shanBefore+1);assert.equal(frame.hash_hex,JSON.parse(readFileSync(join(out,'native-probe-results.json'))).direct.hash);reports.push({case:'UI-Shan-card',frame});
  frame=await slot('shan');const autoBefore=frame.revision;frame=await clickRequest('自动一步','autoStep');assert.equal(frame.revision,autoBefore+1);assert.equal(frame.hash_hex,JSON.parse(readFileSync(join(out,'native-probe-results.json'))).auto.hash);reports.push({case:'UI-Shan-auto-once',frame});
  frame=await slot('native-park-1');assert.equal(frame.choice,null);await wait('Other hidden in UI',()=>evaluate(`document.querySelector('.choice')===null`));reports.push({case:'UI-Other-hidden',frame});
  // Separate real dedicated Worker, same production chunk and current WASM bytes.
  const workerName=readdirSync(join(dist,'assets')).find(n=>/^engine\.native\.worker-.*\.js$/.test(n));assert.ok(workerName);
  await evaluate(`(()=>{window.__probeWorker=new Worker(${JSON.stringify('/assets/') }+${JSON.stringify(workerName)},{type:'module'});window.__probeSeq=800000;window.__probeSend=msg=>new Promise((resolve,reject)=>{const id=window.__probeSeq++;const timer=setTimeout(()=>reject(Error('worker timeout')),30000);const listen=ev=>{if(ev.data.id!==id)return;clearTimeout(timer);window.__probeWorker.removeEventListener('message',listen);resolve(ev.data);};window.__probeWorker.addEventListener('message',listen);window.__probeWorker.postMessage({...msg,id});});})()`);
  const request=async msg=>{const x=await evaluate(`window.__probeSend(${JSON.stringify(msg)})`);assert.equal(x.ok,true,x.error);return x.data;};
  await request({type:'init'});
  const native=JSON.parse(readFileSync(join(out,'native-probe-results.json')));
  for(const name of ['park-0','park-1','own-cancel','shan-auto','shan-direct','shan-cancel','shan-pass']){
    await request({type:'start',players:3,seed:2306,observer:1});const fixture=name==='own-cancel'?'park-0':name.startsWith('park')?name:'shan';
    const baseline=await request({type:'restore',hex:readFileSync(join(out,fixture+'.ngsa')).toString('hex')});assert.ok(baseline.choice);
    const choice=baseline.choice;const cardToken=choice.options.find(x=>x.label_key.startsWith('card:'))?.token;const command=name.endsWith('pass')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[choice.options.find(x=>x.label_key==='pass').token],cancelled:false}:name.endsWith('direct')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[cardToken],cancelled:false}:name.endsWith('cancel')?{type:'resolveChoice',actor:1,choiceId:choice.choice_id,selections:[],cancelled:true}:{type:'autoStep'};
    const next=await request(command);assert.equal(next.revision,baseline.revision+(name==='shan-cancel'?0:1));const expected=native[name]??native[name.replace('shan-','')];assert.equal(next.hash_hex,expected.hash);
    assert.deepEqual(next.result,expected.reply);const archive=Buffer.from((await request({type:'save'})).hex,'hex');assert.deepEqual(archive,readFileSync(join(out,'native-'+name+'.ngsa')));
    reports.push({case:'dedicated-worker-'+name,baseline,next,fullArchiveSha256:sha(archive)});
  }
  await evaluate('window.__probeWorker.terminate()');
  await send('Page.captureScreenshot',{format:'png'}).then(r=>writeFileSync(join(out,'product.png'),Buffer.from(r.data,'base64')));
  const rejected=await evaluate('window.__nativeProduct.rejected');writeFileSync(join(out,'browser-results.json'),JSON.stringify({reports,exceptions,rejected,requested,workerName,dist,runDir},null,2));
  assert.deepEqual(rejected,[]);assert.deepEqual(exceptions,[]);assert.ok(requested.some(x=>x.path.endsWith('.wasm')));console.log(JSON.stringify({passed:reports.length,exceptions:0,rejected:0,realUI:true,realDedicatedWorker:true,currentProductionDist:dist}));
}finally{ws?.close();chrome.kill('SIGTERM');await new Promise(resolve=>server.close(resolve));}
