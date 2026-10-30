import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';

const root = resolve(process.argv[2]);
const evidence = resolve(process.argv[3]);
const pkg = resolve(root, 'apps/web-client/src/worker/native-pkg');
const { default: init, NativeEngine } = await import(pathToFileURL(resolve(pkg, 'engine_native_wasm.js')));
await init({ module_or_path: readFileSync(resolve(pkg, 'engine_native_wasm_bg.wasm')) });
let stateComparisons = 0, seatChecks = 0, commands = 0, archiveRestarts = 0;

for (const name of ['XunXun', 'JiJie', 'GaiMing', 'HaoTianTa', 'Liaoyuan']) {
  const fixture = JSON.parse(readFileSync(resolve(evidence, 'machine-traces', `${name}.json`), 'utf8'));
  const directory = name === 'GaiMing' ? 'boss-zhugeliang-upstream-liuli'
    : ['HaoTianTa', 'Liaoyuan'].includes(name) ? 'shahai-swd-upstream-noname' : null;
  const packs = directory ? `[${readFileSync(resolve(root, 'packages/extensions', directory, 'assets/cards.json'), 'utf8')}]` : '[]';
  const engine = new NativeEngine(fixture.frames[0].visible.length, 1n);
  const check = (frame, index) => {
    assert.equal(engine.state_hash_hex(), frame.hash, `${name}/${index}: hash including RNG`);
    assert.equal(engine.revision(), BigInt(frame.revision), `${name}/${index}: revision`);
    for (let seat = 0; seat < frame.visible.length; seat++) {
      const view = JSON.parse(engine.view_json(seat));
      const ids = Object.keys(view.cards).map(Number).sort((a,b) => a-b);
      assert.deepEqual(ids, frame.visible[seat], `${name}/${index}/seat${seat}: authorized cards`);
      assert.equal(view.deckCount, frame.deckCount);
      if (index === 0) assert(ids.every(id => !fixture.held.includes(id)), `${name}: machine-held privacy`);
      const choice = JSON.parse(engine.open_choice_json(seat));
      if (!frame.choice) assert.equal(choice, null);
      else if (seat === frame.choice.observer) assert.deepEqual(choice, frame.choice, `${name}/${index}: exact owner choice`);
      else {
        assert.equal(choice.hidden, true);
        assert.equal(choice.options, undefined);
        assert.equal(choice.id, frame.choice.id);
        assert.equal(choice.observer, frame.choice.observer);
      }
      seatChecks++;
    }
    stateComparisons++;
  };
  try {
    engine.restore_hex_with_packs_json(fixture.archiveHex, packs);
    archiveRestarts++;
    check(fixture.frames[0], 0);
    for (let index = 0; index < fixture.commands.length; index++) {
      const { command, codes } = fixture.commands[index];
      const log = engine.log_len();
      const result = JSON.parse(command.kind === 'answer'
        ? engine.answer(command.actor, BigInt(command.id), new Uint32Array(command.indices))
        : engine.next_phase(command.actor));
      assert(['accepted', 'pending'].includes(result.outcome), `${name}/${index}: ${JSON.stringify(result)}`);
      assert.deepEqual(JSON.parse(engine.public_event_codes_json(log)), codes, `${name}/${index}: events`);
      commands++;
      check(fixture.frames[index + 1], index + 1);
      const saved = engine.save_hex();
      engine.restore_hex_with_packs_json(saved, packs);
      archiveRestarts++;
      check(fixture.frames[index + 1], index + 1);
    }
    console.log(`PASS ${name}: pending archive, exact choices/privacy, continued hashes/events and repeated byte restores`);
  } finally {
    engine.free();
  }
}
assert.equal(commands, 35);
assert.equal(archiveRestarts, 40);
console.log(JSON.stringify({ windows: 5, stateComparisons, seatChecks, commands, archiveRestarts, failed: 0 }));
