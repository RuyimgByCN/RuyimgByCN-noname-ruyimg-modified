// NC03 前端接线回归：合法动作来自内核权威查询（`legal_actions_json`）。
//
// 修复前（源码依据 `docs/operations/evidence/2026-10-18-nc03-frontend-authoritative/
// repro-before.log`）：适配层按 `hand.slice(0, 8)` 拼接 `PlayCard`、只合成一个
// `ResolveChoice`（永远取 options[0]），从不问内核。本文件用桩窄壳验收接线：
//   1. 候选逐项等于内核返回（不按手牌拼接、不按阶段补 EndPhase）；
//   2. 内核不给候选就不出现（含观察者以外座位、取消候选）；
//   3. 查询按 (handle, revision, seat) 记忆化，读档后失效；
//   4. 旧 wasm 产物缺导出／畸形返回 → 显式报错，不静默回落拼接；
//   5. 内核动作逐项可提交（对应命令面存在），伪造动作的拒绝原因对界面可见。
// 运行：pnpm --dir apps/web-client test
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  commandOutcome,
  nativeDispatch,
  nativeEnvelope,
  parseLegalActionsJson,
} from "../src/worker/nativeBackend.ts";

const ACCEPTED = JSON.stringify({
  outcome: "accepted",
  revision: 7,
  finished: false,
  reason: null,
});

