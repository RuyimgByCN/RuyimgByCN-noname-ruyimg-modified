import { readFileSync } from "node:fs";

const fixtureRoot = new URL("../../../crates/engine-protocol/tests/fixtures/", import.meta.url);
export const presentationBatch = () => JSON.parse(readFileSync(new URL("i02-presentation.json", fixtureRoot), "utf8"));
const invalidRegularBatches = () => JSON.parse(readFileSync(new URL("i02-presentation-invalid.json", fixtureRoot), "utf8")).map(({name, changes}) => {
  const batch = presentationBatch();
  for (const change of changes) {
    const parts = change.pointer.slice(1).split("/");
    const key = parts.pop();
    const parent = parts.reduce((value, part) => value[part], batch);
    if (change.remove) delete parent[key]; else parent[key] = change.value;
  }
  return { name, batch };
});
export const surrogatePresentationBatches = () => JSON.parse(readFileSync(new URL("i02-presentation-surrogates.json", fixtureRoot), "utf8")).map(({name,pointer,raw_json_value}) => {
  const batch=presentationBatch();const parts=pointer.slice(1).split("/");const key=parts.pop();
  const parent=parts.reduce((value,part)=>value[part],batch);parent[key]=JSON.parse(raw_json_value);
  return {name,batch};
});
export const invalidPresentationBatches = () => [...invalidRegularBatches(),...surrogatePresentationBatches()];
export function presentationView(revision = 2) {
  return { revision, state_hash: Array(32).fill(0), observer: "Spectator", round: 1, current_player: 1,
    phase: "Play", seats: [1, 2].map(player => ({player, seat: player - 1, hp: 4, max_hp: 4, alive: true, hand_count: 0, identity: null, skills: []})),
    known_cards: [], outcome: "InProgress" };
}
export function presentationEnvelope(batch = presentationBatch()) {
  return { result: {outcome: "accepted"}, events: [], presentation: batch, view: presentationView(),
    choice: null, legal_actions: [], hash_hex: "00".repeat(32), revision: 2, finished: false };
}
