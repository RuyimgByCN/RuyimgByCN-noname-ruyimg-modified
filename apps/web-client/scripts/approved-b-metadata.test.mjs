import assert from 'node:assert/strict';
import { test } from 'node:test';
import { nativeEnvelope, nativeObserverToSnapshot } from '../src/worker/nativeBackend.ts';
const general = { def: 0, name: '曹操', faction: 'wei', revealed: false };
const view = { seat: 0, players: [{ seat: 0, characterDef: 0, generals: [general, null], hp: 4, maxHp: 4, alive: true, hand: { own: [] }, handCount: 0, equips: [], judge: [], skills: [] }], discard: [], table: [], phase: 'play', turn: 0, round: 1 };
function envelope(choice) {
 return nativeEnvelope({ observer: 0, engine: { view_json: () => JSON.stringify(view), open_choice_json: () => JSON.stringify(choice == null ? choice : {min:0,max:0,...choice}), revision: () => 7, is_finished: () => false, legal_actions_json: () => JSON.stringify({seat: 0, revision: 7, actions: []}), activate_skill() {} } });
}
test('u32 choice counts reject malformed fields before host acceptance', async () => {
 const {validateChoiceCounts}=await import('../src/engine/observerMetadata.ts');
 for(const count of [0,255,256,257,0xffffffff]) {
  assert.equal(envelope({id:9,min:0,max:count,options:[]}).choice.max_count,count);
  validateChoiceCounts({min_count:0,max_count:count});
 }
 for(const count of [-1,1.5,Infinity,NaN,0x100000000,'256',null]) {
  assert.throws(()=>envelope({id:9,min:0,max:count,options:[]}),/choice/);
  assert.throws(()=>validateChoiceCounts({min_count:0,max_count:count}),/choice/);
 }
 assert.throws(()=>validateChoiceCounts({min_count:257,max_count:256}),/choice/);
 assert.throws(()=>envelope({id:9,min:257,max:256,options:[]}),/choice/);
});

test('two slots preserve def zero and defaults accept old payloads', () => {
 assert.deepEqual(nativeObserverToSnapshot(view).seats[0].generals, [general, null]);
 const old = structuredClone(view); delete old.players[0].generals;
 assert.equal(nativeObserverToSnapshot(old).seats[0].generals, undefined);
 assert.equal(nativeObserverToSnapshot(old).seats[0].character_def, 0);
});
test('canonical kinds and prompts reach real envelope conversion', () => {
 for (const [native_kind, kind] of [['confirm','Confirm'], ['select_cards','SelectCards'], ['select_players','SelectTargets'], ['respond_card','Custom'], ['pick_tag','Custom']]) {
  const result = envelope({id: 9, min: 1, max: 1, allowCancel: true, native_kind, kind, prompt: 'respond_shan', options: [{card:42}]});
  assert.equal(result.choice.native_kind, native_kind); assert.equal(result.choice.kind, kind); assert.equal(result.choice.prompt, 'respond_shan');
  assert.deepEqual(result.choice.options, [{token:0,label_key:'card:42'}]);
 }
 assert.equal(envelope({id:9, hidden:true}).choice, null);
 assert.equal(envelope({id:9, options:[]}).choice.kind, 'Custom');
});
test('metadata wrong types refuse explicitly at local JSON boundary', () => {
 for (const generals of [null, [], [general], [general,null,null], [{...general,def:-1},null], [{...general,def:'0'},null], [{...general,name:0},null], [{...general,revealed:0},null]]) {
  assert.throws(() => nativeObserverToSnapshot({...view,players:[{...view.players[0],generals}]}), /generals/);
 }
 for (const metadata of [{prompt:3},{native_kind:'extension_new_kind'},{native_kind:3}]) assert.throws(() => envelope({id:9,options:[],...metadata}), /prompt|native_kind/);
});

test('old functional TypeScript reader ignores extra metadata', async () => {
 const {nativeObserverToSnapshot: oldReader} = await import('./approved-b-old-reader.ts');
 const previous=structuredClone(view);delete previous.players[0].generals;
 assert.deepEqual(oldReader(view),oldReader(previous));
});

