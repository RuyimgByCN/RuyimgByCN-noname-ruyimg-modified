// Compare emitted server snapshots with the actual generated WASM32 artifact on the same archives.
// node --import ./scripts/ts-register.mjs scripts/native-server-projection-parity.mjs /tmp/projection.json
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import init, { NativeEngine } from "../src/worker/native-pkg/engine_native_wasm.js";
import { nativeObserverToSnapshot } from "../src/worker/nativeBackend.ts";

assert(process.argv[2], "provide JSON emitted by game-server's emit_projection_fixture example");
const fixtures = JSON.parse(readFileSync(process.argv[2], "utf8"));
await init({ module_or_path: readFileSync(new URL("../src/worker/native-pkg/engine_native_wasm_bg.wasm", import.meta.url)) });
const kinds = new Set();
const zones = new Set();
let comparisons = 0;
let privateChecks = 0;

const cards = (snapshot, server) => snapshot.known_cards.map((card) => ({
  ...card,
  owner: server && card.owner !== null ? card.owner - 1 : card.owner,
})).sort((a, b) => a.id - b.id);
const seats = (snapshot, server) => snapshot.seats.map((seat) => ({
  player: server ? seat.player - 1 : seat.player,
  seat: seat.seat,
  character_def: seat.character_def,
  identity: seat.identity,
  hp: seat.hp,
  max_hp: seat.max_hp,
  alive: seat.alive,
  turned_over: seat.turned_over,
  hand_count: seat.hand_count,
}));

for (const fixture of fixtures) {
  const engine = NativeEngine.new_identity(fixture.seats.length, 1515n);
  try {
    for (let restore = 0; restore < 2; restore++) {
      engine.restore_hex(fixture.archiveHex);
      for (let seat = 0; seat < fixture.seats.length; seat++) {
        const context = `${fixture.name}/restore${restore}/seat${seat}`;
        const raw = JSON.parse(engine.view_json(seat));
        const actual = nativeObserverToSnapshot(raw);
        const server = fixture.seats[seat];
        assert.equal(server.observer.Player - 1, seat, context);
        assert.deepEqual(cards(actual, false), cards(server, true), `${context}: all faces/owners/zones`);
        assert.deepEqual(seats(actual, false), seats(server, true), `${context}: identity/general/seat fields`);
        assert.deepEqual(raw.players[seat].hand.own, fixture.hands[seat], `${context}: exact own hand`);
        const hidden = new Set([
          ...fixture.deck,
          ...fixture.hands.flatMap((hand, owner) => owner === seat ? [] : hand),
        ]);
        for (const snapshot of [actual, server]) {
          assert(snapshot.known_cards.every((card) => !hidden.has(card.id)), `${context}: hidden hand/deck ID leak`);
        }
        for (const player of raw.players.filter((player) => player.seat !== seat)) {
          assert.equal(player.hand.own, undefined, `${context}: foreign raw hand`);
          assert.equal(player.hand.hidden, true, `${context}: foreign raw hand must be hidden`);
        }
        assert.equal(raw.deckCount, fixture.deck.length, `${context}: deck count`);
        assert.equal(raw.deck, undefined, `${context}: raw deck contents`);
        if (hidden.size) privateChecks++;
        for (const card of actual.known_cards) {
          kinds.add(card.kind);
          zones.add(card.zone);
        }
        comparisons++;
      }
    }
    const hidden = new Set([...fixture.deck, ...fixture.hands.flat()]);
    assert.equal(fixture.spectator.observer, "Spectator");
    assert(fixture.spectator.known_cards.every((card) => !hidden.has(card.id)
      && card.zone !== "Hand" && card.zone !== "DrawPile"), `${fixture.name}: supplied-seat spectator leak`);
    console.log(`PASS ${fixture.name}: every seat, two archive restores, server/generated-WASM32 parity`);
  } finally {
    engine.free();
  }
}
assert.equal(fixtures.length, 5);
assert.equal(comparisons, 40);
assert.equal(privateChecks, 32);
assert.deepEqual([...kinds].sort(), ["Sha", "Tao", "Shan", "Jiu", "ShaFire", "ShaThunder", "Other"].sort());
assert.deepEqual([...zones].sort(), ["Hand", "Equip", "Judge", "DiscardPile", "Processing"].sort());
console.log(JSON.stringify({ fixtures: fixtures.length, comparisons, privateChecks, kinds: [...kinds].sort(), zones: [...zones].sort(), failed: 0 }));