/** observer 视图：座位 0 有 12 张手牌（> 旧实现的 8 张截断），座位 1 只有数量。 */
function viewJson({ hand = Array.from({ length: 12 }, (_, i) => 100 + i), phase = "play" } = {}) {
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

/**
 * 窄壳桩：`legal` 是内核权威查询的返回值（对象 → JSON 字符串，或原始字符串）。
 * 记录每次调用，便于断言「问了内核几次、问了什么」。
 */
function stubHandle({ legal = [], choice = null, reply = () => ACCEPTED, view = {} } = {}) {
  const calls = [];
  const state = { revision: 7 };
  const engine = {
    revision: () => BigInt(state.revision),
    view_json: () => viewJson(view),
    is_finished: () => false,
    open_choice_json: () => {
      calls.push({ call: "open_choice_json" });
      return choice === null ? "null" : JSON.stringify(choice);
    },
    legal_actions_json: (seat) => {
      calls.push({ call: "legal_actions_json", seat, revision: state.revision });
      const body = typeof legal === "function" ? legal(state.revision, seat) : legal;
      return typeof body === "string"
        ? body
        : JSON.stringify({ seat, revision: state.revision, actions: body });
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
    next_phase: (actor) => {
      calls.push({ call: "next_phase", actor: Number(actor) });
      return reply();
    },
    play_card: (actor, card, targets) => {
      calls.push({
        call: "play_card",
        actor: Number(actor),
        card: Number(card),
        targets: [...targets],
      });
      return reply();
    },
    activate_skill: (actor, skill) => {
      calls.push({ call: "activate_skill", actor: Number(actor), skill: Number(skill) });
      return reply();
    },
    auto_step: () => reply(),
    restore_hex: (hex) => calls.push({ call: "restore_hex", hex }),
  };
  const commands = () => calls.filter((c) => c.call !== "legal_actions_json");
  const queries = () => calls.filter((c) => c.call === "legal_actions_json");
  return { handle: { engine, seed: 1, players: 2, observer: 0 }, calls, commands, queries, state };
}

test("候选逐项等于内核返回：不按 hand.slice(0,8) 拼接", () => {
  // 内核说：第 3、9、12 张手牌可用（下标 2/8/11，其中两个超出 slice(0,8)）。
  const kernel = [
    "EndPhase",
    { PlayCard: { card: 102, targets: [] } },
    { PlayCard: { card: 108, targets: [1] } },
    { PlayCard: { card: 111, targets: [] } },
    { ActivateSkill: { skill: 5, targets: [], card_targets: [] } },
  ];
  const { handle, queries } = stubHandle({ legal: kernel });
  const envelope = nativeEnvelope(handle);

  assert.deepEqual(envelope.legal_actions, kernel, "候选必须逐项来自内核");
  assert.deepEqual(queries(), [{ call: "legal_actions_json", seat: 0, revision: 7 }]);
  // 视图里有 12 张手牌，但内核只给 3 张出牌候选 → 不得补齐（旧实现给 8 张）
  const played = envelope.legal_actions.filter((a) => a.PlayCard).map((a) => a.PlayCard.card);
  assert.deepEqual(played, [102, 108, 111]);
  assert.ok(!played.includes(109), "内核没给的卡不得凭空出现");
  assert.ok(played.includes(111), "第 12 张手牌（下标 11）也必须能成为候选");
});

test("无合成：内核不给 EndPhase/应答候选就不出现", () => {
  // 阶段是 play（旧实现必然补 EndPhase），内核只给一张牌。
  const only = [{ PlayCard: { card: 100, targets: [] } }];
  const play = stubHandle({ legal: only, view: { phase: "play" } });
  assert.deepEqual(nativeEnvelope(play.handle).legal_actions, only);

  // 开放选择存在，但内核只给「取消」候选 → 不得凭空补 options[0] 应答。
  const choice = {
    id: 7,
    options: [{ card: 42 }, { card: 99 }, "pass"],
    min: 1,
    max: 1,
    allowCancel: true,
  };
  const cancelOnly = stubHandle({
    legal: [{ ResolveChoice: { choice_id: 7, selections: [], cancelled: true } }],
    choice,
  });
  const envelope = nativeEnvelope(cancelOnly.handle);
  assert.deepEqual(envelope.choice.options.map((o) => o.token), [0, 1, 2]);
  assert.deepEqual(envelope.legal_actions, [
    { ResolveChoice: { choice_id: 7, selections: [], cancelled: true } },
  ]);

  // 内核给出「第二项」应答时，selections 原样保留（旧实现恒为 [0]）。
  const second = stubHandle({
    legal: [{ ResolveChoice: { choice_id: 7, selections: [1], cancelled: false } }],
    choice,
  });
  const secondEnvelope = nativeEnvelope(second.handle);
  assert.deepEqual(secondEnvelope.legal_actions, [
    { ResolveChoice: { choice_id: 7, selections: [1], cancelled: false } },
  ]);

  // 观察者以外的座位：内核给空 → 界面空（旧实现仍会合成应答）。
  const other = stubHandle({ legal: [], choice });
  assert.deepEqual(nativeEnvelope(other.handle).legal_actions, []);
});

test("查询按 (handle, revision) 记忆化，读档后失效", () => {
  const { handle, queries, state } = stubHandle({
    legal: (revision) => [{ PlayCard: { card: 100 + revision, targets: [] } }],
  });
  assert.equal(nativeEnvelope(handle).legal_actions[0].PlayCard.card, 107);
  assert.equal(nativeEnvelope(handle).legal_actions[0].PlayCard.card, 107);
  assert.equal(queries().length, 1, "同一修订号只查一次内核（1.6 ms/次）");

  state.revision = 8; // 命令被接受后修订号前进
  assert.equal(nativeEnvelope(handle).legal_actions[0].PlayCard.card, 108);
  assert.equal(queries().length, 2, "修订号前进必须重查");

  // 读档：引擎被换成另一份状态，即使修订号相同也必须重查。
  nativeDispatch(handle, { type: "restore", hex: "00" });
  assert.equal(queries().length, 3, "restore 必须使候选缓存失效");
});

test("旧 wasm 产物缺导出／畸形返回 → 显式报错，不回落拼接", () => {
  const stale = stubHandle({ legal: [] });
  delete stale.handle.engine.legal_actions_json;
  delete stale.handle.engine.activate_skill;
  assert.throws(
    () => nativeEnvelope(stale.handle),
    /缺少 legal_actions_json\/activate_skill[\s\S]*build:native/,
    "缺导出必须给出重建命令",
  );

  assert.throws(() => parseLegalActionsJson("null"), /结构不符/);
  assert.throws(() => parseLegalActionsJson("<html>"), /非 JSON/);
  assert.throws(
    () => parseLegalActionsJson(JSON.stringify({ seat: 0, revision: 1, actions: [{ Nope: {} }] })),
    /第 0 项结构不符/,
  );
  assert.throws(
    () => parseLegalActionsJson(JSON.stringify({ seat: 0, revision: 1, actions: ["EndPhase", 7] })),
    /第 1 项结构不符/,
  );
  const ok = parseLegalActionsJson(
    JSON.stringify({
      seat: 0,
      revision: 3,
      actions: [
        "EndPhase",
        { ResolveChoice: { choice_id: 1, selections: [0], cancelled: false } },
        {
          UseSkillCard: { skill: 2, card_kind: 9, targets: [1], equip_slot: 0 },
        },
      ],
    }),
  );
  assert.equal(ok.revision, 3);
  assert.equal(ok.actions.length, 3);
});

test("内核候选逐项可提交：命令面覆盖每种候选，伪造动作的拒绝原因可见", () => {
  const kernel = [
    { PlayCard: { card: 104, targets: [1] } },
    { ActivateSkill: { skill: 5, targets: [], card_targets: [] } },
    { ResolveChoice: { choice_id: 7, selections: [1], cancelled: false } },
    "EndPhase",
  ];
  const { handle, commands } = stubHandle({
    legal: kernel,
    choice: { id: 7, options: [{ card: 42 }, { card: 99 }], min: 1, max: 1, allowCancel: false },
  });
  const envelope = nativeEnvelope(handle);
  assert.deepEqual(envelope.legal_actions, kernel);

  // 逐个候选按界面协议提交 → 桩引擎收到对应调用且返回 accepted。
  const outcomes = [];
  for (const action of envelope.legal_actions) {
    if (action === "EndPhase") {
      outcomes.push(commandOutcome(nativeDispatch(handle, { type: "endPhase" }).result));
    } else if (action.PlayCard) {
      outcomes.push(
        commandOutcome(
          nativeDispatch(handle, {
            type: "playCard",
            actor: 0,
            card: action.PlayCard.card,
            targets: action.PlayCard.targets,
          }).result,
        ),
      );
    } else if (action.ActivateSkill) {
      outcomes.push(
        commandOutcome(
          nativeDispatch(handle, {
            type: "activateSkill",
            actor: 0,
            skill: action.ActivateSkill.skill,
          }).result,
        ),
      );
    } else {
      outcomes.push(
        commandOutcome(
          nativeDispatch(handle, {
            type: "resolveChoice",
            actor: 0,
            choiceId: action.ResolveChoice.choice_id,
            selections: action.ResolveChoice.selections,
          }).result,
        ),
      );
    }
  }
  assert.deepEqual(outcomes, ["accepted", "accepted", "accepted", "accepted"]);
  assert.deepEqual(
    commands().filter((c) => c.call !== "open_choice_json"),
    [
      { call: "play_card", actor: 0, card: 104, targets: [1] },
      { call: "activate_skill", actor: 0, skill: 5 },
      { call: "answer", actor: 0, choiceId: 7, selections: [1] },
      { call: "next_phase", actor: 0 },
    ],
    "每个候选都必须有命令面（含主动技）",
  );

  // 主动技不接受预选目标：内核技能目标走开放选择应答，不能悄悄丢弃。
  assert.throws(
    () =>
      nativeDispatch(handle, {
        type: "activateSkill",
        actor: 0,
        skill: 5,
        targets: [1],
      }),
    /不接受预选目标/,
  );

  // 伪造卡 id／座位被内核拒绝，拒绝原因进入 envelope.result（界面上可见）。
  const denied = stubHandle({
    legal: [],
    reply: () =>
      JSON.stringify({
        outcome: "rejected",
        revision: 7,
        finished: false,
        reason: "CardNotInHand",
      }),
  });
  const forged = nativeDispatch(denied.handle, {
    type: "playCard",
    actor: 0,
    card: 9999,
    targets: [],
  });
  assert.equal(commandOutcome(forged.result), "rejected");
  assert.deepEqual(forged.result, {
    outcome: "rejected",
    revision: 7,
    finished: false,
    reason: "CardNotInHand",
  });
  // 越权座位同样拒绝且原因可见。
  const wrongSeat = stubHandle({
    legal: [],
    reply: () =>
      JSON.stringify({
        outcome: "rejected",
        revision: 7,
        finished: false,
        reason: "NotYourTurn",
      }),
  });
  const denied2 = nativeDispatch(wrongSeat.handle, { type: "endPhase", actor: 1 });
  assert.equal(denied2.result.reason, "NotYourTurn");
});
