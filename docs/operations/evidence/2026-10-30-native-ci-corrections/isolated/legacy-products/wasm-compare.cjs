const fs = require('node:fs');
const assert = require('node:assert/strict');
const {WasmEngine} = require('./wasm/engine_wasm.js');
const lines = fs.readFileSync(__dirname+'/traces-after.log','utf8').trim().split('\n');
const groups = new Map();
for(const line of lines){const row=JSON.parse(line);if(row.dynamic || !('step' in row))continue;const key=JSON.stringify([row.ruleset,row.players,row.seed]); if(!groups.has(key))groups.set(key,[]);groups.get(key).push(row);}
let commands=0,checkpoints=0,events=0,choices=0;
for(const [key,rows] of groups){
 const engine = new WasmEngine(); const last=rows[rows.length-1]; const raw=Buffer.from(last.replay).toString('utf8'); const replay=JSON.parse(raw);
 for(const entry of replay.commands){
  const row=rows.find(r=>r.state.revision===entry.revision_after);assert(row, key+' missing native checkpoint');
  const env=JSON.parse(engine.submitJson(JSON.stringify(entry.command)));
  assert('Accepted' in env.result,key+' rejected');
  assert.equal(env.hash_hex,Buffer.from(row.hash).toString('hex'),key+' hash');
  assert.deepEqual(env.events,row.events,key+' events');
  assert.deepEqual(env.choice,row.choice,key+' choice');commands++;events+=env.events.length;choices+=env.choice!==null;
 }
 const restored = new WasmEngine(); const metadata=JSON.parse(restored.loadReplay(raw)); assert(metadata.chain_verified);
 for(const row of rows){const frame=JSON.parse(restored.seekReplay(row.state.revision,'full',1));assert.equal(frame.hash_hex,Buffer.from(row.hash).toString('hex'),key+' replay');assert.equal(frame.revision,row.state.revision);assert(frame.verified);checkpoints++;}
 engine.free();restored.free();
}
console.log(JSON.stringify({cases:groups.size,commands,events,choices,checkpoints,comparisons:'actual wasm32 old engine-core vs native: hashes/events/choices; full native state remains in traces',passed:true}));
