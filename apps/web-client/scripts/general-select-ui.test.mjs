// NC05 选将 UI 面板回归：候选 → 渲染行、点击 → chooseGeneral → 刷新、失败 → 错误。
//
// 测试边界：`BattleView.vue` 是 Vue SFC，node:test 不能直接装载；本文件覆盖它
// 调用的纯函数层 `src/engine/generalSelectModel.ts`（候选校验收窄、点击动作、
// 刷新投影、错误文本）。Vue 模板绑定/CSS/真实浏览器渲染不在本文件内，证据
// README 会写明这条边界；组件接线用 grep 与 typecheck 证明。
//
// 运行：pnpm --dir apps/web-client test
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  applyCandidatePayload,
  candidateRowsFromPayload,
  emptySeatProjection,
  failCandidateLoad,
  initialGeneralSelectState,
  markCandidatesLoading,
  seatProjectionFromView,
  setGeneralSelectSeat,
  submitGeneralSelection,
  withSeatProjection,
} from "../src/engine/generalSelectModel.ts";

const CANDIDATES = {
  count: 2,
  candidates: [
    { def: 0, name: "甲", skills: ["鬼道", "天妒"] },
    { def: 1, name: "乙", skills: [] },
  ],
};

function skillView(name, id) {
  return {
    id,
    def: id,
    name,
    enabled: true,
    use_count: 0,
    active: false,
    view_as: false,
    locked: false,
  };
}

/** 刷新信封只用到 view.seats；其余字段与 Worker 回包同形占位。 */
function envelopeFor(seat, skills, { def = 0, hash = "ab".repeat(32) } = {}) {
  return {
    result: null,
    events: [],
    choice: null,
    view: {
      revision: 7,
      state_hash: hash,
      observer: seat,
      round: 1,
      current_player: seat,
      phase: "Play",
      seats: [
        {
          player: seat,
          character_def: def,
          seat,
          hp: 4,
          max_hp: 4,
          alive: true,
          turned_over: false,
          hand_count: 2,
          identity: null,
          skills: skills.map((name, index) => skillView(name, index)),
        },
      ],
      known_cards: [],
      outcome: null,
    },
    legal_actions: [],
    hash_hex: hash,
    revision: 7,
    finished: false,
  };
}

function readyState(seat = 0) {
  return applyCandidatePayload(initialGeneralSelectState(seat), CANDIDATES);
}

test("候选回包 → 渲染行：名称、技能、稳定顺序（且不改写输入）", () => {
  const rows = candidateRowsFromPayload(CANDIDATES);
  assert.deepEqual(
    rows.map((row) => ({ def: row.def, name: row.name, skills: row.skills })),
    [
      { def: 0, name: "甲", skills: ["鬼道", "天妒"] },
      { def: 1, name: "乙", skills: [] },
    ],
  );
  assert.equal(rows[0].skillLabel, "鬼道 · 天妒");
  assert.equal(rows[1].skillLabel, "无技能");

  const state = readyState(2);
  assert.equal(state.seat, 2);
  assert.equal(state.loadStatus, "ready");
  assert.equal(state.error, null);
  assert.deepEqual(state.candidates, rows);
  assert.deepEqual(CANDIDATES.candidates[0].skills, ["鬼道", "天妒"]);
});

test("空候选是显式空状态，不是错误也不是静默白屏", () => {
  const state = applyCandidatePayload(initialGeneralSelectState(0), {
    count: 0,
    candidates: [],
  });
  assert.equal(state.loadStatus, "empty");
  assert.deepEqual(state.candidates, []);
  assert.equal(state.error, null);
});

test("畸形候选：逐项报错并清空列表，不把坏数据过滤成更小候选集", () => {
  const cases = [
    ["null", null],
    ["数组", []],
    ["缺 candidates", { count: 0 }],
    ["count 类型错", { count: "1", candidates: [] }],
    ["count 负数", { count: -1, candidates: [] }],
    ["count 与候选数不一致", { count: 2, candidates: [CANDIDATES.candidates[0]] }],
    ["项为 null", { count: 1, candidates: [null] }],
    ["def 负数", { count: 1, candidates: [{ def: -1, name: "甲", skills: [] }] }],
    ["def 非整数", { count: 1, candidates: [{ def: 1.5, name: "甲", skills: [] }] }],
    ["name 空串", { count: 1, candidates: [{ def: 0, name: "  ", skills: [] }] }],
    ["skills 非字符串数组", { count: 1, candidates: [{ def: 0, name: "甲", skills: [1] }] }],
    [
      "def 重复",
      {
        count: 2,
        candidates: [
          { def: 0, name: "甲", skills: [] },
          { def: 0, name: "乙", skills: [] },
        ],
      },
    ],
  ];

  for (const [label, payload] of cases) {
    const state = applyCandidatePayload(readyState(), payload);
    assert.equal(state.loadStatus, "error", `${label}: loadStatus`);
    assert.deepEqual(state.candidates, [], `${label}: 不应渲染任何候选`);
    assert.match(
      state.error ?? "",
      /选将候选不可用/,
      `${label}: 必须显示错误文本`,
    );
  }

  assert.throws(
    () => candidateRowsFromPayload(cases[0][1]),
    /选将候选结构不符/,
    "直接解析也必须抛错，而不是返回空数组",
  );
});