test('production prompt literals and extension keys have readable presentation without internal keys', async () => {
 const {readFileSync,readdirSync} = await import('node:fs');
 const {join} = await import('node:path');
 const {fileURLToPath} = await import('node:url');
 const {choicePromptLabel} = await import('../src/engine/observerMetadata.ts');
 const source=readFileSync(new URL('../../../crates/engine-native-kernel/src/archive/resolution.rs',import.meta.url),'utf8');
 const prompts=[...source.split('KNOWN_PROMPTS:')[1].split('];')[0].matchAll(/^    "([^"]+)",/gm)].map(match=>match[1]);
 // Literal inventory is a presentation guard, not a reachability/restore proof.
 const visit=directory=>readdirSync(directory,{withFileTypes:true}).flatMap(entry=>entry.isDirectory()?visit(join(directory,entry.name)):entry.name.endsWith('.rs')?[join(directory,entry.name)]:[]);
 const files=['engine-native-kernel','engine-native-content','engine-native'].flatMap(crate=>visit(fileURLToPath(new URL(`../../../crates/${crate}/src`,import.meta.url))));
 for(const file of files)for(const match of readFileSync(file,'utf8').matchAll(/prompt:\s*"([^"]+)"/g))prompts.push(match[1]);
 for(const prompt of new Set(prompts))assert(!choicePromptLabel({prompt,native_kind:'select_cards'}).includes(prompt),prompt);
 for(const key of ['<img src=x onerror=alert(1)>extension','__proto__','constructor','toString']) assert.equal(choicePromptLabel({prompt:key,native_kind:'respond_card'}),'响应出牌');
});

test('r05h linked, r05_effects, and tiqi_adjustment context preserve signed/zero values and reject malformed payloads', async () => {
 const {validateObserverMetadata} = await import('../src/engine/observerMetadata.ts');
 const r05 = [
  { skill: { namespace: 'official', package_id: 'xianding', logical_skill_def_id: 101 }, tiqi_limit: 0, baoshu_combs: -2, dcbihuo_next: null, dcbihuo_current: null },
  { skill: { namespace: 'official', package_id: 'sp2', logical_skill_def_id: 202 }, tiqi_limit: null, baoshu_combs: null, dcbihuo_next: 1, dcbihuo_current: 0 },
 ];
 const snap = nativeObserverToSnapshot({
  ...view,
  phase: 'prepare',
  players: [{ ...view.players[0], linked: true, r05Effects: r05 }],
 });
 assert.equal(snap.phase, 'Prepare');
 assert.equal(snap.seats[0].linked, true);
 assert.deepEqual(snap.seats[0].r05_effects, r05);
 const ctx = { kind: 'tiqi_adjustment', skill: { namespace: 'official', package_id: 'xianding', logical_skill_def_id: 101 }, target: 1, delta: 2 };
 const env = envelope({ id: 12, min: 1, max: 1, allowCancel: true, native_kind: 'pick_tag', prompt: 'rvm_tiqi_sign', context: ctx, options: [{ tag: 1 }, { tag: 2 }] });
 assert.deepEqual(env.choice.context, ctx);
 validateObserverMetadata(snap, env.choice);
 for (const badEffects of [
  null,
  [{}],
  [{ skill: { namespace: '', package_id: 'x', logical_skill_def_id: 1 }, tiqi_limit: 0, baoshu_combs: null, dcbihuo_next: null, dcbihuo_current: null }],
  [{ skill: { namespace: 'official', package_id: 'x', logical_skill_def_id: 1 }, tiqi_limit: null, baoshu_combs: null, dcbihuo_next: null, dcbihuo_current: null }],
 ]) {
  assert.throws(() => nativeObserverToSnapshot({ ...view, players: [{ ...view.players[0], r05Effects: badEffects }] }), /r05_effects/);
 }
 for (const badCtx of [
  { kind: 'other', skill: ctx.skill, target: 1, delta: 2 },
  { kind: 'tiqi_adjustment', skill: { ...ctx.skill, namespace: '' }, target: 1, delta: 2 },
  { kind: 'tiqi_adjustment', skill: ctx.skill, target: -1, delta: 2 },
  { kind: 'tiqi_adjustment', skill: ctx.skill, target: 1, delta: 0 },
 ]) {
  assert.throws(() => envelope({ id: 12, min: 1, max: 1, options: [], context: badCtx }), /context/);
 }
});
