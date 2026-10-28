import assert from "node:assert/strict";
import { test } from "node:test";
import {
  chooseGeneral,
  nativeCreate,
  nativeCreateIdentity,
  nativeCreateWithPacks,
  nativeDispatch,
} from "../src/worker/nativeBackend.ts";

const unreachable = () => { throw new Error("unexpected WASM call"); };
const handle = {
  engine: {
    next_phase: unreachable,
    play_card: unreachable,
    activate_skill: unreachable,
    open_choice_json: unreachable,
    general_candidates_json: unreachable,
    assign_general: unreachable,
  },
  seed: 1,
  players: 4,
  observer: 0,
  eventCursor: 0,
};

test("座位和人数在进入 WASM 及选将前拒绝整数截断", () => {
  for (const invalid of [-1, 0.5, 256, 2 ** 32, NaN, Infinity]) {
    for (const create of [
      (players, observer) => nativeCreate(players, 1, observer),
      (players, observer) => nativeCreateIdentity(players, 1, observer),
      (players, observer) => nativeCreateWithPacks(players, 1, observer, "[]"),
    ]) {
      assert.throws(() => create(invalid, 0), /native: players 非法整数/);
      assert.throws(() => create(4, invalid), /native: observer 非法整数/);
    }
    assert.throws(() => chooseGeneral(handle, invalid, "甲"), /native: seat 非法整数/);
    for (const request of [
      { type: "endPhase", actor: invalid },
      { type: "playCard", actor: invalid, card: 0, targets: [] },
      { type: "activateSkill", actor: invalid, skill: 0 },
      { type: "resolveChoice", actor: invalid, choiceId: 1, selections: [] },
    ]) {
      assert.throws(() => nativeDispatch(handle, request), /native: actor 非法整数/);
    }
    assert.throws(
      () => nativeDispatch(handle, { type: "playCard", actor: 0, card: 0, targets: [invalid] }),
      /native: target 非法整数/,
    );
  }
});

test("牌和技能 ID 在进入 WASM 前拒绝整数截断", () => {
  for (const invalid of [-1, 0.5, 2 ** 32, NaN, Infinity]) {
    assert.throws(
      () => nativeDispatch(handle, { type: "playCard", actor: 0, card: invalid, targets: [] }),
      /native: card 非法整数/,
    );
  }
  for (const invalid of [-1, 0.5, 2 ** 16, 2 ** 32, NaN, Infinity]) {
    assert.throws(
      () => nativeDispatch(handle, { type: "activateSkill", actor: 0, skill: invalid }),
      /native: skill 非法整数/,
    );
  }
});