test("候选拉取失败保留原始错误文本（含旧产物 rebuild 提示）", () => {
  const reason =
    "native: wasm 产物缺少 general_candidates_json/assign_general（NC05 选将导出）——执行 pnpm --dir apps/web-client build:native 重建 native-pkg";
  const state = failCandidateLoad(readyState(), reason);
  assert.equal(state.loadStatus, "error");
  assert.deepEqual(state.candidates, []);
  assert.match(state.error ?? "", /选将候选加载失败/);
  assert.match(state.error ?? "", /build:native/);
  assert.match(state.error ?? "", /general_candidates_json/);
});

test("重新加载中保留旧候选但清掉旧错误", () => {
  const failed = failCandidateLoad(readyState(), "boom");
  assert.deepEqual(failed.candidates, [], "加载失败清空旧候选，避免可选到过期列表");
  const loading = markCandidatesLoading(readyState());
  assert.equal(loading.loadStatus, "loading");
  assert.equal(loading.error, null);
  assert.equal(loading.candidates.length, 2);
});

test("座位投影：只取目标座位，提取原生技能名与 id", () => {
  const view = envelopeFor(1, ["奸雄", "护驾"], { def: 9 }).view;
  view.seats.unshift({
    player: 0,
    character_def: 0,
    seat: 0,
    hp: 4,
    max_hp: 4,
    alive: true,
    turned_over: false,
    hand_count: 2,
    identity: null,
    skills: [skillView("别人的技能", 42)],
  });
  const projection = seatProjectionFromView(view, 1);
  assert.equal(projection.characterDef, 9);
  assert.deepEqual(projection.skillNames, ["奸雄", "护驾"]);
  assert.deepEqual(projection.skillIds, [0, 1]);
  assert.deepEqual(seatProjectionFromView(view, 8), emptySeatProjection());
  assert.deepEqual(seatProjectionFromView(null, 1), emptySeatProjection());
});

test("绑定新对局座位：清掉最近装配与投影，但复用候选目录（同座位也清）", () => {
  const state = withSeatProjection(
    {
      ...readyState(0),
      assignment: { seat: 0, name: "甲", revision: 7, state_hash: "cd".repeat(32) },
      lastStateHash: "cd".repeat(32),
    },
    seatProjectionFromView(envelopeFor(0, ["鬼道"]).view, 0),
  );
  const moved = setGeneralSelectSeat(state, 1);
  assert.equal(moved.seat, 1);
  assert.equal(moved.assignment, null);
  assert.equal(moved.lastStateHash, null);
  assert.deepEqual(moved.projection, emptySeatProjection());
  assert.equal(moved.candidates.length, 2);
  assert.equal(moved.selectionStatus, "idle");

  const rebound = setGeneralSelectSeat(state, 0);
  assert.equal(rebound.assignment, null, "引擎重建后同座位也不能留旧成功态");
  assert.deepEqual(rebound.projection, emptySeatProjection());
});

test("点击候选：调用 chooseGeneral(seat,name)，成功后应用刷新信封并更新技能", async () => {
  const state = readyState(2);
  const chooseCalls = [];
  const applied = [];
  const pendingSeen = [];
  const payload = {
    assignment: {
      seat: 2,
      name: "甲",
      revision: 7,
      state_hash: "ab".repeat(32),
    },
    envelope: envelopeFor(2, ["鬼道", "天妒"]),
  };

  let resolveChoose;
  const promise = submitGeneralSelection(
    state,
    "甲",
    {
      chooseGeneral(seat, name) {
        chooseCalls.push([seat, name]);
        return new Promise((resolve) => {
          resolveChoose = resolve;
        });
      },
      applyEnvelope(envelope) {
        applied.push(envelope);
      },
    },
    (pending) => {
      pendingSeen.push({
        selectionStatus: pending.selectionStatus,
        pendingName: pending.pendingName,
        candidates: pending.candidates.length,
      });
    },
  );

  // 点击后立即进入 pending，并在等待内核时已经发起 chooseGeneral。
  assert.deepEqual(chooseCalls, [[2, "甲"]]);
  assert.deepEqual(pendingSeen, [
    { selectionStatus: "pending", pendingName: "甲", candidates: 2 },
  ]);

  resolveChoose(payload);
  const next = await promise;

  assert.equal(next.selectionStatus, "accepted");
  assert.equal(next.pendingName, null);
  assert.equal(next.error, null);
  assert.deepEqual(next.assignment, payload.assignment);
  assert.equal(next.lastStateHash, "ab".repeat(32));
  assert.deepEqual(next.projection.skillNames, ["鬼道", "天妒"]);
  assert.deepEqual(next.projection.skillIds, [0, 1]);
  assert.equal(applied.length, 1, "成功后必须应用刷新信封");
  assert.equal(applied[0], payload.envelope, "应用的就是 Worker 返回的信封");
  assert.equal(next.candidates.length, 2, "刷新不应丢掉候选目录");
});

