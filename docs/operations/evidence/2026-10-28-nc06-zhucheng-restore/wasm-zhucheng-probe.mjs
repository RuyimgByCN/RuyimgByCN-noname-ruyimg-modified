import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const root = process.argv[2];
assert.ok(root, 'expected isolated export path');
const pkg = join(root, 'apps/web-client/src/worker/native-pkg');
const { default: init, NativeEngine } = await import(pathToFileURL(join(pkg, 'engine_native_wasm.js')).href);
await init({ module_or_path: readFileSync(join(pkg, 'engine_native_wasm_bg.wasm')) });
const ext = join(root, 'packages/extensions/wudian-upstream-noname');
const manifest = JSON.parse(readFileSync(join(ext, 'package.npack.json'), 'utf8'));
const cards = JSON.parse(readFileSync(join(ext, 'assets/cards.json'), 'utf8'));
const packs = JSON.stringify([{ manifest: manifest.manifest, cards }]);
const original = NativeEngine.new_with_packs_json(3, 3501n, packs);
original.assign_general(0, '刘馥');
let choice = null;
for (let i = 0; i < 32 && !choice; i++) {
  choice = JSON.parse(original.open_choice_json(0));
  if (!choice) {
    const actor = JSON.parse(original.view_json(0)).turn;
    const outcome = JSON.parse(original.next_phase(actor));
    assert.notEqual(outcome.outcome, 'rejected', JSON.stringify(outcome));
  }
}
assert.equal(choice?.prompt, 'trigger_confirm');
const confirm = choice.options.indexOf('confirm');
assert.ok(confirm >= 0, JSON.stringify(choice));
const answer = JSON.parse(original.answer(0, BigInt(choice.id), Uint32Array.of(confirm)));
assert.notEqual(answer.outcome, 'rejected', JSON.stringify(answer));
const hex = original.save_hex();
const hash = original.state_hash_hex();
const restored = NativeEngine.new_with_packs_json(3, 1n, packs);
restored.restore_hex_with_packs_json(hex, packs);
assert.equal(restored.state_hash_hex(), hash);
assert.throws(() => restored.restore_hex_with_packs_json(hex, '[]'));
assert.equal(restored.state_hash_hex(), hash);
let replayed = 0;
for (let i = 0; i < 8; i++) {
  if (JSON.parse(original.open_choice_json(0))) break;
  const actor = JSON.parse(original.view_json(0)).turn;
  const a = JSON.parse(original.next_phase(actor));
  const b = JSON.parse(restored.next_phase(actor));
  assert.deepEqual(b, a);
  assert.equal(restored.state_hash_hex(), original.state_hash_hex());
  replayed++;
}
assert.ok(replayed > 0);
console.log(`PASS wasm_zhucheng_save_restore_replay: ${replayed} steps, missing package rejected`);
console.log('RESULT 1 passed / 0 failed');
