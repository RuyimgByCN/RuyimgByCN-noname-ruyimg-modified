import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createServer } from 'node:http';
import { readFile, writeFile, mkdir, mkdtemp, cp, rm, rename } from 'node:fs/promises';
import { createWriteStream } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, resolve, join, extname } from 'node:path';
import { fileURLToPath } from 'node:url';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const root = resolve(process.env.NONAME_REPO_ROOT || process.cwd());
const out = dirname(fileURLToPath(import.meta.url));
const distRoot = resolve(process.env.NONAME_WEB_DIST || join(root, 'apps/web-client/dist'));
const data = await mkdtemp(join(tmpdir(), 'noname-ws-crash-browser-'));
const serverPort = Number(process.env.NONAME_WS_PORT || 5484);
const webPort = Number(process.env.NONAME_WEB_PORT || 5483);
const origin = `http://127.0.0.1:${webPort}`;
const wsUrl = `ws://127.0.0.1:${serverPort}/ws`;
const results = [], errors = [], pages = [];
const check = (name, details) => { results.push({ name, details }); console.log(`PASS ${name}: ${JSON.stringify(details)}`); };
const delay = ms => new Promise(r => setTimeout(r, ms));
async function until(f, label) {
  for (let n = 0; n < 200; n++) { const v = await f(); if (v) return v; await delay(100); }
  throw new Error(`timeout: ${label}`);
}
const mime = { '.html':'text/html', '.js':'text/javascript', '.css':'text/css', '.wasm':'application/wasm', '.json':'application/json' };
const staticServer = createServer(async (req, res) => {
  try {
    const path = resolve(distRoot, `.${new URL(req.url, origin).pathname}`);
    if (path !== distRoot && !path.startsWith(distRoot + '/')) throw new Error('invalid path');
    let body, target = path;
    try { body = await readFile(target); } catch { target = join(distRoot, 'index.html'); body = await readFile(target); }
    res.setHeader('Content-Type', mime[extname(target)] || 'application/octet-stream'); res.end(body);
  } catch { res.writeHead(404); res.end(); }
});
await new Promise((r,j) => staticServer.once('error', j).listen(webPort, '127.0.0.1', r));
let game, browser;
const cfg = {
  ...process.env,
  GAME_SERVER_ADDR: `127.0.0.1:${serverPort}`,
  NONAME_ORIGINAL_CONTENT_ROOT: join(data, 'original'),
  NONAME_CONTENT_STORE_ROOT: join(data, 'legacy-store'),
  NONAME_EXTENSIONS_ROOT: join(data, 'legacy-ext'),
  NONAME_NATIVE_PACKAGES_ROOT: join(data, 'packages'),
  NONAME_NATIVE_STORE_ROOT: join(data, 'native-store'),
  NONAME_NATIVE_ROOM_STORE: join(data, 'rooms'),
  NONAME_NATIVE_ROOMS: '1', NONAME_NATIVE_ROOM_NAMESPACE: 'upstream-noname',
  NONAME_NATIVE_ROOM_PACKAGES: 'shahai-swd,wudian',
};
for (const p of ['original','legacy-ext','packages']) await mkdir(join(data,p));
for (const name of ['shahai-swd-upstream-noname','wudian-upstream-noname']) {
  await cp(join(root,'packages/extensions',name),join(data,'packages',name),{recursive:true});
}
async function startServer(tag) {
  const log = createWriteStream(join(out, `browser-server-${tag}.log`));
  game = spawn(join(root, 'target/debug/game-server'), [], { cwd: root, env: cfg, stdio:['ignore', 'pipe','pipe'] });
  game.stdout.pipe(log, {end:false}); game.stderr.pipe(log, {end:false});
  game.once('exit', () => log.end());
  await until(async () => {
    if (game.exitCode != null) throw new Error(`game-server exit ${game.exitCode}`);
    try { return (await fetch(`http://127.0.0.1:${serverPort}/health`)).ok; } catch { return false; }
  }, 'server health');
}
async function stopServer(signal = 'SIGTERM') {
  const p = game; game = undefined;
  if (!p || p.exitCode != null) return;
  return await new Promise(r => { p.once('exit', (code, killedBy) => r({code,signal:killedBy})); p.kill(signal); });
}
function track(page) {
  const frames = [], sent = [];
  page.on('pageerror', e => errors.push(String(e)));
  page.on('websocket', socket => {
    socket.on('framereceived', ({payload}) => { try { frames.push(JSON.parse(String(payload))); } catch {} });
    socket.on('framesent', ({payload}) => { try { sent.push(JSON.parse(String(payload))); } catch {} });
  });
  const latest = () => {
    for (let i=frames.length-1;i>=0;i--) if (frames[i].payload?.view) return frames[i].payload;
  };
  pages.push(page); return {page, frames, sent, latest};
}
async function enter(client, action, name, password, id=1) {
  const p = client.page;
  await p.goto(origin);
  await p.getByLabel('玩家人数').selectOption('2');
  await p.getByLabel('随机种子').fill('4242');
  await p.locator('input[type=url]').fill(wsUrl);
  await p.locator('input[autocomplete=nickname]').fill(name);
  await p.locator('input[type=password]').fill(password);
  await p.locator('input[inputmode=numeric]').fill(String(id));
  await p.getByRole('button',{name:action,exact:true}).click();
  const joined = await until(() => client.frames.find(f=>f.type==='JoinOk')?.payload, `${name} JoinOk`);
  await p.locator('section.toolbar').waitFor({state:'visible'});
  return joined;
}
async function synced(clients, rev) {
  await until(() => clients.every(c=>c.latest()?.view.revision===rev), `all views revision ${rev}`);
  const hashes=clients.map(c=>JSON.stringify(c.latest().view.state_hash));
  assert.ok(hashes.every(h=>h===hashes[0]));
}
try {
  await startServer('initial');
  browser = await chromium.launch({headless:true,channel:'chrome'});
  const a=track(await (await browser.newContext()).newPage());
  const b=track(await (await browser.newContext()).newPage());
  const s=track(await (await browser.newContext()).newPage());
  const aj=await enter(a,'创建并入座','ws-owner','room-secret');
  const bj=await enter(b,'加入房间','ws-peer','room-secret',aj.room_id);
  const sj=await enter(s,'旁观','ws-spectator','',aj.room_id);
  assert.equal(aj.player_id,1); assert.equal(bj.player_id,2); assert.equal(sj.player_id,0);
  assert.equal(sj.view.known_cards.length,0); assert.equal(sj.legal_actions.length,0);
  for (const j of [aj,bj]) {
    assert.ok(j.view.known_cards.length>0);
    assert.ok(j.view.known_cards.every(c=>c.owner===j.player_id));
  }
  await synced([a,b,s], aj.view.revision);
  const recordPath=join(data,'rooms',`${aj.room_id}.json`);
  let record=await until(async()=>{try{return JSON.parse(await readFile(recordPath,'utf8'));}catch{return false;}},'room persisted');
  assert.equal(record.state.pack_refs.length,2);
  assert.equal(record.state.frozen_packages?.length,2,'new rooms freeze actual package bytes');
  const frozenBefore=JSON.stringify(record.state.frozen_packages);
  const refsBefore=JSON.stringify(record.state.pack_refs);
  assert.ok(record.state.pack_refs.some(p=>p.includes('shahai-swd')));
  assert.ok(record.state.pack_refs.some(p=>p.includes('wudian')));
  check('production_ui_formal_room_and_private_views',{room:aj.room_id, packs:record.state.pack_refs, handCounts:[aj.view.known_cards.length,bj.view.known_cards.length,0]});
  async function endPhaseFromUi() {
    const before=a.latest().view.revision;
    await a.page.waitForFunction(()=>!Array.from(document.querySelectorAll('.toolbar button')).find(b=>b.textContent.trim()==='结束阶段')?.disabled);
    await a.page.getByRole('button',{name:'结束阶段',exact:true}).click();
    await until(()=>a.latest().view.revision>before,'UI command accepted');
    await synced([a,b,s],a.latest().view.revision);
  }
  await endPhaseFromUi();
  const revision=a.latest().view.revision;
  check('real_ws_personalized_state_sync',{revision,hash:a.latest().view.state_hash});
  await endPhaseFromUi();
  await endPhaseFromUi();
  const pendingChoice=a.latest().choice;
  assert.ok(pendingChoice,'third phase must open a private discard choice');
  assert.equal(b.latest().choice,null); assert.equal(s.latest().choice,null);
  assert.ok(a.latest().legal_actions.some(x=>x.ResolveChoice));
  check('real_ws_private_choice_barrier',{choice:pendingChoice.choice_id,owner:pendingChoice.observer,options:pendingChoice.options.length});
  const priorJoins=a.frames.filter(f=>f.type==='JoinOk').length;
  await a.page.reload();
  await until(()=>a.frames.filter(f=>f.type==='JoinOk').length>priorJoins,'reload token rejoin');
  assert.equal(a.frames.filter(f=>f.type==='JoinOk').at(-1).payload.room_id,aj.room_id);
  assert.equal(a.frames.filter(f=>f.type==='JoinOk').at(-1).payload.player_id,1);
  assert.equal(a.frames.filter(f=>f.type==='JoinOk').at(-1).payload.reconnect_token,aj.reconnect_token);
  assert.ok(a.sent.some(f=>f.type==='JoinRoom'&&f.payload.reconnect_token===aj.reconnect_token));
  check('page_reload_reuses_original_room',{room:aj.room_id});
  assert.deepEqual(a.latest().choice,pendingChoice);
  const saved=a.latest().view;
  const savedActions=a.latest().legal_actions;
  await until(async()=>{
    const r=JSON.parse(await readFile(recordPath,'utf8'));
    return r.state.snapshot_hex!==record.state.snapshot_hex;
  },'updated snapshot persisted');
  const pendingSnapshotHex=JSON.parse(await readFile(recordPath,'utf8')).state.snapshot_hex;
  const savedStates=[a,b,s].map(c=>c.latest());
  const failedAt=[a,b,s].map(c=>c.frames.length);
  const confirmedBytes=await readFile(recordPath);
  const confirmedDir=join(data,'rooms-confirmed');
  await rename(join(data,'rooms'),confirmedDir);
  await writeFile(join(data,'rooms'),'injected write failure: directory blocked');
  await a.page.locator('section.choice button').first().click();
  await until(()=>a.frames.slice(failedAt[0]).some(f=>f.type==='Error'&&f.payload.code==='Internal'),'write failure returned as protocol error');
  assert.deepEqual(await readFile(join(confirmedDir,`${aj.room_id}.json`)),confirmedBytes);
  await delay(200);
  for (const [i,c] of [a,b,s].entries()) {
    assert.deepEqual(c.latest(),savedStates[i],'failed write must keep confirmed personalized state');
    assert.equal(c.frames.slice(failedAt[i]).filter(f=>f.type==='CommandOutcome').length,0,'no accepted reply or fanout before disk success');
  }
  check('write_failure_refuses_success_and_fanout',{revision:saved.revision,confirmedFile:'unchanged'});
  const joins=pages.map((_,i)=>[a,b,s][i].frames.filter(f=>f.type==='JoinOk').length);
  const killed=await stopServer('SIGKILL');
  assert.equal(killed.signal,'SIGKILL');
  await rm(join(data,'rooms'));
  await rename(confirmedDir,join(data,'rooms'));
  const partialPath=join(data,'rooms',`${aj.room_id}.json.tmp`);
  const completePath=join(data,'rooms',`${aj.room_id}.json.80aca046-976f-4a1a-abf7-410f396f113f.tmp`);
  const corruptPath=join(data,'rooms','999.json');
  const partialBytes=Buffer.from('{"room_id":1,"state":');
  const completeBytes=Buffer.from(JSON.stringify(record));
  const corruptBytes=Buffer.from('{"room_id":999,"state":');
  await writeFile(partialPath,partialBytes);
  await writeFile(completePath,completeBytes);
  await writeFile(corruptPath,corruptBytes);
  check('sigkill_and_uncommitted_temp_files',{signal:killed.signal,partialTemp:true,completeUncommittedTemp:true,corruptRoom:999});
  await startServer('restored');
  await until(()=>[a,b,s].every((c,i)=>c.frames.filter(f=>f.type==='JoinOk').length>joins[i]),'all browsers automatically rejoin restored room');
  await synced([a,b,s],saved.revision);
  assert.deepEqual(a.latest().view,saved);
  assert.deepEqual(a.latest().choice,pendingChoice);
  assert.deepEqual(a.latest().legal_actions,savedActions);
  for (const [client, initial] of [[a,aj],[b,bj],[s,sj]]) {
    const joined=client.frames.filter(f=>f.type==='JoinOk').at(-1).payload;
    assert.equal(joined.room_id,initial.room_id); assert.equal(joined.player_id,initial.player_id);
    assert.equal(joined.role,initial.role); assert.equal(joined.seat,initial.seat);
    if (joined.role==='Player') assert.equal(joined.reconnect_token,initial.reconnect_token);
  }
  assert.equal(s.frames.filter(f=>f.type==='JoinOk').at(-1).payload.role,'Spectator');
  assert.equal(s.latest().view.known_cards.length,0);
  const roster=a.frames.filter(f=>f.type==='RoomRoster').at(-1).payload;
  assert.equal(roster.seats.length,2); assert.equal(new Set(roster.seats.map(x=>x.seat)).size,2);
  check('sigkill_restore_confirmed_choice_and_roles',{revision:saved.revision,players:roster.seats.length,spectatorRole:'Spectator'});
  await a.page.locator('section.choice button').first().click();
  await until(()=>a.latest().view.revision>saved.revision,'restored game continues');
  await synced([a,b,s],a.latest().view.revision);
  await until(async()=>{
    const disk=JSON.parse(await readFile(recordPath,'utf8'));
    if (disk.state.snapshot_hex===pendingSnapshotHex) return false;
    assert.equal(JSON.stringify(disk.state.frozen_packages),frozenBefore,'continued save keeps original copies');
    assert.equal(JSON.stringify(disk.state.pack_refs),refsBefore,'continued save keeps original content lock');
    return true;
  },'continued room persists frozen copies');
  check('restored_room_continues_over_ws',{revision:a.latest().view.revision,resolvedChoice:pendingChoice.choice_id});
  assert.deepEqual(await readFile(partialPath),partialBytes);
  assert.deepEqual(await readFile(completePath),completeBytes);
  assert.deepEqual(await readFile(corruptPath),corruptBytes);
  const newcomer=track(await (await browser.newContext()).newPage());
  const newRoom=await enter(newcomer,'创建并入座','new-owner','new-secret');
  assert.equal(newRoom.room_id,1000,'corrupt saved room ID must be reserved');
  assert.deepEqual(await readFile(corruptPath),corruptBytes);
  check('corrupt_room_isolated_and_id_reserved',{healthyRoom:aj.room_id,newRoom:newRoom.room_id,corruptFile:'unchanged',tempFiles:'unchanged'});
  await a.page.waitForFunction(()=>!document.querySelector('section.choice'));
  await delay(200);
  await a.page.screenshot({path:join(out,'browser-ws-restored.png'),fullPage:true});
  assert.deepEqual(errors,[]);
  check('uncaught_page_errors',errors.length);
  await writeFile(join(out,'browser-report.json'),JSON.stringify({results,errors,data,origin,wsUrl,browserVersion:browser.version(),servedDist:distRoot},null,2));
} finally {
  await browser?.close(); await stopServer(); await new Promise(r=>staticServer.close(r));
}