test("选将失败：显示错误、不应用刷新信封、不留下成功态", async () => {
  const state = withSeatProjection(
    readyState(0),
    seatProjectionFromView(envelopeFor(0, ["旧技能"]).view, 0),
  );
  const before = structuredClone(state.projection);
  let applied = 0;

  const next = await submitGeneralSelection(state, "查无此将", {
    chooseGeneral: async () => {
      throw new Error("assign_general 被拒绝: NoSuchGeneral");
    },
    applyEnvelope: () => {
      applied += 1;
    },
  });

  assert.equal(applied, 0, "失败不得刷新主视图");
  assert.equal(next.selectionStatus, "rejected");
  assert.notEqual(next.selectionStatus, "accepted");
  assert.equal(next.assignment, null);
  assert.equal(next.lastStateHash, null);
  assert.equal(next.pendingName, null);
  assert.match(
    next.error ?? "",
    /选将失败（seat=0, name=查无此将）：assign_general 被拒绝: NoSuchGeneral/,
  );
  assert.deepEqual(next.projection, before, "失败必须保留点击前的技能投影");
});

test("失败后重试成功：清除错误并进入成功态", async () => {
  const state = readyState(0);
  const failed = await submitGeneralSelection(state, "甲", {
    chooseGeneral: async () => {
      throw "assign_general 被拒绝: NoSuchGeneral";
    },
  });
  assert.equal(failed.selectionStatus, "rejected");
  assert.match(failed.error ?? "", /assign_general 被拒绝/);

  const payload = {
    assignment: { seat: 0, name: "甲", revision: 7, state_hash: "ef".repeat(32) },
    envelope: envelopeFor(0, ["鬼道"]),
  };
  const ok = await submitGeneralSelection(failed, "甲", {
    chooseGeneral: async () => payload,
    applyEnvelope: () => undefined,
  });
  assert.equal(ok.selectionStatus, "accepted");
  assert.equal(ok.error, null);
  assert.deepEqual(ok.projection.skillNames, ["鬼道"]);
});

test("返回与请求不一致（错座位）不得显示成功", async () => {
  const next = await submitGeneralSelection(readyState(0), "甲", {
    chooseGeneral: async () => ({
      assignment: {
        seat: 1,
        name: "甲",
        revision: 7,
        state_hash: "ab".repeat(32),
      },
      envelope: envelopeFor(1, ["鬼道"]),
    }),
    applyEnvelope: () => assert.fail("错座位回包不得应用到主视图"),
  });
  assert.equal(next.selectionStatus, "rejected");
  assert.equal(next.assignment, null);
  assert.match(next.error ?? "", /选将返回无法确认装配：座位回显 1 ≠ 请求 0/);
});

test("畸形返回（缺刷新信封）不得显示成功", async () => {
  const next = await submitGeneralSelection(readyState(0), "甲", {
    chooseGeneral: async () => ({
      assignment: {
        seat: 0,
        name: "甲",
        revision: 7,
        state_hash: "ab".repeat(32),
      },
      envelope: null,
    }),
  });
  assert.equal(next.selectionStatus, "rejected");
  assert.match(next.error ?? "", /缺刷新信封 view/);
});

test("装配已生效但主视图刷新抛错：保持 accepted，同时显示刷新错误", async () => {
  const next = await submitGeneralSelection(readyState(0), "甲", {
    chooseGeneral: async () => ({
      assignment: {
        seat: 0,
        name: "甲",
        revision: 7,
        state_hash: "ab".repeat(32),
      },
      envelope: envelopeFor(0, ["鬼道"]),
    }),
    applyEnvelope: () => {
      throw new Error("store boom");
    },
  });
  assert.equal(next.selectionStatus, "accepted", "内核确实已装配，不能报成选将失败");
  assert.match(next.error ?? "", /选将已装配，但视图刷新失败：store boom/);
  assert.deepEqual(next.projection.skillNames, ["鬼道"], "面板仍按回包更新投影");
});

test("空候选名在发起请求前就被拒绝", async () => {
  let called = 0;
  const next = await submitGeneralSelection(readyState(0), "  ", {
    chooseGeneral: async () => {
      called += 1;
      return null;
    },
  });
  assert.equal(called, 0);
  assert.equal(next.selectionStatus, "rejected");
  assert.match(next.error ?? "", /候选名为空/);
});
