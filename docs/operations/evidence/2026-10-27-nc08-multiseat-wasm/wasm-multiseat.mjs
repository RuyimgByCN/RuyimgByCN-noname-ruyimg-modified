// Compare independently built WASM with the native test's complete trajectories.
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const [pkgDir, input, output] = process.argv.slice(2);
const { initSync, NativeEngine } = await import(pathToFileURL(join(pkgDir, "engine_native_wasm.js")));
initSync({ module: readFileSync(join(pkgDir, "engine_native_wasm_bg.wasm")) });
const native = JSON.parse(readFileSync(input, "utf8"));
const packs = native.packs;

function start(players, seed) {
  const engine = NativeEngine.new_with_packs_json(players, BigInt(seed), packs);
  const assigned = JSON.parse(engine.assign_general(0, "vm_fixture"));
  assert.equal(assigned.name, "vm_fixture");
  assert.ok(JSON.parse(engine.view_json(0)).players[0].skills.includes("vm_fixture_phase_end_phase_end"));
  return engine;
}

function run(engine, count) {
  const records = [];
  let cursor = Number(engine.log_len());
  for (let i = 0; i < count; i++) {
    const before = JSON.parse(engine.view_json(0));
    const visible0 = JSON.parse(engine.open_choice_json(0));
    let choice = visible0;
    if (visible0?.hidden) {
      assert.equal("options" in visible0, false, "another seat's options must be hidden");
      choice = JSON.parse(engine.open_choice_json(visible0.observer));
      assert.ok(Array.isArray(choice.options), "observer must see its own options");
    }
    const info = choice ? {
      id: choice.id, observer: choice.observer, min: choice.min,
      count: choice.options.length, hidden: !!visible0.hidden,
    } : null;
    const actor = choice ? Number(choice.observer) : Number(before.turn);
    const raw = choice
      ? engine.answer(actor, BigInt(choice.id), Uint32Array.from({ length: choice.min }, (_, n) => n))
      : engine.next_phase(actor);
    const outcome = JSON.parse(raw);
    assert.ok(["accepted", "pending"].includes(outcome.outcome), raw);
    const codes = JSON.parse(engine.public_event_codes_json(cursor));
    cursor = Number(engine.log_len());
    const after = JSON.parse(engine.view_json(0));
    records.push({
      choice: info, actor, before_turn: before.turn, before_phase: before.phase,
      before_hand0: before.players[0].handCount,
      revision: Number(engine.revision()), hash: engine.state_hash_hex(), codes,
      turn: after.turn, phase: after.phase, hand0: after.players[0].handCount,
    });
  }
  return records;
}

const scenarios = [];
for (const reference of native.scenarios) {
  const { players, seed } = reference;
  const direct = run(start(players, seed), reference.direct.length);
  const first = start(players, seed);
  const midpoint = reference.restore.midpoint;
  const before = run(first, midpoint);
  const pending = JSON.parse(first.open_choice_json(0));
  assert.ok(pending?.hidden && pending.observer !== 0, "save at another seat's private choice");
  const saved = first.save_hex();
  const restored = NativeEngine.new_with_packs_json(players, BigInt(seed), packs);
  restored.restore_hex_with_packs_json(saved, packs);
  const after = run(restored, direct.length - midpoint);
  assert.deepEqual(direct, reference.direct, `${players} seats direct`);
  assert.deepEqual(before, reference.restore.before, `${players} seats before restore`);
  assert.deepEqual(after, reference.restore.after, `${players} seats after restore`);
  assert.deepEqual([...before, ...after], direct, `${players} seats same-target restore`);
  assert.ok(direct.some((r) => r.before_turn === 0 && r.before_phase === "discard" &&
    r.choice === null && r.turn === 0 && r.phase === "end" && r.hand0 === r.before_hand0 + 1),
  "VM fixture must draw at end phase");
  scenarios.push({ players, checkpoint: midpoint, direct, restore: { before, after } });
  console.log(JSON.stringify({ players, steps: direct.length, checkpoint: midpoint,
    hidden_choices: direct.filter((r) => r.choice?.hidden).length, final_hash: direct.at(-1).hash }));
}
writeFileSync(output, JSON.stringify({ scenarios }, null, 2) + "\n");
