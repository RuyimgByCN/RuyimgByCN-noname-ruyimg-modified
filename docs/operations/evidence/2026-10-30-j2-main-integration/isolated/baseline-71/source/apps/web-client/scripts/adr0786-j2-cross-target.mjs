// Replay the formal J2 author DSL matrix against an actual wasm32 build.
// Usage: node scripts/adr0786-j2-cross-target.mjs <native-matrix.json> <wasm-pkg-dir> <result.json>
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [input, wasmDir, output] = process.argv.slice(2).map(value => resolve(value));
const vectors = JSON.parse(readFileSync(input, "utf8"));
const module = await import(pathToFileURL(resolve(wasmDir, "engine_native_wasm.js")).href);
await module.default({ module_or_path: readFileSync(resolve(wasmDir, "engine_native_wasm_bg.wasm")) });
const { NativeEngine } = module;
function choice(engine) {
  for (let seat = 0; seat < 4; seat++) {
    const value = JSON.parse(engine.open_choice_json(seat));
    if (value && !value.hidden) return value;
  }
  return null;
}
function apply(engine, command) {
  switch (command.type) {
    case "nextPhase": return JSON.parse(engine.next_phase(command.actor));
    case "activateSkill": return JSON.parse(engine.activate_skill(command.actor, command.skill));
    case "playCard": return JSON.parse(engine.play_card(command.actor, command.card, new Uint32Array(command.targets)));
    case "answer": return JSON.parse(engine.answer(command.actor, BigInt(command.choice_id), new Uint32Array(command.selections)));
    default: throw Error(`Unknown command ${command.type}`);
  }
}
function rejectedProbes(engine) {
  const window = choice(engine);
  if (!window) return;
  const snapshot = engine.save_hex();
  const events = engine.public_event_codes_json(0);
  for (const [actor, id, indices] of [[(window.observer + 1) % 4, window.id, [0]], [window.observer, window.id + 1, [0]], [window.observer, window.id, [999]]]) {
    let rejected = false;
    try { rejected = JSON.parse(engine.answer(actor, BigInt(id), new Uint32Array(indices))).outcome === "rejected"; }
    catch { rejected = true; }
    assert.ok(rejected, "invalid answer must reject");
    assert.equal(engine.save_hex(), snapshot, "invalid answer preserves complete archive and dedup key");
    assert.equal(engine.public_event_codes_json(0), events, "invalid answer cannot append events");
  }
}
let checkpoints = 0;
let retries = 0;
const results = [];
for (const scenario of vectors.scenarios) {
  const engine = new NativeEngine(4, 0n);
  engine.restore_hex_with_packs_json(scenario.initial, scenario.packs_json);
  for (const invalid of scenario.invalid_archives ?? []) {
    const before = engine.save_hex();
    assert.throws(() => engine.restore_hex_with_packs_json(invalid, scenario.packs_json), `${scenario.name}: forged content reference rejects restore`);
    assert.equal(engine.save_hex(), before, `${scenario.name}: failed restore preserves the live game`);
  }
  const hashes = [];
  for (const [index, step] of scenario.steps.entries()) {
    const label = `${scenario.name} step${index} ${step.command.type}`;
    assert.deepEqual(choice(engine), step.before_choice, `${label}: choice before command`);
    const restored = new NativeEngine(4, 0n);
    restored.restore_hex_with_packs_json(engine.save_hex(), scenario.packs_json);
    rejectedProbes(engine);
    rejectedProbes(restored);
    const cursor = Number(engine.log_len());
    const direct = apply(engine, step.command);
    const resumed = apply(restored, step.command);
    assert.equal(direct.outcome, step.outcome, `${label}: native/WASM outcome`);
    assert.equal(resumed.outcome, step.outcome, `${label}: resumed outcome`);
    assert.equal(engine.state_hash_hex(), step.hash, `${label}: native/WASM hash`);
    assert.equal(restored.state_hash_hex(), step.hash, `${label}: resumed hash`);
    assert.deepEqual(choice(engine), step.choice, `${label}: choice after command`);
    assert.deepEqual(choice(restored), step.choice, `${label}: resumed choice`);
    assert.deepEqual(JSON.parse(engine.public_event_codes_json(cursor)), step.codes, `${label}: native/WASM event trajectory`);
    assert.deepEqual(JSON.parse(restored.public_event_codes_json(0)), step.codes, `${label}: restored event trajectory`);
    // Full archives compare all ordered zones, RNG state/counter, VM continuation, fuel,
    // pending effects, event/choice ids, dedup watermark and authorization generation.
    assert.equal(engine.save_hex(), step.archive, `${label}: full native/WASM archive`);
    assert.equal(restored.save_hex(), step.resumed_archive, `${label}: full resumed native/WASM archive`);
    if (step.command.type === "answer") {
      const bytes = engine.save_hex();
      let rejected = false;
      try { rejected = apply(engine, step.command).outcome === "rejected"; }
      catch { rejected = true; }
      assert.ok(rejected, `${label}: stale choice retry rejects`);
      assert.equal(engine.save_hex(), bytes, `${label}: retry has no duplicate effects`);
      retries++;
    }
    checkpoints++;
    hashes.push(step.hash);
    restored.free();
  }
  engine.free();
  results.push({ name: scenario.name, steps: scenario.steps.length, final_hash: hashes.at(-1), hashes });
}
const result = { scenarios: results.length, checkpoints, retries, full_archive_equality: true, native_wasm_events_choices_hashes_match: true, results };
writeFileSync(output, JSON.stringify(result, null, 2) + "\n");
console.log(JSON.stringify({ ...result, results: results.map(({ hashes, ...rest }) => rest) }, null, 2));
