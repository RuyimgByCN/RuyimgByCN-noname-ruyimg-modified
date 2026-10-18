// NC03 前端接线：修复前证据探针（针对 3ee01345 的 nativeBackend.ts）。
//
// 目的：用桩窄壳证明前端合法动作**不来自内核权威查询**：
//   ① 出牌候选按 `hand.slice(0, 8)` 截断（12 张手牌只给 8 个 PlayCard）；
//   ② 开放选择只合成 1 个 `ResolveChoice`（取 options[0]），内核会有逐项候选；
//   ③ 适配层从不调用 `legal_actions_json`（桩故意不提供该方法也不报错）。
//
// **这是修复前的历史证据**：接线后适配层要求桩提供权威查询导出，用本探针的
// 桩重跑会显式报错（预期）。接线后的验收见 `scripts/native-legal-actions.test.mjs`
// 与 `pnpm --dir apps/web-client smoke:native`（`smoke-native.log`）。
//
// 用法（工作目录 apps/web-client）：
//   node --import ./scripts/ts-register.mjs \
//     ../../docs/operations/evidence/2026-10-18-nc03-frontend-authoritative/probe-before-authoritative.mjs
import { nativeEnvelope } from "../../../../apps/web-client/src/worker/nativeBackend.ts";

const HAND = Array.from({ length: 12 }, (_, i) => 100 + i);

function viewJson(phase, hand) {
  return JSON.stringify({
    seat: 0,
    players: [
      {
        seat: 0,
        alive: true,
        hp: 4,
        maxHp: 4,
        hand: { own: hand },
        handCount: hand.length,
        equips: [],
        judge: [],
        turnedOver: false,
        skills: [],
        revealedGenerals: [],
      },
      {
        seat: 1,
        alive: true,
        hp: 4,
        maxHp: 4,
        hand: { hidden: true },
        handCount: 4,
        equips: [],
        judge: [],
        turnedOver: false,
        skills: [],
        revealedGenerals: [],
      },
    ],
    deckCount: 100,
    discard: [],
    table: [],
    turn: 0,
    phase,
    round: 1,
    outcome: null,
  });
}

function stub(phase, hand, choice) {
  const calls = [];
  return {
    calls,
    handle: {
      seed: 1,
      players: 2,
      observer: 0,
      engine: {
        view_json: () => viewJson(phase, hand),
        is_finished: () => false,
        revision: () => 7,
        open_choice_json: () => {
          calls.push("open_choice_json");
          return choice === null ? "null" : JSON.stringify(choice);
        },
        // 修复前适配层不调用权威查询；桩不提供该方法，若被调用会直接抛错。
        answer: () => JSON.stringify({ outcome: "accepted", revision: 7, finished: false }),
        next_phase: () => JSON.stringify({ outcome: "accepted", revision: 7, finished: false }),
        play_card: () => JSON.stringify({ outcome: "accepted", revision: 7, finished: false }),
        auto_step: () => JSON.stringify({ outcome: "accepted", revision: 7, finished: false }),
      },
    },
  };
}

let failed = 0;
const check = (name, ok, detail) => {
  if (!ok) failed++;
  console.log(`${ok ? "PASS" : "FAIL"} ${name}${detail ? ` | ${detail}` : ""}`);
};

// ① 出牌候选截断
const play = stub("play", HAND, null);
const playEnv = nativeEnvelope(play.handle);
const played = playEnv.legal_actions.filter((a) => a.PlayCard).map((a) => a.PlayCard.card);
check(
  "①12 张手牌 → PlayCard 候选被 slice(0,8) 截断为 8",
  played.length === 8 && played.at(-1) === 107 && !played.includes(108),
  `hand=${HAND.length} play=${played.length} cards=${played.join(",")}`,
);
check(
  "①适配层未调用内核权威查询",
  !play.calls.includes("legal_actions_json") && typeof play.handle.engine.legal_actions_json !== "function",
  `calls=${JSON.stringify(play.calls)}`,
);

// ② 开放选择合成单一应答
const choice = {
  id: 7,
  options: [{ card: 42 }, { card: 99 }, "pass"],
  min: 1,
  max: 1,
  allowCancel: true,
};
const withChoice = stub("play", HAND, choice);
const choiceEnv = nativeEnvelope(withChoice.handle);
const synthesized = choiceEnv.legal_actions.filter((a) => a.ResolveChoice);
check(
  "②3 个候选只合成 1 个 ResolveChoice（内核有 3 个逐项应答候选）",
  synthesized.length === 2 && // 1 个取 options[0] + 1 个 cancelled
    synthesized[0].ResolveChoice.selections.length === 1 &&
    synthesized[0].ResolveChoice.selections[0] === 0 &&
    synthesized[1].ResolveChoice.cancelled === true,
  `resolve=${JSON.stringify(synthesized)}`,
);

// ③ 无主动技候选（内核会枚举已授予且启用的主动技）
const skillEnv = nativeEnvelope(stub("play", HAND, null).handle);
check(
  "③适配层不产出 ActivateSkill 候选（内核查询才会枚举）",
  skillEnv.legal_actions.every((a) => typeof a === "string" || !("ActivateSkill" in a)),
  `kinds=${JSON.stringify(skillEnv.legal_actions.map((a) => (typeof a === "string" ? a : Object.keys(a)[0])))}`,
);

console.log(JSON.stringify({ total: 4, failed }));
process.exit(failed ? 1 : 0);
