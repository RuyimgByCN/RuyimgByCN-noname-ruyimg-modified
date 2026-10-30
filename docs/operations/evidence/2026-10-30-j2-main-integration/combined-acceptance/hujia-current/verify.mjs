import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
const [root, vectorFile, reportFile] = process.argv.slice(2);
const vector = JSON.parse(readFileSync(vectorFile, 'utf8'));
const pkg = resolve(root, 'apps/web-client/src/worker/native-pkg');
const { default: init, NativeEngine } = await import(pathToFileURL(resolve(pkg, 'engine_native_wasm.js')));
await init({ module_or_path: readFileSync(resolve(pkg, 'engine_native_wasm_bg.wasm')) });
const engine = new NativeEngine(vector.players, BigInt(vector.seed));
try {
  engine.restore_hex(vector.archiveHex);
  assert.equal(engine.save_hex(), vector.expectedRestoredHex, 'restore auth generation matches native');
  for (let seat = 0; seat < vector.players; seat++) {
    assert.deepEqual(JSON.parse(engine.open_choice_json(seat)), vector.initialChoices[seat]);
  }
  const initial = JSON.parse(engine.open_choice_json(vector.observer));
  assert.deepEqual(initial.options, []);
  assert.equal(initial.min, 0);
  assert.equal(initial.allowCancel, false);
  assert.equal(engine.log_len(), vector.logStart);
  const replyString = engine.auto_step();
  assert.equal(replyString, vector.expectedReplyString, 'complete reply bytes');
  assert.deepEqual(JSON.parse(replyString), vector.expectedReplyJSON, 'complete reply JSON');
  assert.equal(engine.save_hex(), vector.expectedEmptyAnswerSavedHex, 'complete archive bytes against native manual empty answer');
  assert.equal(engine.state_hash_hex(), vector.expectedHash);
  assert.equal(engine.revision().toString(), vector.expectedRevision);
  for (let seat = 0; seat < vector.players; seat++) {
    assert.deepEqual(JSON.parse(engine.open_choice_json(seat)), vector.expectedChoices[seat]);
  }
  assert.deepEqual(JSON.parse(engine.public_event_codes_json(vector.logStart)), vector.expectedEventCodes);
  const report = { passed: true, runtime: 'actual wasm32 web-target artifact loaded by generated JavaScript in Node', players: vector.players, seed: vector.seed, fixtureSteps: vector.fixtureSteps, prompt: 'hujia_respond', options: [], min: 0, cancel: false, exactReply: true, exactArchiveAgainstNativeManualEmptyAnswer: true, exactRestoredAuthGeneration: true, hash: engine.state_hash_hex(), revision: engine.revision().toString(), choicesCompared: 16, eventCodes: vector.expectedEventCodes };
  writeFileSync(reportFile, JSON.stringify(report, null, 2) + '\n');
  console.log(JSON.stringify(report));
} finally { engine.free(); }
