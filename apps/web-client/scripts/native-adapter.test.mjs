// NC03 前端适配层回归：2026-10-18 架构审查的三个源码反例（stub 窄壳边界，无浏览器/WASM）。
// 反例来源：docs/operations/evidence/2026-10-18-architecture-audit/adapter-probe.log
//   1. 候选卡牌值 42/99 时产生索引 token 0/1，提交 token 0 却按卡牌值查找 → 找不到候选
//   2. 引擎返回 rejected，适配层仍返回 envelope.result = null
//   3. ChoiceOption Kind(105) 被标成 0
// 运行：pnpm --dir apps/web-client test（node --test，直接转译 src 下的 TS）
import assert from "node:assert/strict";
import { test } from "node:test";
import { applyEnvelope, createViewStore } from "../src/engine/viewStore.ts";

import {
  commandOutcome,
  nativeDispatch,
  nativeEnvelope,
  nativeObserverToSnapshot,
  optionLabel,
  optionValue,
  parseCommandResult,
} from "../src/worker/nativeBackend.ts";

/** 最小 observer 视图：座位 0 有手牌，座位 1 只手牌数。 */
const VIEW_JSON = JSON.stringify({
  seat: 0,
  players: [
    {
      seat: 0,
      alive: true,
      hp: 4,
      maxHp: 4,
      hand: { own: [11, 12] },
      handCount: 2,
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
  phase: "play",
  round: 1,
  outcome: null,
});

const accepted = JSON.stringify({ outcome: "accepted", revision: 7, finished: false });
const rejected = JSON.stringify({ outcome: "rejected", revision: 7, finished: false });

/**
 * 窄壳 stub：只实现 nativeBackend 实际调用的方法面。
 *
 * **NC03 起候选来自内核权威查询**：桩扮演内核，`legal_actions_json` 必须给出
 * 候选；适配层不再自己按手牌拼接，也不再合成应答。默认按 `choice` 推导
 * 「内核会说的候选」（逐项应答 + 可选取消），`legal` 可显式覆盖。
 * @param {{choice?: object|null, reply?: () => string, legal?: unknown[]}} [options]
 */
function stubHandle({ choice = null, reply = () => accepted, legal } = {}) {
  const calls = [];
  const kernelLegal = () => {
    if (legal !== undefined) return legal;
    if (choice === null) return ["EndPhase"];
    const options = choice.options ?? [];
    if (options.length === 0) return [];
    return [
      { ResolveChoice: { choice_id: choice.id, selections: [0], cancelled: false } },
      ...(choice.allowCancel
        ? [{ ResolveChoice: { choice_id: choice.id, selections: [], cancelled: true } }]
        : []),
    ];
  };
  const engine = {
    view_json: () => VIEW_JSON,
    is_finished: () => false,
    revision: () => 7,
    legal_actions_json: (seat) =>
      JSON.stringify({ seat, revision: 7, actions: kernelLegal() }),
    activate_skill: () => reply(),
    open_choice_json: (seat) => {
      calls.push({ call: "open_choice_json", seat });
      return choice === null ? "null" : JSON.stringify(choice);
    },
    answer: (actor, choiceId, selections) => {
      calls.push({
        call: "answer",
        actor: Number(actor),
        choiceId: Number(choiceId),
        selections: [...selections],
      });
      return reply();
    },
    next_phase: () => reply(),
    play_card: (actor, card, targets) => {
      calls.push({ call: "play_card", actor: Number(actor), card: Number(card), targets: [...targets] });
      return reply();
    },
    auto_step: () => reply(),
  };
  const commands = () => calls.filter((c) => c.call !== "open_choice_json");
  return { handle: { engine, seed: 1, players: 2, observer: 0 }, calls, commands };
}

test("反例 1：候选值 42/99 → token 是下标且能回查候选", () => {
  const { handle, commands } = stubHandle({
    choice: { id: 7, options: [{ card: 42 }, { card: 99 }], min: 1, max: 1, allowCancel: false },
  });
  const envelope = nativeEnvelope(handle);
  assert.deepEqual(envelope.choice.options, [
    { token: 0, label_key: "card:42" },
    { token: 1, label_key: "card:99" },
  ]);

  // 合法动作给出的回传值必须是 token，不是候选值 42
  const resolve = envelope.legal_actions.find((a) => a.ResolveChoice);
  assert.deepEqual(resolve.ResolveChoice.selections, [0]);

  // 回传 token 0 必须落到 0 号候选（值 42）
  const out = nativeDispatch(handle, {
    type: "resolveChoice",
    actor: 0,
    choiceId: 7,
    selections: [0],
  });
  assert.deepEqual(commands(), [{ call: "answer", actor: 0, choiceId: 7, selections: [0] }]);
  assert.equal(out.result.outcome, "accepted");
  assert.deepEqual(envelope.choice.options[commands()[0].selections[0]].label_key, "card:42");

  // 候选值本身不是 token：越界必须显式报错，不得静默改成别的候选
  assert.throws(
    () =>
      nativeDispatch(handle, {
        type: "resolveChoice",
        actor: 0,
        choiceId: 7,
        selections: [42],
      }),
    /token 42 不在候选（0\.\.1）/,
  );
});

test("反例 1 补充：裸数字/字符串候选的 token 同样是下标", () => {
  const { handle, commands } = stubHandle({
    choice: { id: 5, options: [42, 99, "confirm", "pass"], min: 1, max: 1, allowCancel: false },
  });
  const envelope = nativeEnvelope(handle);
  assert.deepEqual(
    envelope.choice.options.map((o) => o.token),
    [0, 1, 2, 3],
  );
  assert.deepEqual(
    envelope.choice.options.map((o) => o.label_key),
    ["42", "99", "confirm", "pass"],
  );
  const out = nativeDispatch(handle, {
    type: "resolveChoice",
    actor: 0,
    choiceId: 5,
    selections: [3],
  });
  assert.deepEqual(commands(), [{ call: "answer", actor: 0, choiceId: 5, selections: [3] }]);
  assert.equal(out.result.outcome, "accepted");
});

test("反例 2：引擎 rejected 显式进入 envelope.result", () => {
  const { handle } = stubHandle({
    choice: { id: 7, options: [{ card: 42 }], min: 1, max: 1, allowCancel: false },
    reply: () => rejected,
  });
  const out = nativeDispatch(handle, {
    type: "resolveChoice",
    actor: 0,
    choiceId: 7,
    selections: [0],
  });
  assert.notEqual(out.result, null);
  assert.equal(commandOutcome(out.result), "rejected");
  assert.deepEqual(out.result, { outcome: "rejected", revision: 7, finished: false });
});

test("反例 2 补充：playCard 被拒同样暴露，observer 无命令时保持 null", () => {
  const denied = stubHandle({
    choice: null,
    reply: () => rejected,
  });
  const played = nativeDispatch(denied.handle, {
    type: "playCard",
    actor: 0,
    card: 11,
    targets: [1],
  });
  assert.equal(commandOutcome(played.result), "rejected");

  const idle = stubHandle({ choice: null });
  assert.equal(nativeEnvelope(idle.handle).result, null);
  assert.equal(nativeDispatch(idle.handle, { type: "observer" }).result, null);
});

test("反例 3：ChoiceOption Kind(105) 不被归零", () => {
  assert.equal(optionValue({ kind: 105 }), 105);
  assert.equal(optionLabel({ kind: 105 }), "kind:105");
  assert.equal(optionLabel({ card: 42 }), "card:42");
  assert.equal(optionLabel({ player: 3 }), "player:3");
  assert.equal(optionLabel({ tag: 7 }), "tag:7");

  const { handle, commands } = stubHandle({
    choice: { id: 9, options: [{ kind: 105 }], min: 1, max: 1, allowCancel: true },
  });
  const envelope = nativeEnvelope(handle);
  assert.deepEqual(envelope.choice.options, [{ token: 0, label_key: "kind:105" }]);
  assert.deepEqual(envelope.legal_actions.at(-1), {
    ResolveChoice: { choice_id: 9, selections: [], cancelled: true },
  });

  const out = nativeDispatch(handle, {
    type: "resolveChoice",
    actor: 0,
    choiceId: 9,
    selections: [0],
  });
  assert.deepEqual(commands(), [{ call: "answer", actor: 0, choiceId: 9, selections: [0] }]);
  assert.equal(out.result.outcome, "accepted");
});

test("命令结果解析：非 JSON 或无 outcome 字段不臆造判决", () => {
  assert.equal(parseCommandResult("not json"), null);
  assert.equal(parseCommandResult(undefined), null);
  assert.equal(commandOutcome(null), null);
  assert.equal(commandOutcome(parseCommandResult(JSON.stringify({ done: true }))), null);
  assert.equal(commandOutcome(parseCommandResult(rejected)), "rejected");
  assert.equal(commandOutcome(parseCommandResult(accepted)), "accepted");
});

test("choice_id 不匹配时拒绝应答，不触发引擎调用", () => {
  const { handle, commands } = stubHandle({
    choice: { id: 7, options: [{ card: 42 }], min: 1, max: 1, allowCancel: false },
  });
  assert.throws(
    () =>
      nativeDispatch(handle, {
        type: "resolveChoice",
        actor: 0,
        choiceId: 8,
        selections: [0],
      }),
    /choice 8 不匹配/,
  );
  assert.deepEqual(commands(), []);
});

test("原生公开事件按粗粒度展示，不把防止伤害或完成选择写成已造成伤害或等待选择", () => {
  const { handle } = stubHandle();
  const codes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 99];
  handle.eventCursor = 0;
  handle.engine.state_hash_hex = () => "00";
  handle.engine.log_len = () => codes.length;
  handle.engine.public_event_codes_json = () => JSON.stringify(codes);
  const envelope = nativeEnvelope(handle, { outcome: "accepted" });
  const labels = [
    "对局开始", "摸牌", "使用或响应牌", "伤害结算", "回复体力", "濒死状态变化",
    "玩家阵亡", "阶段或回合推进", "对局结束", "选择状态变化", "事件 99",
  ];
  assert.deepEqual(envelope.events.map((event) => event.PublicEvent.label), labels);
  const store = createViewStore();
  applyEnvelope(store, envelope);
  assert.deepEqual(store.log.map((event) => event.text), labels.toReversed());
});

// ---------------------------------------------------------------------------
// NC03/NC09（GT-013，裁决⑥⑦）：view_json 身份与可见牌面适配。
// ---------------------------------------------------------------------------
test("identity 随座位透传（裁决⑥）：本人/主公非空，隐藏座位为 null，不再恒置 null", () => {
  const snapshot = nativeObserverToSnapshot({
    seat: 0,
    players: [
      {
        seat: 0, alive: true, hp: 5, maxHp: 5,
        hand: { own: [11] }, handCount: 1, equips: [], judge: [],
        turnedOver: false, skills: [], revealedGenerals: [],
        identity: "Lord",
      },
      {
        seat: 1, alive: true, hp: 4, maxHp: 4,
        hand: { hidden: true }, handCount: 3, equips: [40], judge: [],
        turnedOver: false, skills: [], revealedGenerals: [],
        identity: null,
      },
    ],
    deckCount: 90,
    discard: [55],
    table: [66],
    turn: 0,
    phase: "play",
    round: 1,
    outcome: null,
  });
  assert.equal(snapshot.seats[0].identity, "Lord");
  assert.equal(snapshot.seats[1].identity, null);
});

test("可见牌面进入 known_cards（裁决⑦）：本人手牌/全场装备判定/弃牌/处理区带真实 face，不占位", () => {
  const face = (def, suit, rank, kind) => ({ def, suit, rank, kind });
  const snapshot = nativeObserverToSnapshot({
    seat: 0,
    players: [
      {
        seat: 0, alive: true, hp: 5, maxHp: 5,
        hand: { own: [11, 12] }, handCount: 2, equips: [31], judge: [44],
        turnedOver: false, skills: [], revealedGenerals: [],
        identity: "Lord",
      },
      {
        seat: 1, alive: true, hp: 4, maxHp: 4,
        hand: { hidden: true }, handCount: 3, equips: [40], judge: [47],
        turnedOver: false, skills: [], revealedGenerals: [],
        identity: null,
      },
    ],
    cards: {
      "11": face(0, "Spade", 7, "Sha"),
      "12": face(2, "Heart", 12, "Shan"),
      "31": face(10, "Club", 1, "Other"),
      "44": face(30, "Spade", 6, "Other"),
      "40": face(12, "Diamond", 13, "Other"),
      "47": face(31, "Club", 4, "Other"),
      "55": face(20, "Heart", 11, "Other"),
      "66": face(0, "Spade", 8, "Sha"),
    },
    deckCount: 90,
    discard: [55],
    table: [66],
    turn: 0,
    phase: "play",
    round: 1,
    outcome: null,
  });
  const byId = new Map(snapshot.known_cards.map((c) => [c.id, c]));
  // 授权字段非占位（GT-013）：def/suit/rank 有值。
  assert.deepEqual(byId.get(11), {
    id: 11, def: 0, suit: "Spade", rank: 7, owner: 0, zone: "Hand", kind: "Sha",
  });
  assert.equal(byId.get(12).rank, 12);
  // 全场装备/判定（公开）带 owner 归属。
  assert.deepEqual(
    { ...byId.get(40), id: 40 },
    { id: 40, def: 12, suit: "Diamond", rank: 13, owner: 1, zone: "Equip", kind: "Other" },
  );
  assert.equal(byId.get(47).zone, "Judge");
  assert.equal(byId.get(47).owner, 1);
  // 弃牌/处理区无 owner。
  assert.equal(byId.get(55).zone, "DiscardPile");
  assert.equal(byId.get(55).owner, null);
  assert.equal(byId.get(66).zone, "Processing");
  // 他人手牌 id（本就不在 view_json 的 own 列表）不进 known_cards。
  assert.equal(snapshot.known_cards.length, 8);
  assert(!snapshot.known_cards.some((c) => c.zone === "Hand" && c.owner === 1));
});

test("visible general definition reaches the existing seat view", () => {
  const view = JSON.parse(VIEW_JSON);
  view.players[0].characterDef = 17;
  view.players[1].characterDef = 0;
  const snapshot = nativeObserverToSnapshot(view);
  assert.equal(snapshot.seats[0].character_def, 17);
  assert.equal(snapshot.seats[1].character_def, 0);
});

test("Skill options keep stable names and use index tokens without truncation", () => {
  assert.equal(optionValue({ skill: "c331_heal" }), "c331_heal");
  assert.equal(optionLabel({ skill: "c331_heal" }), "skill:c331_heal");
});
